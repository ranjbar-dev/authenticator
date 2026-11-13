//! Session-only encrypted export form and cancellable off-UI destination review.
use super::{components as c, theme};
use crate::{
    backup::{BackupError, Destination, DestinationReview},
    platform::lifecycle::LifecycleWatch,
};
use eframe::egui;
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
use zeroize::{Zeroize, Zeroizing};

type Selection = Result<Option<(PathBuf, DestinationReview)>, BackupError>;
pub(super) struct Picker {
    pub generation: u64,
    cancelled: Arc<AtomicBool>,
    result: Arc<Mutex<Option<Selection>>>,
}
impl Picker {
    pub fn start(
        generation: u64,
        watch: LifecycleWatch,
        deadline: Instant,
        context: egui::Context,
    ) -> Self {
        Self::start_with(generation, watch, deadline, context, |stopped| {
            let Some(path) = crate::platform::backup_picker::choose(stopped)? else {
                return Ok(None);
            };
            if stopped() {
                return Ok(None);
            }
            let reviewed = Destination::review(&path)?;
            Ok(Some((path, reviewed)))
        })
    }
    fn start_with(
        generation: u64,
        watch: LifecycleWatch,
        deadline: Instant,
        context: egui::Context,
        choose: impl FnOnce(&dyn Fn() -> bool) -> Selection + Send + 'static,
    ) -> Self {
        let cancelled = Arc::new(AtomicBool::new(false));
        let result = Arc::new(Mutex::new(None));
        let token = cancelled.clone();
        let slot = result.clone();
        std::thread::spawn(move || {
            let stopped =
                || token.load(Ordering::SeqCst) || watch.changed() || Instant::now() >= deadline;
            let selection =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| choose(&stopped)))
                    .unwrap_or(Err(BackupError::Unavailable));
            let mut result = slot.lock().unwrap_or_else(|e| e.into_inner());
            // A deadline may expire while fresh ROOT activity extends the
            // session. Publish a nonsensitive cancellation so UI can release
            // the busy slot even then. Never publish the stale chosen path.
            *result = Some(if stopped() { Ok(None) } else { selection });
            drop(result);
            context.request_repaint();
        });
        Self {
            generation,
            cancelled,
            result,
        }
    }
    pub fn poll(&self) -> Option<Selection> {
        match self.result.try_lock() {
            Ok(mut slot) => slot.take(),
            Err(std::sync::TryLockError::Poisoned(e)) => e.into_inner().take(),
            Err(std::sync::TryLockError::WouldBlock) => None,
        }
    }
}
impl Drop for Picker {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::SeqCst);
        // Drop never waits for a native dialog, file read or publication.
    }
}
pub(super) struct ExportForm {
    pub path: Option<PathBuf>,
    pub destination: Option<DestinationReview>,
    pub passphrase: Zeroizing<String>,
    pub confirmation: Zeroizing<String>,
    pub error: Option<&'static str>,
    pub scroll_error: bool,
    pub success: bool,
    pub exporting: bool,
    pub saving_preferences: bool,
    pub focus: bool,
    pub overwrite: Option<c::Dialog>,
    pub opener: egui::Id,
}
impl ExportForm {
    pub fn new(opener: egui::Id) -> Self {
        Self {
            path: None,
            destination: None,
            passphrase: Zeroizing::new(String::new()),
            confirmation: Zeroizing::new(String::new()),
            error: None,
            scroll_error: false,
            success: false,
            exporting: false,
            saving_preferences: false,
            focus: true,
            overwrite: None,
            opener,
        }
    }
    pub fn clear_secrets(&mut self, context: &egui::Context) {
        self.passphrase.zeroize();
        self.confirmation.zeroize();
        c::clear_sensitive_widget_memory(context);
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.destination.is_none() {
            return Err("Choose a local destination before exporting.");
        }
        if self.passphrase.len() < 12 {
            return Err("Use at least 12 UTF-8 bytes: try several unrelated words.");
        }
        if self.passphrase.len() > 1024 {
            return Err("Use at most 1024 UTF-8 bytes for the backup passphrase.");
        }
        if self.passphrase.as_str() != self.confirmation.as_str() {
            return Err("The backup passphrases do not match. Enter the same passphrase twice.");
        }
        Ok(())
    }
    pub fn set_error(&mut self, error: &'static str) {
        self.error = Some(error);
        self.scroll_error = true;
        self.focus = false;
    }
    pub fn complete(&mut self, result: Result<(), BackupError>, context: &egui::Context) {
        self.clear_secrets(context);
        self.exporting = false;
        self.saving_preferences = false;
        self.success = result.is_ok();
        self.error = None;
        self.scroll_error = false;
        self.focus = self.success;
        if let Err(error) = result {
            self.set_error(error_message(error));
        }
        if !self.success {
            self.destination = None;
        }
    }
}
pub(super) fn error_message(error: BackupError) -> &'static str {
    match error {
        BackupError::Changed => {
            "The destination changed after selection. Choose the file again and review replacement before exporting."
        }
        BackupError::Unavailable => {
            "Cannot write this backup. Check folder permissions and free disk space, then choose a writable local destination and retry."
        }
        BackupError::Limit => {
            "This file or vault exceeds the supported backup limits. Choose a different destination; if the vault is too large, reduce it before retrying."
        }
        BackupError::Cancelled => {
            "The export was cancelled. If writing had already finished, a complete encrypted backup may exist at the selected destination."
        }
        BackupError::Locked => "Unlock the vault before exporting an encrypted backup.",
        _ => {
            "The encrypted backup could not be created. Choose a destination and enter the backup passphrase again to retry."
        }
    }
}
pub(super) enum Action {
    None,
    Back,
    Pick,
    Submit(egui::Id),
    CancelAndLock,
}
pub(super) fn form(ui: &mut egui::Ui, form: &mut ExportForm, picking: bool) -> Action {
    let mut action = Action::None;
    if form.exporting || form.saving_preferences {
        c::progress(
            ui,
            if form.saving_preferences {
                "Saving display settings before backup…"
            } else {
                "Encrypting and saving backup…"
            },
            None,
        );
        c::secondary(
            ui,
            if form.saving_preferences {
                "Your latest view, sort, search and folder selection will be saved before export starts. Cancelling locks the vault."
            } else {
                "Keep this window open until export finishes. Cancelling locks the vault; an already finished write may leave a complete encrypted backup."
            },
        );
        let cancel = c::button(ui, "Cancel and lock", c::ButtonStyle::Secondary);
        if form.focus {
            cancel.request_focus();
            form.focus = false;
        }
        if cancel.clicked() {
            return Action::CancelAndLock;
        }
        return action;
    }
    let back = c::button(ui, "Back to Export & Import", c::ButtonStyle::Secondary);
    if back.clicked()
        || (ui.ctx().memory(|m| m.top_modal_layer().is_none())
            && ui.input(|i| i.key_pressed(egui::Key::Escape)))
    {
        return Action::Back;
    }
    if form.success {
        c::status(
            ui,
            "export-success",
            "Encrypted backup saved. Keep its passphrase separately; it is required to restore this file.",
            c::StatusKind::Info,
        );
        if let Some(path) = &form.path {
            c::body(ui, &path.to_string_lossy());
        }
        if form.focus {
            back.request_focus();
            form.focus = false;
        }
        return action;
    }
    c::body(
        ui,
        "Includes all accounts, recovery codes and their used state, folders, Trash, and preferences. The device vault key and local vault passphrase are not exported.",
    );
    ui.add_space(theme::SMALL_GAP);
    c::heading(ui, "Local destination", 2);
    if let Some(path) = &form.path {
        c::body(ui, &path.to_string_lossy());
    } else {
        c::secondary(ui, "Choose where to keep your encrypted .authpbk file.");
    }
    if picking {
        c::progress(ui, "Choosing and checking destination…", None);
        c::secondary(
            ui,
            "Use the local Save dialog. Cancel there to keep this form, or lock the vault to close the dialog and clear this form.",
        );
        if c::button(ui, "Cancel and lock", c::ButtonStyle::Secondary).clicked() {
            action = Action::CancelAndLock;
        }
        return action;
    }
    let picker = c::button(ui, "Choose backup file…", c::ButtonStyle::Secondary);
    if form.focus {
        picker.request_focus();
        form.focus = false;
    }
    if picker.clicked() {
        action = Action::Pick;
    }
    let changed = c::labeled_secret_field(
        ui,
        "backup-export-passphrase",
        "Backup passphrase",
        &mut form.passphrase,
        "Use a separate, long passphrase. 12–1024 UTF-8 bytes; spaces are kept exactly.",
        None,
    )
    .changed();
    let confirm = c::labeled_secret_field(
        ui,
        "backup-export-confirmation",
        "Confirm backup passphrase",
        &mut form.confirmation,
        "Enter exactly the same backup passphrase.",
        None,
    );
    if changed || confirm.changed() {
        form.error = None;
    }
    c::secondary(
        ui,
        "This backup passphrase cannot be recovered or reset. Keep it separately from the file. Anyone with this passphrase and file can read the backup. Encryption is always enabled.",
    );
    // Keep the holder even when empty: inserting/removing it would change
    // Submit's auto-ID and break keyboard focus after validation/correction.
    let status = ui
        .scope(|ui| {
            if let Some(error) = form.error {
                c::status(ui, "export-error", error, c::StatusKind::Error);
            }
        })
        .response;
    if form.scroll_error && form.error.is_some() {
        status.scroll_to_me(Some(egui::Align::Center));
        form.scroll_error = false;
    }
    let submit = c::button(ui, "Export encrypted backup", c::ButtonStyle::Primary);
    if submit.clicked() || (confirm.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
        action = Action::Submit(submit.id);
    }
    action
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expired_picker_publishes_quiet_completion_without_selected_metadata()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("private-picker-sentinel.authpbk");
        let review = Destination::review(&path)?;
        let picker = Picker::start_with(
            7,
            LifecycleWatch::healthy_for_test(),
            Instant::now(),
            egui::Context::default(),
            move |_| Ok(Some((path, review))),
        );
        let deadline = Instant::now() + std::time::Duration::from_secs(2);
        loop {
            if let Some(selection) = picker.poll() {
                assert!(matches!(selection, Ok(None)));
                break;
            }
            if Instant::now() >= deadline {
                return Err("picker completion timeout".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(std::fs::read_dir(directory.path())?.next().is_none());
        Ok(())
    }
    #[test]
    fn picker_cancellation_never_waits_on_publication() {
        let cancelled = Arc::new(AtomicBool::new(false));
        let result = Arc::new(Mutex::new(None));
        let picker = Picker {
            generation: 0,
            cancelled: cancelled.clone(),
            result: result.clone(),
        };
        let held = result.lock().unwrap_or_else(|e| e.into_inner());
        let started = Instant::now();
        drop(picker);
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
        assert!(cancelled.load(Ordering::SeqCst));
        drop(held);
    }
}
