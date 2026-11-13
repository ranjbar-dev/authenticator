//! Session-only native encrypted backup review. No secret values enter summaries.
use super::components as c;
use crate::{
    backup::{
        BackupError, Counts, ImportMode, ImportOptions, ImportPlan, ImportPreview, MergeChoice,
    },
    domain::records::{
        Account, DeletionState, Folder, FolderFilter, Preferences, RecordState, Vault,
    },
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

type Selection = Result<Option<PathBuf>, BackupError>;
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
        Self::start_with(
            generation,
            watch,
            deadline,
            context,
            crate::platform::backup_picker::choose_import,
        )
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
            *slot.lock().unwrap_or_else(|e| e.into_inner()) =
                Some(if stopped() { Ok(None) } else { selection });
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expired_picker_never_publishes_selected_metadata_and_drop_never_waits()
    -> Result<(), Box<dyn std::error::Error>> {
        let picker = Picker::start_with(
            9,
            LifecycleWatch::healthy_for_test(),
            Instant::now(),
            egui::Context::default(),
            |_| Ok(Some(PathBuf::from("public-disposable-picker-sentinel"))),
        );
        let deadline = Instant::now() + std::time::Duration::from_secs(2);
        loop {
            if let Some(result) = picker.poll() {
                assert!(matches!(result, Ok(None)));
                break;
            }
            if Instant::now() >= deadline {
                return Err("picker test timeout".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let cancelled = picker.cancelled.clone();
        let result = picker.result.clone();
        let held = result.lock().unwrap_or_else(|e| e.into_inner());
        let started = Instant::now();
        drop(picker);
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
        assert!(cancelled.load(Ordering::SeqCst));
        drop(held);
        Ok(())
    }
}
impl Drop for Picker {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Busy {
    Preview,
    Plan,
    Apply,
}
pub(super) struct ImportForm {
    pub path: Option<PathBuf>,
    pub passphrase: Zeroizing<String>,
    pub preview: Option<Arc<ImportPreview>>,
    pub plan: Option<ImportPlan>,
    pub options: ImportOptions,
    pub busy: Option<Busy>,
    pub saving_preferences: bool,
    pub error: Option<&'static str>,
    pub focus: bool,
    pub opener: egui::Id,
    pub replace: Option<c::Dialog>,
    conflict_page: usize,
    folder_page: usize,
    account_page: usize,
    recovery_page: usize,
    membership_page: usize,
}
impl ImportForm {
    pub fn new(opener: egui::Id) -> Self {
        Self {
            path: None,
            passphrase: Zeroizing::new(String::new()),
            preview: None,
            plan: None,
            options: ImportOptions::default(),
            busy: None,
            saving_preferences: false,
            error: None,
            focus: true,
            opener,
            replace: None,
            conflict_page: 0,
            folder_page: 0,
            account_page: 0,
            recovery_page: 0,
            membership_page: 0,
        }
    }
    pub fn clear_secret(&mut self, ctx: &egui::Context) {
        self.passphrase.zeroize();
        c::clear_sensitive_widget_memory(ctx);
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.path.is_none() {
            return Err("Choose a local encrypted backup first.");
        }
        if !(12..=1024).contains(&self.passphrase.len()) {
            return Err(
                "Enter the exact backup passphrase, 12–1024 UTF-8 bytes. Spaces are kept exactly.",
            );
        }
        Ok(())
    }
    pub fn plan_options(&self) -> ImportOptions {
        let mut options = self.options.clone();
        if options.mode == ImportMode::Replace {
            options.account_choices.clear();
            options.folder_choices.clear();
        }
        options
    }
    pub fn reset_selection(&mut self) {
        self.preview = None;
        self.plan = None;
        self.options = ImportOptions::default();
        self.conflict_page = 0;
        self.folder_page = 0;
        self.account_page = 0;
        self.recovery_page = 0;
        self.membership_page = 0;
        self.error = None;
        self.focus = true;
    }
}
pub(super) fn error_message(error: BackupError) -> &'static str {
    match error {
        BackupError::Invalid => {
            "The backup passphrase is incorrect or the backup is damaged. Existing data is unchanged. Enter the passphrase again or choose another backup."
        }
        BackupError::Unsupported => {
            "This file or version is unsupported. Choose a native Authenticator encrypted backup (AUTHPBK1). Local AUTHLOC1 vaults and the reference app’s private .vault format are not supported; renaming a file does not convert it."
        }
        BackupError::Changed => {
            "The backup file or current vault changed after preview. Existing data was preserved. Go back, choose the backup and preview it again."
        }
        BackupError::Unavailable => {
            "Cannot read or save this backup. Check access and free disk space, then preview again. The existing vault is preserved if the write failed."
        }
        BackupError::Limit => {
            "This backup or merged vault exceeds supported limits. Nothing was imported. Choose a smaller backup or review Replace carefully."
        }
        BackupError::InvalidDecision => {
            "These choices cannot form a valid import. Change the conflict or folder choices and review again. Nothing was imported."
        }
        BackupError::Cancelled => {
            "Import was cancelled. An already admitted atomic write may finish completely. Unlock and inspect your vault before retrying."
        }
        BackupError::Locked => "Unlock the vault and preview the backup again.",
    }
}
pub(super) enum Action {
    None,
    Back,
    Pick,
    Preview,
    Plan,
    Apply(egui::Id),
    CancelAndLock,
}
const PAGE_SIZE: usize = 5;
fn pages(ui: &mut egui::Ui, id: &str, label: &str, page: &mut usize, count: usize) {
    let total = count.div_ceil(PAGE_SIZE).max(1);
    *page = (*page).min(total - 1);
    c::secondary(
        ui,
        &format!("Page {} of {total} · {count} records", *page + 1),
    );
    ui.push_id(id, |ui| {
        if total > 1 {
            // Keep both identities/positions stable as pages change so focused
            // Next never becomes Previous through conditional auto-ID reuse.
            let previous = ui
                .add_enabled_ui(*page > 0, |ui| {
                    c::button(
                        ui,
                        &format!("Previous {label} page"),
                        c::ButtonStyle::Secondary,
                    )
                })
                .inner;
            let next = ui
                .add_enabled_ui(*page + 1 < total, |ui| {
                    c::button(ui, &format!("Next {label} page"), c::ButtonStyle::Secondary)
                })
                .inner;
            if previous.clicked() {
                *page -= 1;
            }
            if next.clicked() {
                *page += 1;
            }
        }
    });
}
pub(super) fn counts(ui: &mut egui::Ui, title: &str, value: Counts) {
    c::heading(ui, title, 2);
    c::body(
        ui,
        &format!(
            "{} accounts · {} folders · {} memberships",
            value.accounts, value.folders, value.memberships
        ),
    );
    c::body(
        ui,
        &format!(
            "{} recovery sets · {} recovery codes · {} used codes · {} trashed records",
            value.recovery_sets, value.recovery_codes, value.used_codes, value.trashed_records
        ),
    );
}
fn state_text(state: RecordState) -> String {
    format!(
        "created {} · updated {} · {}",
        state.created_at.0,
        state.updated_at.0,
        match state.deletion {
            DeletionState::Active => "Active".to_owned(),
            DeletionState::Trashed { at } => format!("Trash since {}", at.0),
        }
    )
}
fn account(ui: &mut egui::Ui, label: &str, account: &Account) {
    let draft = account.credential();
    c::heading(ui, label, 3);
    c::body(ui, draft.service());
    c::body(
        ui,
        draft
            .account_identifier()
            .unwrap_or("No account identifier"),
    );
    if let Some(website) = draft.website() {
        c::secondary(ui, website);
    }
    c::secondary(
        ui,
        &format!(
            "ID {} · {:?} · {} digits · {} second period",
            account.id().as_uuid(),
            draft.settings().algorithm,
            draft.settings().digits.count(),
            draft.settings().period.seconds()
        ),
    );
    c::secondary(ui, &state_text(account.state()));
}
fn folder(ui: &mut egui::Ui, label: &str, folder: &Folder) {
    c::heading(ui, label, 3);
    c::body(ui, folder.name());
    c::secondary(
        ui,
        &format!("{:?} · ID {}", folder.color(), folder.id().as_uuid()),
    );
    c::secondary(ui, &state_text(folder.state()));
}
fn display(ui: &mut egui::Ui, vault: &Vault) {
    let p = vault.preferences();
    let codes = p.codes();
    c::body(
        ui,
        &format!(
            "Sort: {} · View: {:?}",
            super::codes::sort_label(p.sort()),
            p.view()
        ),
    );
    c::body(
        ui,
        &format!(
            "Search: {}",
            if codes.query.is_empty() {
                "(empty)"
            } else {
                &codes.query
            }
        ),
    );
    let selected = match codes.folder {
        FolderFilter::All => "All accounts".into(),
        FolderFilter::Unfiled => "Unfiled".into(),
        FolderFilter::Folder(id) => vault
            .folders()
            .iter()
            .find(|f| f.id() == id)
            .map(|f| format!("{} · {}", f.name(), id.as_uuid()))
            .unwrap_or_else(|| "All accounts (missing folder)".into()),
    };
    c::body(ui, &format!("Folder: {selected}"));
}
fn security(ui: &mut egui::Ui, p: &Preferences) {
    c::body(
        ui,
        &format!(
            "Lock after {} seconds · clipboard expiry {} seconds",
            p.lock_timeout_seconds(),
            p.clipboard_timeout_seconds()
        ),
    );
}
fn set_choice(choices: &mut Vec<(usize, MergeChoice)>, index: usize, value: MergeChoice) {
    if let Some((_, old)) = choices.iter_mut().find(|(i, _)| *i == index) {
        *old = value;
    } else {
        choices.push((index, value));
    }
}
fn options(ui: &mut egui::Ui, form: &mut ImportForm, preview: &ImportPreview) {
    counts(ui, "In this authenticated backup", preview.counts());
    c::secondary(
        ui,
        "Counts include active and trashed records. Used recovery-code dates, record dates, metadata and associations are retained. Timestamps below are Unix seconds. No seeds or recovery values are shown.",
    );
    c::labeled_selection(
        ui,
        "backup-import-mode",
        "Import mode",
        &mut form.options.mode,
        &[
            (ImportMode::Merge, "Merge · preserve existing data"),
            (ImportMode::Replace, "Replace · remove current collections"),
        ],
    );
    if form.options.mode == ImportMode::Replace {
        c::status(
            ui,
            "replace-warning",
            "Replace removes every current account, folder, membership and recovery record, including Trash. Consider exporting a current encrypted backup first. You will review exact counts and confirm before applying.",
            c::StatusKind::Error,
        );
    } else {
        c::body(
            ui,
            "Default: keep current conflicting account bundles; skip their incoming recovery codes and memberships. Keep both adds the complete incoming bundle with remapped IDs. Incoming duplicates and Trash also count as conflicts.",
        );
        c::heading(ui, "Account conflicts", 2);
        let conflicts = preview.conflicts();
        pages(
            ui,
            "conflict-pages",
            "conflict",
            &mut form.conflict_page,
            conflicts.len(),
        );
        for (row, conflict) in conflicts
            .iter()
            .enumerate()
            .skip(form.conflict_page * PAGE_SIZE)
            .take(PAGE_SIZE)
        {
            ui.push_id(("import-conflict", row), |ui| c::card(ui, |ui| {
                if let Some(index) = conflict.incoming_account {
                    account(ui, "Incoming account", &preview.incoming_vault().accounts()[index]);
                    c::body(ui, &format!("Matches {} current and {} earlier incoming accounts. Reasons: same ID {}; same secret/settings {}; same service/identifier {}. Reasons may describe different matches.", conflict.local_match_count, conflict.incoming_match_count, conflict.same_id, conflict.same_seed_and_settings, conflict.same_service_and_identifier));
                    if let Some(i) = conflict.local_account { account(ui, "Representative current match", &preview.local_vault().accounts()[i]); }
                    if let Some(i) = conflict.other_incoming_account { account(ui, "Representative earlier incoming match", &preview.incoming_vault().accounts()[i]); }
                    let mut value = form.options.account_choices.iter().find(|(i, _)| *i == index).map(|(_, v)| *v).unwrap_or(MergeChoice::KeepLocal);
                    if c::labeled_selection(ui, "account-resolution", "Decision for this incoming bundle", &mut value,
                        &[(MergeChoice::KeepLocal, "Keep current · skip incoming bundle"), (MergeChoice::KeepBoth, "Keep both · add incoming bundle")]) {
                        set_choice(&mut form.options.account_choices, index, value);
                    }
                } else {
                    c::body(ui, &format!("Folder ID collision: {} · equal folder names: {} · global ID collisions: {}. Review folder resolutions below; IDs are remapped together.", conflict.folder_id_collision, conflict.folder_name_collision, conflict.global_id_collisions));
                }
            }));
        }
        c::heading(ui, "Incoming folders", 2);
        c::secondary(
            ui,
            "Default keeps current metadata/Trash for the same stable folder ID. Different IDs stay distinct even with equal names. Explicit keep-current can reuse the lowest-ID same-name current folder; a cross-type ID collision may reuse exact content. Final review shows every resolution.",
        );
        let folders = preview.incoming_vault().folders();
        pages(
            ui,
            "folder-pages",
            "folder",
            &mut form.folder_page,
            folders.len(),
        );
        for (index, incoming) in folders
            .iter()
            .enumerate()
            .skip(form.folder_page * PAGE_SIZE)
            .take(PAGE_SIZE)
        {
            ui.push_id(("incoming-folder", index), |ui| {
                c::card(ui, |ui| {
                    folder(ui, "Incoming folder", incoming);
                    let mut value = form
                        .options
                        .folder_choices
                        .iter()
                        .find(|(i, _)| *i == index)
                        .map(|(_, v)| *v);
                    let allowed = preview.folder_decisions(index);
                    let mut choices = vec![(None, "Default folder policy")];
                    if allowed.is_some_and(|a| a.keep_both) {
                        choices.push((
                            Some(MergeChoice::KeepBoth),
                            "Keep both · separate folder copy",
                        ));
                    }
                    if allowed.is_some_and(|a| a.keep_local) {
                        choices.push((
                            Some(MergeChoice::KeepLocal),
                            "Keep current · reuse current folder",
                        ));
                    }
                    if choices.len() == 1 {
                        c::body(ui, "No current folder conflict. Import as a new folder with its associations.");
                    } else if c::labeled_selection(
                        ui,
                        "folder-resolution",
                        "Folder decision",
                        &mut value,
                        &choices,
                    ) {
                        form.options.folder_choices.retain(|(i, _)| *i != index);
                        if let Some(value) = value {
                            set_choice(&mut form.options.folder_choices, index, value);
                        }
                    }
                })
            });
        }
    }
    c::heading(ui, "Optional display settings", 2);
    c::labeled_selection(
        ui,
        "import-display",
        "Display settings to use",
        &mut form.options.import_display_preferences,
        &[
            (false, "Keep current display settings"),
            (true, "Use backup display settings"),
        ],
    );
    c::heading(ui, "Current display settings", 3);
    display(ui, preview.local_vault());
    c::heading(ui, "Backup display settings", 3);
    display(ui, preview.incoming_vault());
    c::heading(ui, "Optional security settings", 2);
    c::labeled_selection(
        ui,
        "import-security",
        "Security settings to use",
        &mut form.options.import_security_preferences,
        &[
            (false, "Keep current lock and clipboard timeouts"),
            (true, "Use backup lock and clipboard timeouts"),
        ],
    );
    c::heading(ui, "Current security settings", 3);
    security(ui, preview.local_vault().preferences());
    c::heading(ui, "Backup security settings", 3);
    security(ui, preview.incoming_vault().preferences());
    c::secondary(
        ui,
        "Longer timeouts increase exposure after you leave your device. Backup startup settings are never imported; local startup and local vault identity are always preserved. Import does not change Windows startup registration.",
    );
}
fn final_review(
    ui: &mut egui::Ui,
    form: &mut ImportForm,
    preview: &ImportPreview,
    plan: &ImportPlan,
) {
    let summary = plan.summary();
    c::heading(ui, "Review exact changes", 2);
    c::body(
        ui,
        if summary.mode == ImportMode::Merge {
            "Merge · existing records retained"
        } else {
            "Replace · current collections will be removed"
        },
    );
    counts(ui, "Current vault before import", summary.before);
    counts(ui, "Incoming backup", summary.incoming);
    counts(ui, "Vault after approved import", summary.after);
    if summary.mode == ImportMode::Replace {
        counts(
            ui,
            "Current records removed by Replace",
            summary.removed_local_records,
        );
    }
    c::body(
        ui,
        &format!(
            "{} incoming account bundles skipped (including their recovery codes and memberships). {} IDs remapped with associations.",
            summary.skipped_account_bundles, summary.remapped_ids
        ),
    );
    c::body(
        ui,
        "Trash and recovery used dates follow the reviewed candidate. Import never marks an unused recovery code used or restores Trash automatically.",
    );
    c::heading(ui, "Resulting accounts", 2);
    pages(
        ui,
        "candidate-accounts",
        "account",
        &mut form.account_page,
        plan.candidate_for_review().accounts().len(),
    );
    for a in plan
        .candidate_for_review()
        .accounts()
        .iter()
        .skip(form.account_page * PAGE_SIZE)
        .take(PAGE_SIZE)
    {
        c::card(ui, |ui| account(ui, "Account after import", a));
    }
    c::heading(ui, "Resulting account–folder associations", 2);
    let candidate = plan.candidate_for_review();
    pages(
        ui,
        "candidate-memberships",
        "association",
        &mut form.membership_page,
        candidate.memberships().len(),
    );
    for link in candidate
        .memberships()
        .iter()
        .skip(form.membership_page * PAGE_SIZE)
        .take(PAGE_SIZE)
    {
        c::card(ui, |ui| {
            if let Some(a) = candidate
                .accounts()
                .iter()
                .find(|a| a.id() == link.account_id())
            {
                c::heading(ui, "Linked account", 3);
                c::body(ui, a.credential().service());
                c::body(
                    ui,
                    a.credential()
                        .account_identifier()
                        .unwrap_or("No account identifier"),
                );
                c::secondary(ui, &format!("Account ID {}", a.id().as_uuid()));
            }
            if let Some(f) = candidate
                .folders()
                .iter()
                .find(|f| f.id() == link.folder_id())
            {
                folder(ui, "Linked folder", f);
            }
            c::secondary(ui, &format!("Membership ID {}", link.id().as_uuid()));
            c::body(ui, &format!("Association: {}", state_text(link.state())));
        });
    }
    c::heading(ui, "Resulting recovery states", 2);
    pages(
        ui,
        "candidate-recovery",
        "recovery",
        &mut form.recovery_page,
        summary.after.recovery_codes,
    );
    for (set, code) in plan
        .candidate_for_review()
        .backup_sets()
        .iter()
        .flat_map(|set| set.codes().iter().map(move |code| (set, code)))
        .skip(form.recovery_page * PAGE_SIZE)
        .take(PAGE_SIZE)
    {
        c::card(ui, |ui| {
            if let Some(a) = plan
                .candidate_for_review()
                .accounts()
                .iter()
                .find(|a| a.id() == set.account_id())
            {
                c::body(ui, a.credential().service());
                c::body(
                    ui,
                    a.credential()
                        .account_identifier()
                        .unwrap_or("No account identifier"),
                );
            }
            c::secondary(
                ui,
                &format!(
                    "Recovery record {} · set {}",
                    code.id().as_uuid(),
                    set.id().as_uuid()
                ),
            );
            c::body(
                ui,
                &code
                    .used_at()
                    .map(|at| format!("Used since {}", at.0))
                    .unwrap_or_else(|| "Unused".into()),
            );
            c::secondary(ui, &format!("Code: {}", state_text(code.state())));
            c::secondary(ui, &format!("Set: {}", state_text(set.state())));
        });
    }
    c::heading(ui, "Folder resolutions", 2);
    pages(
        ui,
        "resolved-folders",
        "folder resolution",
        &mut form.folder_page,
        summary.folder_copies.len(),
    );
    for resolution in summary
        .folder_copies
        .iter()
        .skip(form.folder_page * PAGE_SIZE)
        .take(PAGE_SIZE)
    {
        c::card(ui, |ui| {
            folder(
                ui,
                "Incoming folder",
                &preview.incoming_vault().folders()[resolution.incoming_folder],
            );
            if let Some(result) = plan
                .candidate_for_review()
                .folders()
                .iter()
                .find(|f| f.id() == resolution.resulting_id)
            {
                folder(ui, "Resulting folder", result);
            }
            c::body(
                ui,
                if resolution.copied {
                    "Copy incoming folder and metadata; map incoming memberships here."
                } else {
                    "Reuse current folder and its metadata/Trash; map incoming memberships here."
                },
            );
            c::secondary(
                ui,
                &format!(
                    "Same-ID collision {} · name collision {} · cross-type ID collision {} · exact content reused {}",
                    resolution.local_folder_id_collision,
                    resolution.local_folder_name_collision,
                    resolution.local_global_id_collision,
                    resolution.matching_content_reused
                ),
            );
        });
    }
    c::heading(
        ui,
        if summary.display_preferences_imported {
            "Display settings will change to"
        } else {
            "Display settings retained"
        },
        2,
    );
    display(ui, plan.candidate_for_review());
    if summary.selected_folder_reset {
        c::status(
            ui,
            "filter-reset",
            "The selected folder is absent after import. The account filter will reset to All accounts.",
            c::StatusKind::Info,
        );
    }
    c::heading(
        ui,
        if summary.security_preferences_imported {
            "Security settings will change to"
        } else {
            "Security settings retained"
        },
        2,
    );
    security(ui, plan.candidate_for_review().preferences());
    c::body(
        ui,
        &format!(
            "Local startup retained: {}. Local vault passphrase, key and identity are retained.",
            if plan.candidate_for_review().preferences().auto_start() {
                "enabled"
            } else {
                "disabled"
            }
        ),
    );
    c::secondary(
        ui,
        "Apply commits this complete validated candidate atomically. If the file or local vault changed, preview again. Cancel before applying leaves collections unchanged; display settings requested before preview may already have been saved.",
    );
}
pub(super) fn form(ui: &mut egui::Ui, form: &mut ImportForm, picking: bool) -> Action {
    if form.busy.is_some() || form.saving_preferences || picking {
        c::progress(
            ui,
            if form.saving_preferences {
                "Saving display settings before preview…"
            } else if picking {
                "Choosing local encrypted backup…"
            } else {
                match form.busy {
                    Some(Busy::Preview) => "Authenticating backup and finding conflicts…",
                    Some(Busy::Plan) => "Building validated import review…",
                    _ => "Applying approved encrypted import…",
                }
            },
            None,
        );
        c::secondary(
            ui,
            "Work stays on this device. Cancel locks the vault and hides this review immediately. An already admitted atomic write may finish completely; inspect the vault after unlocking before retrying.",
        );
        let cancel = c::button(ui, "Cancel and lock", c::ButtonStyle::Secondary);
        if form.focus {
            cancel.request_focus();
            form.focus = false;
        }
        return if cancel.clicked() {
            Action::CancelAndLock
        } else {
            Action::None
        };
    }
    let mut action = Action::None;
    let back = c::button(
        ui,
        if form.plan.is_some() {
            "Back to import choices"
        } else if form.preview.is_some() {
            "Back to backup file"
        } else {
            "Back to Export & Import"
        },
        c::ButtonStyle::Secondary,
    );
    if back.clicked()
        || (ui.ctx().memory(|m| m.top_modal_layer().is_none())
            && ui.input(|i| i.key_pressed(egui::Key::Escape)))
    {
        return Action::Back;
    }
    if form.focus {
        back.request_focus();
        form.focus = false;
    }
    if let Some(preview) = form.preview.clone() {
        if let Some(plan) = form.plan.take() {
            final_review(ui, form, &preview, &plan);
            form.plan = Some(plan);
        } else {
            options(ui, form, &preview);
        }
    } else {
        c::body(
            ui,
            "Choose a native Authenticator encrypted backup. The file is recognized by AUTHPBK1 content, regardless of extension. Local AUTHLOC1 and reference-app private .vault files are unsupported.",
        );
        if let Some(path) = &form.path {
            c::body(ui, &path.to_string_lossy());
        }
        if c::button(ui, "Choose backup to import…", c::ButtonStyle::Secondary).clicked() {
            action = Action::Pick;
        }
        c::labeled_secret_field(
            ui,
            "backup-import-passphrase",
            "Backup passphrase",
            &mut form.passphrase,
            "Enter this backup’s separate passphrase exactly. It cannot be recovered or reset.",
            None,
        );
    }
    ui.scope(|ui| {
        if let Some(error) = form.error {
            c::status(ui, "backup-import-error", error, c::StatusKind::Error);
        }
    });
    let label = if form.plan.is_some() {
        "Apply reviewed import"
    } else if form.preview.is_some() {
        "Review exact import changes"
    } else {
        "Unlock backup and preview"
    };
    let submit = c::button(ui, label, c::ButtonStyle::Primary);
    if submit.clicked() {
        action = if form.plan.is_some() {
            Action::Apply(submit.id)
        } else if form.preview.is_some() {
            Action::Plan
        } else {
            Action::Preview
        };
    }
    action
}
