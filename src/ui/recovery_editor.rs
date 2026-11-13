//! Account-linked concealed editor. Raw source paths never enter form state.
use super::{components as c, state::SaveIntent};
use crate::{
    domain::{
        records::*,
        recovery_edit::{RecoveryChanges, RecoveryTarget},
        recovery_input::{MAX_TEXT_BYTES, RecoveryInput},
    },
    error::AppError,
    platform::lifecycle::LifecycleWatch,
};
use eframe::egui;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
use zeroize::Zeroizing;

pub struct Preview {
    input: Zeroizing<String>,
    parsed: RecoveryInput,
    pub changes: RecoveryChanges,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::otp::{OtpSettings, Secret};
    use std::time::Duration;
    fn editor() -> Result<Editor, AppError> {
        let mut v = Vault::new(Timestamp(1));
        let id = v.add_account(
            AccountDraft::new(
                "Public worker fixture",
                None,
                None,
                Secret::from_base32("MY")?,
                OtpSettings::default(),
            )?,
            Timestamp(1),
        )?;
        Editor::new(&v, id)
    }
    fn finish(form: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
        let deadline = Instant::now() + Duration::from_secs(5);
        while form.job.is_some() {
            form.poll(1, true, &egui::Context::default());
            if Instant::now() > deadline {
                return Err("preview timeout".into());
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        Ok(())
    }
    #[test]
    fn recovery_file_job_previews_without_mutation_and_failures_keep_prior_edits()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("public-generated.txt");
        std::fs::write(&path, "\u{feff} public-alpha \r\n\nβeta\npublic-alpha\n")?;
        let mut form = editor()?;
        form.text = Zeroizing::new("prior-unsaved-public-input".into());
        let chosen = path.clone();
        form.job = Some(Job::start_with(
            1,
            LifecycleWatch::healthy_for_test(),
            Instant::now() + Duration::from_secs(5),
            egui::Context::default(),
            form.target.clone(),
            None,
            move |_| Ok(Some(chosen)),
        ));
        finish(&mut form)?;
        assert!(form.text.starts_with('\u{feff}'));
        assert_eq!(
            form.preview.as_ref().map(|p| (
                p.parsed.values.len(),
                p.parsed.blanks,
                p.parsed.duplicates
            )),
            Some((2, 1, 1))
        );
        assert!(matches!(
            form.save_intent(),
            Err(AppError::ReviewBackupDuplicates)
        ));
        let previous = form.text.clone();
        for bytes in [vec![0xff], vec![b'x'; MAX_TEXT_BYTES + 1]] {
            std::fs::write(&path, bytes)?;
            let chosen = path.clone();
            form.job = Some(Job::start_with(
                1,
                LifecycleWatch::healthy_for_test(),
                Instant::now() + Duration::from_secs(5),
                egui::Context::default(),
                form.target.clone(),
                None,
                move |_| Ok(Some(chosen)),
            ));
            finish(&mut form)?;
            assert_eq!(form.text, previous);
            assert!(form.error.is_some());
        }
        assert_eq!(std::fs::read_dir(dir.path())?.count(), 1);
        Ok(())
    }
    #[test]
    fn recovery_preview_expiry_stale_generation_and_drop_do_not_publish_values_or_wait()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut form = editor()?;
        form.text = Zeroizing::new("public-current-input".into());
        form.job = Some(Job::start(
            1,
            LifecycleWatch::healthy_for_test(),
            Instant::now(),
            egui::Context::default(),
            form.target.clone(),
            Some(Zeroizing::new("public-stale-input".into())),
        ));
        finish(&mut form)?;
        assert_eq!(&**form.text, "public-current-input");
        assert!(form.preview.is_none());
        let job = Job::start(
            2,
            LifecycleWatch::healthy_for_test(),
            Instant::now() + Duration::from_secs(5),
            egui::Context::default(),
            form.target.clone(),
            Some(Zeroizing::new("public-stale-input".into())),
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        let slot = job.result.clone();
        loop {
            if slot.lock().unwrap_or_else(|e| e.into_inner()).is_some() {
                break;
            }
            if Instant::now() > deadline {
                return Err("publication timeout".into());
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        form.job = Some(job);
        form.poll(1, true, &egui::Context::default());
        assert!(form.preview.is_none());
        assert_eq!(&**form.text, "public-current-input");
        let job = Job::start(
            1,
            LifecycleWatch::healthy_for_test(),
            Instant::now() + Duration::from_secs(5),
            egui::Context::default(),
            form.target.clone(),
            Some(form.text.clone()),
        );
        let slot = job.result.clone();
        let held = slot.lock().unwrap_or_else(|e| e.into_inner());
        let started = Instant::now();
        drop(job);
        assert!(started.elapsed() < Duration::from_secs(1));
        drop(held);
        Ok(())
    }
}
type ResultSlot = Result<Option<Preview>, AppError>;
pub struct Job {
    pub generation: u64,
    cancelled: Arc<AtomicBool>,
    result: Arc<Mutex<Option<ResultSlot>>>,
    from_file: bool,
}
impl Job {
    pub fn start(
        generation: u64,
        watch: LifecycleWatch,
        deadline: Instant,
        context: egui::Context,
        target: RecoveryTarget,
        input: Option<Zeroizing<String>>,
    ) -> Self {
        Self::start_with(
            generation,
            watch,
            deadline,
            context,
            target,
            input,
            crate::platform::backup_picker::choose_text,
        )
    }
    fn start_with(
        generation: u64,
        watch: LifecycleWatch,
        deadline: Instant,
        context: egui::Context,
        target: RecoveryTarget,
        input: Option<Zeroizing<String>>,
        choose: impl FnOnce(
            &dyn Fn() -> bool,
        ) -> Result<Option<std::path::PathBuf>, crate::backup::BackupError>
        + Send
        + 'static,
    ) -> Self {
        let from_file = input.is_none();
        let cancelled = Arc::new(AtomicBool::new(false));
        let result = Arc::new(Mutex::new(None));
        let token = cancelled.clone();
        let slot = result.clone();
        std::thread::spawn(move || {
            let stopped =
                || token.load(Ordering::SeqCst) || watch.changed() || Instant::now() >= deadline;
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if stopped() {
                    return Ok(None);
                }
                let input = match input {
                    Some(text) => text,
                    None => {
                        let Some(path) =
                            choose(&stopped).map_err(|_| AppError::BackupFileUnavailable)?
                        else {
                            return Ok(None);
                        };
                        if stopped() {
                            return Ok(None);
                        }
                        crate::platform::recovery_file::read(&path)?
                    }
                };
                let parsed = RecoveryInput::parse(&input)?;
                let changes = RecoveryChanges::preview(&target, &parsed.values);
                Ok(Some(Preview {
                    input,
                    parsed,
                    changes,
                }))
            }))
            .unwrap_or(Err(AppError::BackupFileUnavailable));
            let mut held = slot.lock().unwrap_or_else(|e| e.into_inner());
            *held = Some(if stopped() { Ok(None) } else { outcome });
            drop(held);
            // Close cancellation/publication race without making UI drop wait.
            if stopped() {
                *slot.lock().unwrap_or_else(|e| e.into_inner()) = Some(Ok(None));
            }
            context.request_repaint();
        });
        Self {
            generation,
            cancelled,
            result,
            from_file,
        }
    }
    fn poll(&self) -> Option<ResultSlot> {
        match self.result.try_lock() {
            Ok(mut slot) => slot.take(),
            Err(std::sync::TryLockError::Poisoned(e)) => e.into_inner().take(),
            Err(std::sync::TryLockError::WouldBlock) => None,
        }
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::SeqCst);
        match self.result.try_lock() {
            Ok(mut slot) => {
                slot.take();
            }
            Err(std::sync::TryLockError::Poisoned(e)) => {
                e.into_inner().take();
            }
            Err(std::sync::TryLockError::WouldBlock) => {}
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Confirmation {
    Discard,
    Replace,
    Delete,
    File,
}
pub enum Action {
    None,
    Preview,
    ReadFile,
    Save(Box<SaveIntent>),
    Cancel,
    Lock,
}
pub struct Editor {
    pub target: RecoveryTarget,
    pub text: Zeroizing<String>,
    pub reveal: bool,
    pub error: Option<String>,
    pub focus: bool,
    pub job: Option<Job>,
    pub saving: bool,
    pub deleting: bool,
    pub blocked: bool,
    pub sensitive_rect: Option<egui::Rect>,
    preview: Option<Preview>,
    duplicates_acknowledged: bool,
    dialog: Option<(Confirmation, c::Dialog)>,
    cancel_opener: Option<egui::Id>,
    delete_on_open: bool,
}
impl Editor {
    pub fn new(vault: &Vault, id: AccountId) -> Result<Self, AppError> {
        let target = RecoveryTarget::capture(vault, id)?;
        let mut text = Zeroizing::new(String::new());
        let blocked = target.set().is_some_and(|s| {
            s.state().deletion != DeletionState::Active
                || s.codes()
                    .iter()
                    .filter(|c| c.state().deletion == DeletionState::Active)
                    .try_fold(0usize, |n, c| n.checked_add(c.expose_value().len() + 1))
                    .map(|n| n.saturating_sub(1))
                    .is_none_or(|n| n > MAX_TEXT_BYTES)
        });
        if !blocked && let Some(set) = target.set() {
            for code in set
                .codes()
                .iter()
                .filter(|c| c.state().deletion == DeletionState::Active)
            {
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str(code.expose_value());
            }
        }
        Ok(Self {
            target,
            text,
            reveal: false,
            error: None,
            focus: true,
            job: None,
            saving: false,
            deleting: false,
            blocked,
            sensitive_rect: None,
            preview: None,
            duplicates_acknowledged: false,
            dialog: None,
            cancel_opener: None,
            delete_on_open: false,
        })
    }
    pub fn request_cancel(&mut self) {
        self.dialog = Some((
            Confirmation::Discard,
            c::Dialog::new(
                "Discard backup-code edits?",
                self.cancel_opener
                    .unwrap_or_else(|| egui::Id::unique("recovery-cancel")),
            ),
        ));
    }
    pub fn request_delete(&mut self, opener: egui::Id) {
        self.dialog = Some((
            Confirmation::Delete,
            c::Dialog::new("Move backup-code set to Trash?", opener),
        ));
    }
    /// Detail is replaced by this form before its confirmation appears. Bind
    /// cancellation to the actual editor Delete control when it is rendered,
    /// rather than retaining the vanished detail control's ID.
    pub fn request_delete_on_open(&mut self) {
        self.delete_on_open = true;
    }
    pub fn invalidate(&mut self) {
        self.preview = None;
        self.duplicates_acknowledged = false;
        self.error = None;
    }
    pub fn poll(&mut self, generation: u64, unlocked: bool, context: &egui::Context) -> bool {
        let Some(job) = &self.job else {
            return false;
        };
        let Some(result) = job.poll() else {
            return false;
        };
        let current = job.generation == generation && unlocked;
        let from_file = job.from_file;
        self.job = None;
        if current {
            match result {
                Ok(Some(preview)) => {
                    if from_file {
                        self.text = preview.input.clone();
                        self.reveal = false;
                        c::clear_sensitive_widget_memory(context);
                    }
                    if self.text == preview.input {
                        self.preview = Some(preview);
                        self.duplicates_acknowledged = false;
                        self.error = None;
                    }
                }
                Ok(None) => {}
                Err(error) => self.error = Some(error.to_string()),
            }
            self.focus = true;
        }
        true
    }
    pub fn save_intent(&self) -> Result<SaveIntent, AppError> {
        let preview = self
            .preview
            .as_ref()
            .filter(|p| p.input == self.text)
            .ok_or(AppError::InvalidBackupCodes)?;
        if self.blocked {
            return Err(AppError::InvalidBackupCodes);
        }
        if preview.parsed.duplicates > 0 && !self.duplicates_acknowledged {
            return Err(AppError::ReviewBackupDuplicates);
        }
        Ok(SaveIntent::RecoverySave {
            target: self.target.clone(),
            values: preview.parsed.values.clone(),
        })
    }
    fn can_save(&self) -> bool {
        !self.blocked
            && self.preview.as_ref().is_some_and(|p| {
                p.input == self.text && (p.parsed.duplicates == 0 || self.duplicates_acknowledged)
            })
    }
    pub fn content(&mut self, ui: &mut egui::Ui, vault: &Vault) -> Action {
        let mut action = Action::None;
        self.sensitive_rect = None;
        if self.saving {
            c::progress(ui, "Saving encrypted backup-code changes…", None);
            if c::button(ui, "Cancel and lock", c::ButtonStyle::Secondary).clicked() {
                action = Action::Lock;
            }
            return action;
        }
        super::backup_codes::selected_identity(ui, vault, self.target.account_id());
        c::heading(
            ui,
            if self.target.set().is_some() {
                "Edit Backup Codes"
            } else {
                "Add Backup Codes"
            },
            2,
        );
        if let Err(error) = self.target.check(vault) {
            self.blocked = true;
            self.error = Some(error.to_string());
        }
        if self.blocked {
            c::body(
                ui,
                "This set is preserved and cannot be edited here. Sets in Trash must be restored through Trash. A set larger than the 1 MiB editor limit remains intact; export an encrypted backup to preserve it.",
            );
        } else {
            ui.add_enabled_ui(self.job.is_none(), |ui| {
                let input = c::labeled_sensitive_multiline(ui, "recovery-text", "Backup codes · one per line", &mut self.text, self.reveal);
                self.sensitive_rect = Some(input.rect);
                if self.focus { input.request_focus(); self.focus = false; }
                if input.changed() { self.invalidate(); }
                c::secondary(ui, "Service-issued codes only. UTF-8 with optional BOM; CRLF or LF. Up to 1 MiB, 10,000 nonblank lines, 4096 bytes per code. Surrounding whitespace and blank lines are removed.");
                if c::button(ui, if self.reveal { "Conceal backup codes" } else { "Reveal backup codes" }, c::ButtonStyle::Secondary).clicked() {
                    self.reveal = !self.reveal;
                    c::clear_sensitive_widget_memory(ui.ctx());
                }
                let file = ui.push_id("recovery-file", |ui| c::button(ui, "Read Backup Codes from File", c::ButtonStyle::Secondary)).inner;
                if file.clicked() {
                    if self.text.is_empty() { action = Action::ReadFile; } else { self.dialog = Some((Confirmation::File, c::Dialog::new("Read a file instead of this input?", file.id))); }
                }
                if ui.push_id("recovery-preview", |ui| c::button(ui, "Preview changes", c::ButtonStyle::Secondary)).inner.clicked() { action = Action::Preview; }
            });
        }
        if self.job.is_some() {
            c::progress(ui, "Reading and previewing locally…", None);
            if c::button(ui, "Cancel reading · keep input", c::ButtonStyle::Secondary).clicked() {
                self.job = None;
                self.focus = true;
            }
        }
        if let Some(preview) = &self.preview {
            c::heading(ui, "Preview", 2);
            c::body(
                ui,
                &format!(
                    "{} unique valid code(s) · {} blank line(s) discarded · {} exact duplicate line(s)",
                    preview.parsed.values.len(),
                    preview.parsed.blanks,
                    preview.parsed.duplicates
                ),
            );
            let changes = preview.changes;
            c::body(
                ui,
                &format!(
                    "{} retained ({} used) · {} new and unused · {} removed to Trash · {} re-added from Trash ({} used) · {} kept in Trash",
                    changes.retained,
                    changes.retained_used,
                    changes.added,
                    changes.removed,
                    changes.restored,
                    changes.restored_used,
                    changes.kept_in_trash
                ),
            );
            if preview.parsed.duplicates > 0
                && ui
                    .push_id("recovery-deduplicate", |ui| {
                        c::button(
                            ui,
                            if self.duplicates_acknowledged {
                                "Duplicate removal · acknowledged"
                            } else {
                                "Remove exact duplicates when saving"
                            },
                            c::ButtonStyle::Secondary,
                        )
                    })
                    .inner
                    .clicked()
            {
                self.duplicates_acknowledged = !self.duplicates_acknowledged;
            }
        }
        c::secondary(
            ui,
            "Save keeps the IDs and used history of exact matching codes, including codes re-added from Trash. New codes start unused. Omitted codes remain encrypted in Trash with no automatic expiry. Copying never marks a code used.",
        );
        // Permanent region and explicit action IDs keep focus stable as errors change.
        ui.push_id("recovery-feedback", |ui| {
            if let Some(error) = &self.error {
                c::status(ui, "error", error, c::StatusKind::Error);
            }
        });
        ui.push_id("recovery-save", |ui| {
            ui.add_enabled_ui(self.job.is_none() && self.can_save(), |ui| {
                let save = c::button(ui, "Save backup codes", c::ButtonStyle::Primary);
                if save.clicked() {
                    if self.target.set().is_some() {
                        self.dialog = Some((
                            Confirmation::Replace,
                            c::Dialog::new("Save changes to this backup-code set?", save.id),
                        ));
                    } else if let Ok(intent) = self.save_intent() {
                        action = Action::Save(Box::new(intent));
                    }
                }
            });
        });
        let cancel = ui
            .push_id("recovery-cancel", |ui| {
                c::button(ui, "Cancel", c::ButtonStyle::Secondary)
            })
            .inner;
        self.cancel_opener = Some(cancel.id);
        if self.blocked && self.focus {
            cancel.request_focus();
            self.focus = false;
        }
        if cancel.clicked() {
            self.request_cancel();
        }
        if self
            .target
            .set()
            .is_some_and(|s| s.state().deletion == DeletionState::Active)
        {
            let delete = ui
                .push_id("recovery-delete", |ui| {
                    c::button(ui, "Delete backup-code set…", c::ButtonStyle::Secondary)
                })
                .inner;
            if delete.clicked() || self.delete_on_open {
                self.delete_on_open = false;
                self.request_delete(delete.id);
            }
        }
        if let Some((kind, mut dialog)) = self.dialog.take() {
            let mut consent = false;
            let close = dialog.show(ui.ctx(), |ui, first| {
                c::body(ui, match kind {
                    Confirmation::Discard => "Discard these unsaved edits? Stored backup codes stay unchanged.",
                    Confirmation::File => "A valid selected file replaces this unsaved input and opens a new preview. Cancel or a failed read keeps your input. The source file is never copied or changed.",
                    Confirmation::Replace => "Apply the previewed changes to this account's existing set? Exact retained and re-added codes keep used history. Removed codes move to Trash; no records are permanently deleted.",
                    Confirmation::Delete => "Move this entire backup-code set to Trash? Code values and used history remain encrypted with no automatic expiry. The 2FA account stays active. Open Settings → Trash to restore or permanently delete this set. Codes deleted separately stay in Trash.",
                });
                let cancel = c::button(ui, "Keep editing", c::ButtonStyle::Secondary);
                if first { cancel.request_focus(); }
                let confirm = c::button(ui, match kind { Confirmation::Discard => "Discard edits", Confirmation::File => "Choose local UTF-8 file", Confirmation::Replace => "Save reviewed changes", Confirmation::Delete => "Move set to Trash" }, c::ButtonStyle::Primary);
                if confirm.clicked() { consent = true; }
                cancel.clicked() || consent
            });
            if consent {
                action = match kind {
                    Confirmation::Discard => Action::Cancel,
                    Confirmation::File => Action::ReadFile,
                    Confirmation::Replace => match self.save_intent() {
                        Ok(intent) => Action::Save(Box::new(intent)),
                        Err(error) => {
                            self.error = Some(error.to_string());
                            Action::None
                        }
                    },
                    Confirmation::Delete => {
                        self.deleting = true;
                        Action::Save(Box::new(SaveIntent::RecoveryTrash(self.target.clone())))
                    }
                };
            } else if !close {
                self.dialog = Some((kind, dialog));
            }
        }
        action
    }
}
