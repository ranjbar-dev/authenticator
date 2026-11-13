//! UI state owns the repository, never a second decrypted credential cache.
use crate::{
    domain::{
        clock::{Clock, SystemClock},
        records::{
            AccountDraft, AccountId, BackupCode, CodesPreferences, DeletionState, RecordState,
            Timestamp, Vault,
        },
    },
    error::AppError,
    platform::{
        capture::{CaptureOutcome, CaptureTask},
        lifecycle::LifecycleWatch,
    },
    qr::ScanReview,
    storage::{OperationCancellation, VaultStore},
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use zeroize::Zeroize;
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Route {
    #[default]
    Codes,
    BackupCodes,
    Settings,
}
impl Route {
    pub const ALL: [Self; 3] = [Self::Codes, Self::BackupCodes, Self::Settings];
    pub fn title(self) -> &'static str {
        match self {
            Self::Codes => "2FA Codes",
            Self::BackupCodes => "Backup Codes",
            Self::Settings => "Settings",
        }
    }
}
/// Actual workflow intent, handed off to the later workflow modules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Workflow {
    AddAccount,
    AddBackupCodes,
    VaultLifecycle,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanAlternative {
    Image,
    Manual,
}

/// Explicit operation intent. An edit can only update its selected stable ID.
pub enum SaveIntent {
    TrashChange {
        review: crate::domain::trash::Review,
        duplicate_consent: bool,
        preferences: CodesPreferences,
    },
    FolderSave {
        target: crate::domain::folder_edit::FolderTarget,
        name: Zeroizing<String>,
        color: crate::domain::records::FolderColor,
        selected: std::collections::BTreeSet<AccountId>,
        duplicate_consent: bool,
        preferences: CodesPreferences,
    },
    FolderTrash {
        target: crate::domain::folder_edit::FolderTarget,
        preferences: CodesPreferences,
    },
    RecoveryCode {
        target: crate::domain::recovery_code::CodeTarget,
        change: crate::domain::recovery_code::CodeChange,
    },
    RecoverySave {
        target: crate::domain::recovery_edit::RecoveryTarget,
        values: Vec<Zeroizing<String>>,
    },
    RecoveryTrash(crate::domain::recovery_edit::RecoveryTarget),
    CodesPreferences(CodesPreferences),
    ClipboardTimeout(u32),
    IdleTimeout(u32),
    SecuritySettings {
        lock_seconds: u32,
        clipboard_seconds: u32,
        preferences: CodesPreferences,
    },
    StartupChange {
        enabled: bool,
        preferences: CodesPreferences,
        startup: crate::platform::startup::Startup,
    },
    Trash {
        id: AccountId,
        expected: RecordState,
    },
    Import(Vec<AccountDraft>),
    Create {
        draft: AccountDraft,
        codes: Vec<Zeroizing<String>>,
    },
    Edit {
        id: AccountId,
        expected: RecordState,
        original: AccountDraft,
        draft: AccountDraft,
        replace_secret: bool,
    },
}
impl SaveIntent {
    fn validate(&self) -> Result<(), AppError> {
        match self {
            Self::TrashChange { preferences, .. } => preferences.validate()?,
            Self::FolderSave {
                name,
                selected,
                preferences,
                ..
            } => {
                crate::domain::folder_edit::normalized_name(name)?;
                preferences.validate()?;
                if selected.len() > Vault::MAX_RECORDS {
                    return Err(AppError::FolderLimit);
                }
            }
            Self::FolderTrash { preferences, .. } => preferences.validate()?,
            Self::RecoverySave { values, .. } => {
                if values.is_empty() || values.len() > Vault::MAX_RECORDS {
                    return Err(AppError::InvalidBackupCodes);
                }
                for value in values {
                    BackupCode::normalized_value(value)
                        .map_err(|_| AppError::InvalidBackupCodes)?;
                }
            }
            Self::RecoveryTrash(_) | Self::RecoveryCode { .. } => {}
            Self::CodesPreferences(value) => value.validate()?,
            Self::StartupChange { preferences, .. } => preferences.validate()?,
            Self::SecuritySettings {
                lock_seconds,
                clipboard_seconds,
                preferences,
            } => {
                preferences.validate()?;
                if !(1..=86_400).contains(lock_seconds) || !(1..=86_400).contains(clipboard_seconds)
                {
                    return Err(AppError::InvalidRecord);
                }
            }
            Self::ClipboardTimeout(seconds) | Self::IdleTimeout(seconds) => {
                if !(1..=86_400).contains(seconds) {
                    return Err(AppError::InvalidRecord);
                }
            }
            Self::Trash { .. } => {}
            Self::Import(drafts) => {
                if drafts.is_empty() || drafts.len() > crate::qr::MAX_CANDIDATES {
                    return Err(AppError::InvalidRecord);
                }
                for draft in drafts {
                    draft.validate()?;
                }
            }
            Self::Create { draft, codes } => {
                draft.validate()?;
                if codes.len() > Vault::MAX_RECORDS {
                    return Err(AppError::InvalidBackupCodes);
                }
                for code in codes {
                    BackupCode::normalized_value(code).map_err(|_| AppError::InvalidBackupCodes)?;
                }
            }
            Self::Edit { draft, .. } => draft.validate()?,
        }
        Ok(())
    }
    pub(crate) fn apply(
        self,
        vault: &mut Vault,
        at: Timestamp,
    ) -> Result<Vec<AccountId>, AppError> {
        match self {
            Self::TrashChange {
                review,
                duplicate_consent,
                mut preferences,
            } => {
                review.apply(vault, duplicate_consent, at)?;
                if let crate::domain::records::FolderFilter::Folder(id) = preferences.folder
                    && !vault
                        .folders()
                        .iter()
                        .any(|f| f.id() == id && f.state().deletion == DeletionState::Active)
                {
                    preferences.folder = crate::domain::records::FolderFilter::All;
                }
                vault.set_codes_preferences(preferences, at)?;
                Ok(Vec::new())
            }
            Self::FolderSave {
                target,
                name,
                color,
                selected,
                duplicate_consent,
                preferences,
            } => {
                vault.save_folder(
                    &target,
                    crate::domain::folder_edit::FolderUpdate {
                        name: &name,
                        color,
                        selected: &selected,
                        duplicate_consent,
                    },
                    at,
                )?;
                vault.set_codes_preferences(preferences, at)?;
                Ok(Vec::new())
            }
            Self::FolderTrash {
                target,
                mut preferences,
            } => {
                vault.trash_folder(&target, at)?;
                if target.folder().is_some_and(|f| {
                    preferences.folder == crate::domain::records::FolderFilter::Folder(f.id())
                }) {
                    preferences.folder = crate::domain::records::FolderFilter::All;
                }
                vault.set_codes_preferences(preferences, at)?;
                Ok(Vec::new())
            }
            Self::RecoveryCode { target, change } => {
                vault.change_recovery_code(&target, change, at)?;
                Ok(Vec::new())
            }
            Self::RecoverySave { target, values } => {
                vault.save_recovery_codes(&target, values, at)?;
                Ok(Vec::new())
            }
            Self::RecoveryTrash(target) => {
                vault.trash_recovery_set(&target, at)?;
                Ok(Vec::new())
            }
            Self::CodesPreferences(value) => {
                vault.set_codes_preferences(value, at)?;
                Ok(Vec::new())
            }
            Self::ClipboardTimeout(seconds) => {
                vault.set_clipboard_timeout(seconds, at)?;
                Ok(Vec::new())
            }
            Self::SecuritySettings {
                lock_seconds,
                clipboard_seconds,
                preferences,
            } => {
                vault.set_lock_timeout(lock_seconds, at)?;
                vault.set_clipboard_timeout(clipboard_seconds, at)?;
                vault.set_codes_preferences(preferences, at)?;
                Ok(Vec::new())
            }
            Self::StartupChange {
                enabled,
                preferences,
                ..
            } => {
                vault.set_auto_start(enabled, at)?;
                vault.set_codes_preferences(preferences, at)?;
                Ok(Vec::new())
            }
            Self::IdleTimeout(seconds) => {
                let at = vault.mutation_time(at);
                vault.set_lock_timeout(seconds, at)?;
                Ok(Vec::new())
            }
            Self::Trash { id, expected } => {
                vault.trash_account(id, expected, at)?;
                Ok(Vec::new())
            }
            Self::Import(drafts) => drafts
                .into_iter()
                .map(|draft| vault.add_account(draft, at))
                .collect(),
            Self::Create { draft, codes } => {
                let at = vault.mutation_time(at);
                let codes = codes
                    .iter()
                    .map(|value| BackupCode::new(value, at))
                    .collect::<Result<Vec<_>, _>>()?;
                let id = vault.add_account(draft, at)?;
                if !codes.is_empty() {
                    vault.add_backup_set(id, codes, at)?;
                }
                Ok(vec![id])
            }
            Self::Edit {
                id,
                expected,
                original: snapshot,
                draft,
                replace_secret,
            } => {
                let original = vault
                    .accounts()
                    .iter()
                    .find(|a| a.id() == id)
                    .ok_or(AppError::StaleAccount)?;
                let live = original.credential();
                if original.state() != expected
                    || original.state().deletion != DeletionState::Active
                    || snapshot.service() != live.service()
                    || snapshot.website() != live.website()
                    || snapshot.account_identifier() != live.account_identifier()
                    || !snapshot.duplicate_criteria(live).0
                {
                    return Err(AppError::StaleAccount);
                }
                // Preservation is enforced at the transaction boundary too;
                // no placeholder/blank editor seed can overwrite the real seed.
                let draft = if replace_secret {
                    draft
                } else {
                    AccountDraft::new(
                        draft.service(),
                        draft.website(),
                        draft.account_identifier(),
                        original.credential().secret().clone(),
                        draft.settings(),
                    )?
                };
                vault.edit_account(id, draft, at)?;
                Ok(vec![id])
            }
        }
    }
}

pub struct AppState {
    repository: Option<Result<VaultStore, AppError>>,
    pub route: Route,
    pub query: String,
    /// Backup Codes has an independent, session-only search projection.
    pub backup_query: String,
    pending: Option<Workflow>,
    generation: u64,
    worker: Option<Worker>,
    last_activity: Instant,
    capture: Option<(u64, CaptureTask)>,
    review: Option<ScanReview>,
    capture_region: Option<crate::platform::capture::Region>,
    scan_alternative: Option<ScanAlternative>,
    imported: Option<Vec<AccountId>>,
    exported: Option<Result<(), crate::backup::BackupError>>,
    backup_result: Option<BackupResult>,
    trash_review: Option<Result<crate::domain::trash::Review, AppError>>,
    startup_result:
        Option<Result<crate::platform::startup::Status, crate::platform::startup::Error>>,
    image_busy: bool,
    #[cfg(test)]
    next_save_gate: Option<Arc<crate::storage::test_gate::CommitGate>>,
    #[cfg(test)]
    next_trash_review_gate: Option<Arc<crate::storage::test_gate::CommitGate>>,
}
struct Completion {
    repository: VaultStore,
    outcome: Result<(), AppError>,
    imported: Vec<AccountId>,
    exported: Option<Result<(), crate::backup::BackupError>>,
    backup_result: Option<BackupResult>,
    trash_review: Option<Result<crate::domain::trash::Review, AppError>>,
    startup_result:
        Option<Result<crate::platform::startup::Status, crate::platform::startup::Error>>,
}
/// Typed native-backup handoff, separate from QR/account import.
pub enum BackupRequest {
    Preview(std::path::PathBuf, Zeroizing<String>),
    Plan(
        Arc<crate::backup::ImportPreview>,
        crate::backup::ImportOptions,
    ),
    Apply(Box<crate::backup::ImportPlan>),
}
pub enum BackupResult {
    Preview(Result<Arc<crate::backup::ImportPreview>, crate::backup::BackupError>),
    Plan(Result<Box<crate::backup::ImportPlan>, crate::backup::BackupError>),
    Applied(Result<Box<crate::backup::ImportSummary>, crate::backup::BackupError>),
}
/// Worker-only post-publication check closes the race where UI cancel records
/// its request but try_lock misses a slot still held by the producer. After
/// publication unlocks the slot, either this check locks it or a later UI cancel
/// can acquire it itself. Poll independently rejects stale/cancelled adoption.
fn finish_publication(cancel: &OperationCancellation, result: &Mutex<Option<Completion>>) {
    if cancel.is_cancelled()
        && let Some(completion) = result.lock().unwrap_or_else(|e| e.into_inner()).as_mut()
    {
        completion.repository.lock();
        completion.backup_result = None;
        completion.trash_review = None;
        completion.startup_result = None;
    }
}
struct Worker {
    generation: u64,
    cancel: OperationCancellation,
    result: Arc<Mutex<Option<Completion>>>,
    importing: bool,
    deadline: Option<Instant>,
}
impl Worker {
    fn cancel(&self) {
        self.cancel.cancel();
        // Never wait on worker publication/cleanup from the UI. If publication
        // currently owns this slot, its cancellation check and the stale
        // generation check in poll still prevent unlocked UI adoption.
        let mut slot = match self.result.try_lock() {
            Ok(slot) => slot,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return,
        };
        if let Some(result) = slot.as_mut() {
            result.repository.lock();
            result.backup_result = None;
            result.trash_review = None;
            result.startup_result = None;
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VaultAction {
    Create,
    Unlock,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureEvent {
    ReviewReady,
    Cancelled,
    Locked,
    CaptureFailure(crate::platform::capture::CaptureError),
    DecodeFailure(crate::qr::QrError),
}
impl AppState {
    pub fn new(mut repository: Result<VaultStore, AppError>) -> Self {
        // A fresh UI owner starts locked even if its caller had a session.
        if let Ok(store) = &mut repository {
            store.lock();
        }
        Self {
            repository: Some(repository),
            route: Route::Codes,
            query: String::new(),
            backup_query: String::new(),
            pending: None,
            generation: 0,
            worker: None,
            last_activity: Instant::now(),
            capture: None,
            review: None,
            capture_region: None,
            scan_alternative: None,
            imported: None,
            exported: None,
            backup_result: None,
            trash_review: None,
            startup_result: None,
            image_busy: false,
            #[cfg(test)]
            next_save_gate: None,
            #[cfg(test)]
            next_trash_review_gate: None,
        }
    }
    pub fn is_unlocked(&self) -> bool {
        self.repository
            .as_ref()
            .is_some_and(|r| r.as_ref().is_ok_and(VaultStore::is_unlocked))
    }
    pub fn importing(&self) -> bool {
        self.worker.as_ref().is_some_and(|w| {
            w.importing && w.generation == self.generation && !w.cancel.is_cancelled()
        })
    }
    pub fn session_active(&self) -> bool {
        self.is_unlocked() || self.importing()
    }
    pub fn take_imported(&mut self) -> Option<Vec<AccountId>> {
        self.imported.take()
    }
    pub fn take_exported(&mut self) -> Option<Result<(), crate::backup::BackupError>> {
        self.exported.take()
    }
    pub fn take_backup_result(&mut self) -> Option<BackupResult> {
        self.backup_result.take()
    }
    pub fn take_trash_review(&mut self) -> Option<Result<crate::domain::trash::Review, AppError>> {
        self.trash_review.take()
    }
    pub fn take_startup_result(
        &mut self,
    ) -> Option<Result<crate::platform::startup::Status, crate::platform::startup::Error>> {
        self.startup_result.take()
    }
    pub fn idle_deadline(&self) -> Result<Instant, AppError> {
        Ok(self.last_activity
            + Duration::from_secs(self.vault()?.preferences().lock_timeout_seconds().into()))
    }
    pub fn vault(&self) -> Result<&Vault, AppError> {
        self.repository
            .as_ref()
            .ok_or(AppError::VaultLocked)?
            .as_ref()
            .map_err(|e| *e)?
            .vault()
    }
    /// Task 05 setup/unlock integration uses the same repository owner.
    pub fn repository_mut(&mut self) -> Result<&mut VaultStore, AppError> {
        self.repository
            .as_mut()
            .ok_or(AppError::VaultBusy)?
            .as_mut()
            .map_err(|e| *e)
    }
    pub fn storage_error(&self) -> Option<AppError> {
        self.repository
            .as_ref()
            .and_then(|r| r.as_ref().err().copied())
    }
    pub fn navigate(&mut self, route: Route) {
        self.route = route;
        self.pending = None;
    }
    pub fn request(&mut self, workflow: Workflow) -> Result<(), AppError> {
        if workflow != Workflow::VaultLifecycle && !self.is_unlocked() {
            return Err(AppError::VaultLocked);
        }
        self.pending = Some(workflow);
        Ok(())
    }
    pub fn pending(&self) -> Option<Workflow> {
        self.pending
    }
    pub fn cancel_pending(&mut self) {
        self.pending = None;
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn lock(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.query.zeroize();
        self.backup_query.zeroize();
        self.pending = None;
        if let Some((_, capture)) = &self.capture {
            capture.cancel();
        }
        self.review = None;
        self.imported = None;
        self.exported = None;
        self.backup_result = None;
        self.trash_review = None;
        self.startup_result = None;
        self.image_busy = false;
        self.capture_region = None;
        self.scan_alternative = None;
        if let Some(Ok(store)) = &mut self.repository {
            store.lock();
        }
        if let Some(worker) = &self.worker {
            worker.cancel();
        }
    }
    pub fn vault_exists(&self) -> Result<bool, AppError> {
        self.repository
            .as_ref()
            .ok_or(AppError::VaultBusy)?
            .as_ref()
            .map_err(|e| *e)?
            .exists()
    }
    pub fn busy(&self) -> bool {
        self.worker.is_some()
    }
    pub fn capture_busy(&self) -> bool {
        self.capture.is_some()
    }
    pub fn capture_decoding(&self) -> bool {
        self.capture.as_ref().is_some_and(|(_, c)| c.decoding())
    }
    pub fn begin_capture(
        &mut self,
        owner: usize,
        watch: LifecycleWatch,
        repaint: impl Fn() + Send + 'static,
    ) -> Result<(), AppError> {
        if !self.is_unlocked() {
            return Err(AppError::VaultLocked);
        }
        if self.capture_busy() || self.busy() || self.image_busy {
            return Err(AppError::VaultBusy);
        }
        self.review = None;
        let timeout = self.vault()?.preferences().lock_timeout_seconds();
        self.scan_alternative = None;
        self.capture_region = None;
        let deadline = self.last_activity + Duration::from_secs(timeout.into());
        self.capture = Some((
            self.generation,
            CaptureTask::start(owner, watch, deadline, repaint),
        ));
        Ok(())
    }
    pub fn cancel_capture(&mut self) {
        if let Some((_, capture)) = &self.capture {
            capture.cancel();
        }
        self.review = None;
        self.scan_alternative = None;
        self.capture_region = None;
    }
    pub fn begin_image(&mut self) -> Result<(), AppError> {
        if !self.is_unlocked() {
            return Err(AppError::VaultLocked);
        }
        if self.busy() || self.capture_busy() || self.image_busy {
            return Err(AppError::VaultBusy);
        }
        self.image_busy = true;
        Ok(())
    }
    pub fn finish_image(&mut self) {
        self.image_busy = false;
    }
    pub fn poll_capture(&mut self) -> Option<CaptureEvent> {
        let (generation, capture) = self.capture.as_ref()?;
        let mut outcome = capture.poll()?;
        let current = *generation == self.generation && self.is_unlocked();
        self.capture_region = if current { capture.region() } else { None };
        if !current {
            outcome = CaptureOutcome::Cancelled;
        }
        self.capture = None;
        Some(match outcome {
            CaptureOutcome::Review(review) => {
                self.review = Some(review);
                CaptureEvent::ReviewReady
            }
            CaptureOutcome::Cancelled => CaptureEvent::Cancelled,
            CaptureOutcome::Locked => CaptureEvent::Locked,
            CaptureOutcome::CaptureFailure(e) => CaptureEvent::CaptureFailure(e),
            CaptureOutcome::DecodeFailure(e) => CaptureEvent::DecodeFailure(e),
        })
    }
    /// Validated decoder drafts waiting to move into the ephemeral editor.
    pub fn scan_review(&self) -> Option<&ScanReview> {
        self.review.as_ref()
    }
    pub fn capture_region(&self) -> Option<crate::platform::capture::Region> {
        self.capture_region
    }
    pub fn request_scan_alternative(
        &mut self,
        alternative: ScanAlternative,
    ) -> Result<(), AppError> {
        if !self.is_unlocked() {
            return Err(AppError::VaultLocked);
        }
        if self.capture_busy() || self.busy() || self.image_busy {
            return Err(AppError::VaultBusy);
        }
        self.scan_alternative = Some(alternative);
        Ok(())
    }
    /// Optional typed workflow handoff; the shell also handles its own actions.
    pub fn take_scan_alternative(&mut self) -> Option<ScanAlternative> {
        if self.is_unlocked() {
            self.scan_alternative.take()
        } else {
            None
        }
    }
    pub fn take_scan_review(&mut self) -> Option<ScanReview> {
        if self.is_unlocked() {
            self.review.take()
        } else {
            None
        }
    }
    pub fn begin(
        &mut self,
        action: VaultAction,
        passphrase: Zeroizing<String>,
        repaint: impl Fn() + Send + 'static,
    ) -> Result<(), AppError> {
        if self.busy() || self.capture_busy() || self.image_busy {
            return Err(AppError::VaultBusy);
        }
        let mut repository = self
            .repository
            .take()
            .ok_or(AppError::VaultBusy)?
            .inspect_err(|&e| {
                self.repository = Some(Err(e));
            })?;
        let cancel = OperationCancellation::default();
        let result = Arc::new(Mutex::new(None));
        let worker = Worker {
            generation: self.generation,
            cancel: cancel.clone(),
            result: result.clone(),
            importing: false,
            deadline: None,
        };
        // No decrypted repository/key clone. Ownership travels to one worker.
        // The bounded KDF is cooperative: cannot interrupt Argon2 mid-call.
        std::thread::spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if cancel.is_cancelled() {
                    return Err(AppError::VaultLocked);
                }
                match action {
                    VaultAction::Create => repository.create_cancellable(
                        passphrase,
                        Vault::new(Timestamp(SystemClock.unix_seconds()?)),
                        &cancel,
                    ),
                    VaultAction::Unlock => repository.unlock(passphrase),
                }
            }))
            .unwrap_or(Err(AppError::StorageUnavailable));
            let mut slot = result.lock().unwrap_or_else(|e| e.into_inner());
            if cancel.is_cancelled() || outcome.is_err() {
                repository.lock();
            }
            *slot = Some(Completion {
                repository,
                outcome,
                imported: Vec::new(),
                exported: None,
                backup_result: None,
                trash_review: None,
                startup_result: None,
            });
            drop(slot);
            finish_publication(&cancel, &result);
            repaint();
        });
        self.worker = Some(worker);
        Ok(())
    }
    /// Validated approved drafts only; a single worker owns the repository.
    /// Recoverable write errors keep the session unlocked and the editor intact.
    pub fn begin_import(
        &mut self,
        drafts: Vec<AccountDraft>,
        watch: LifecycleWatch,
        repaint: impl Fn() + Send + 'static,
    ) -> Result<(), AppError> {
        self.begin_save(SaveIntent::Import(drafts), watch, repaint)
    }
    pub fn begin_save(
        &mut self,
        intent: SaveIntent,
        watch: LifecycleWatch,
        repaint: impl Fn() + Send + 'static,
    ) -> Result<(), AppError> {
        if self.busy() || self.capture_busy() || self.image_busy {
            return Err(AppError::VaultBusy);
        }
        if !self.is_unlocked() {
            return Err(AppError::VaultLocked);
        }
        intent.validate()?;
        let startup_change = match &intent {
            SaveIntent::StartupChange {
                enabled, startup, ..
            } => Some((*enabled, startup.clone())),
            _ => None,
        };
        let timeout = self.vault()?.preferences().lock_timeout_seconds();
        let deadline = self.last_activity + Duration::from_secs(timeout.into());
        let cancel =
            OperationCancellation::guarded(move || !watch.changed() && Instant::now() < deadline);
        #[cfg(test)]
        let cancel = cancel.with_test_gate(self.next_save_gate.take());
        let mut repository = self.repository.take().ok_or(AppError::VaultBusy)??;
        let result = Arc::new(Mutex::new(None));
        self.worker = Some(Worker {
            generation: self.generation,
            cancel: cancel.clone(),
            result: result.clone(),
            importing: true,
            deadline: Some(deadline),
        });
        std::thread::spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let at = Timestamp(SystemClock.unix_seconds()?);
                repository.transaction_cancellable(&cancel, |vault| intent.apply(vault, at))
            }));
            // The vault commit is authoritative for desired intent. OS registration is a
            // distinct guarded action; a failed OS write cannot roll back ciphertext.
            let startup_result = if matches!(&outcome, Ok(Ok(_))) {
                startup_change.map(|(enabled, startup)| {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        cancel.commit(|| Ok(startup.change(enabled)))
                    }))
                    .ok()
                    .and_then(Result::ok)
                    .unwrap_or(Err(crate::platform::startup::Error::Unavailable))
                })
            } else {
                None
            };
            let (outcome, imported) = match outcome {
                Ok(Ok(ids)) => (Ok(()), ids),
                Ok(Err(error)) => (Err(error), Vec::new()),
                Err(_) => {
                    repository.lock();
                    (Err(AppError::StorageUnavailable), Vec::new())
                }
            };
            let mut slot = result.lock().unwrap_or_else(|e| e.into_inner());
            if cancel.is_cancelled() {
                repository.lock();
            }
            *slot = Some(Completion {
                repository,
                outcome,
                imported,
                exported: None,
                backup_result: None,
                trash_review: None,
                startup_result,
            });
            drop(slot);
            finish_publication(&cancel, &result);
            repaint();
        });
        Ok(())
    }
    /// Read-only review preparation transfers the sole owner just like writes.
    /// Full-graph hashing stays off Root; cancellation can discard its result
    /// immediately even though the bounded hash itself runs to completion.
    pub fn begin_trash_review(
        &mut self,
        operation: crate::domain::trash::Operation,
        watch: LifecycleWatch,
        repaint: impl Fn() + Send + 'static,
    ) -> Result<(), AppError> {
        if self.busy() || self.capture_busy() || self.image_busy {
            return Err(AppError::VaultBusy);
        }
        if !self.is_unlocked() {
            return Err(AppError::VaultLocked);
        }
        let deadline = self.idle_deadline()?;
        let cancel =
            OperationCancellation::guarded(move || !watch.changed() && Instant::now() < deadline);
        let mut repository = self.repository.take().ok_or(AppError::VaultBusy)??;
        #[cfg(test)]
        let review_gate = self.next_trash_review_gate.take();
        let result = Arc::new(Mutex::new(None));
        self.worker = Some(Worker {
            generation: self.generation,
            cancel: cancel.clone(),
            result: result.clone(),
            importing: true,
            deadline: Some(deadline),
        });
        std::thread::spawn(move || {
            let reviewed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if cancel.is_cancelled() {
                    return Err(AppError::VaultLocked);
                }
                #[cfg(test)]
                if let Some(gate) = review_gate {
                    gate.hold_admitted_action();
                }
                if cancel.is_cancelled() {
                    return Err(AppError::VaultLocked);
                }
                crate::domain::trash::Review::new(repository.vault()?, operation)
            }))
            .unwrap_or_else(|_| {
                repository.lock();
                Err(AppError::StorageUnavailable)
            });
            let mut slot = result.lock().unwrap_or_else(|e| e.into_inner());
            let reviewed = if cancel.is_cancelled() {
                repository.lock();
                None
            } else {
                Some(reviewed)
            };
            *slot = Some(Completion {
                repository,
                outcome: Ok(()),
                imported: Vec::new(),
                exported: None,
                backup_result: None,
                trash_review: reviewed,
                startup_result: None,
            });
            drop(slot);
            finish_publication(&cancel, &result);
            repaint();
        });
        Ok(())
    }
    /// Export transfers the sole decrypted owner, with the same generation,
    /// lifecycle, idle and nonblocking publication checks as local writes.
    pub fn begin_export(
        &mut self,
        destination: crate::backup::Destination,
        passphrase: Zeroizing<String>,
        watch: LifecycleWatch,
        repaint: impl Fn() + Send + 'static,
    ) -> Result<(), AppError> {
        if self.busy() || self.capture_busy() || self.image_busy {
            return Err(AppError::VaultBusy);
        }
        if !self.is_unlocked() {
            return Err(AppError::VaultLocked);
        }
        let deadline = self.idle_deadline()?;
        let cancel =
            OperationCancellation::guarded(move || !watch.changed() && Instant::now() < deadline);
        #[cfg(test)]
        let cancel = cancel.with_test_gate(self.next_save_gate.take());
        let mut repository = self.repository.take().ok_or(AppError::VaultBusy)??;
        let result = Arc::new(Mutex::new(None));
        self.worker = Some(Worker {
            generation: self.generation,
            cancel: cancel.clone(),
            result: result.clone(),
            importing: true,
            deadline: Some(deadline),
        });
        std::thread::spawn(move || {
            let exported = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                crate::backup::export(&repository, destination, passphrase, &cancel).map(|_| ())
            }))
            .unwrap_or_else(|_| {
                repository.lock();
                Err(crate::backup::BackupError::Unavailable)
            });
            let mut slot = result.lock().unwrap_or_else(|e| e.into_inner());
            if cancel.is_cancelled() {
                repository.lock();
            }
            *slot = Some(Completion {
                repository,
                outcome: Ok(()),
                imported: Vec::new(),
                exported: Some(exported),
                backup_result: None,
                trash_review: None,
                startup_result: None,
            });
            drop(slot);
            finish_publication(&cancel, &result);
            repaint();
        });
        Ok(())
    }
    /// Native import preview, candidate construction and apply all run on the
    /// same sole-repository worker. No KDF, hashing, graph clone or I/O in paint.
    pub fn begin_backup(
        &mut self,
        request: BackupRequest,
        watch: LifecycleWatch,
        repaint: impl Fn() + Send + 'static,
    ) -> Result<(), AppError> {
        if self.busy() || self.capture_busy() || self.image_busy {
            return Err(AppError::VaultBusy);
        }
        if !self.is_unlocked() {
            return Err(AppError::VaultLocked);
        }
        let deadline = self.idle_deadline()?;
        let cancel =
            OperationCancellation::guarded(move || !watch.changed() && Instant::now() < deadline);
        #[cfg(test)]
        let cancel = cancel.with_test_gate(self.next_save_gate.take());
        let mut repository = self.repository.take().ok_or(AppError::VaultBusy)??;
        let result = Arc::new(Mutex::new(None));
        self.worker = Some(Worker {
            generation: self.generation,
            cancel: cancel.clone(),
            result: result.clone(),
            importing: true,
            deadline: Some(deadline),
        });
        std::thread::spawn(move || {
            use crate::backup::BackupError;
            let backup_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if cancel.is_cancelled() {
                    return None;
                }
                Some(match request {
                    BackupRequest::Preview(path, passphrase) => BackupResult::Preview(
                        crate::backup::preview(&repository, &path, passphrase, &cancel)
                            .map(Arc::new),
                    ),
                    BackupRequest::Plan(preview, options) => {
                        BackupResult::Plan(preview.plan(options).map(Box::new))
                    }
                    BackupRequest::Apply(plan) => {
                        BackupResult::Applied((*plan).apply(&mut repository, &cancel).map(Box::new))
                    }
                })
            }));
            let (outcome, mut backup_result) = match backup_result {
                Ok(result) => (Ok(()), result),
                Err(_) => {
                    repository.lock();
                    (
                        Err(AppError::StorageUnavailable),
                        Some(BackupResult::Applied(Err(BackupError::Unavailable))),
                    )
                }
            };
            let mut slot = result.lock().unwrap_or_else(|e| e.into_inner());
            if cancel.is_cancelled() {
                repository.lock();
                backup_result = None;
            }
            *slot = Some(Completion {
                repository,
                outcome,
                imported: Vec::new(),
                exported: None,
                backup_result,
                trash_review: None,
                startup_result: None,
            });
            drop(slot);
            finish_publication(&cancel, &result);
            repaint();
        });
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn hold_next_save_after_admission(
        &mut self,
        gate: Arc<crate::storage::test_gate::CommitGate>,
    ) {
        self.next_save_gate = Some(gate);
    }
    #[cfg(test)]
    pub(crate) fn hold_next_trash_review(
        &mut self,
        gate: Arc<crate::storage::test_gate::CommitGate>,
    ) {
        self.next_trash_review_gate = Some(gate);
    }
    pub fn poll(&mut self, now: Instant) -> Option<Result<(), AppError>> {
        let worker = self.worker.as_ref()?;
        let mut slot = match worker.result.try_lock() {
            Ok(slot) => slot,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return None,
        };
        let mut done = slot.take()?;
        drop(slot);
        let current = worker.generation == self.generation && !worker.cancel.is_cancelled();
        if !current {
            done.repository.lock();
        }
        if current && done.outcome.is_ok() && !done.imported.is_empty() {
            self.imported = Some(done.imported);
        }
        if current {
            self.exported = done.exported;
            self.backup_result = done.backup_result;
            self.trash_review = done.trash_review;
            self.startup_result = done.startup_result;
        }
        self.repository = Some(Ok(done.repository));
        self.worker = None;
        self.last_activity = now;
        current.then_some(done.outcome)
    }
    pub fn idle_due(&self, now: Instant) -> bool {
        if self.worker.as_ref().is_some_and(|w| {
            w.generation == self.generation && w.deadline.is_some_and(|d| now >= d)
        }) {
            return true;
        }
        self.vault().is_ok_and(|v| {
            now.saturating_duration_since(self.last_activity)
                >= Duration::from_secs(v.preferences().lock_timeout_seconds().into())
        })
    }
    pub fn activity(&mut self, now: Instant) {
        self.last_activity = now;
    }
    pub fn set_idle_timeout(
        &mut self,
        seconds: u32,
        watch: LifecycleWatch,
        repaint: impl Fn() + Send + 'static,
    ) -> Result<(), AppError> {
        self.begin_save(SaveIntent::IdleTimeout(seconds), watch, repaint)
    }
}
impl Drop for AppState {
    fn drop(&mut self) {
        self.lock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::records::Timestamp;
    use zeroize::Zeroizing;
    #[test]
    fn cancellation_does_not_wait_for_completion_publication()
    -> Result<(), Box<dyn std::error::Error>> {
        let result = Arc::new(Mutex::new(None));
        let token = OperationCancellation::default();
        let worker = Worker {
            generation: 0,
            cancel: token.clone(),
            result: result.clone(),
            importing: true,
            deadline: None,
        };
        std::thread::scope(|scope| {
            let held = result.lock().unwrap_or_else(|e| e.into_inner());
            let (sent, received) = std::sync::mpsc::sync_channel(1);
            scope.spawn(move || {
                worker.cancel();
                let _ = sent.send(());
            });
            let outcome = received.recv_timeout(Duration::from_secs(2));
            // Release before assertions/scope joins, even if old blocking code
            // caused timeout. The test must fail safely instead of hanging.
            drop(held);
            assert!(
                outcome.is_ok(),
                "UI cancellation must not wait for producer publication"
            );
            assert!(token.is_cancelled());
        });
        Ok(())
    }
    #[test]
    fn import_worker_overlap_write_error_retry_and_stale_completion()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::domain::{otp::OtpSettings, uri::parse_totp_uri};
        let directory = tempfile::tempdir()?;
        let mut store = VaultStore::open_directory(directory.path())?;
        store.create(
            Zeroizing::new("disposable-import-worker".into()),
            Vault::new(Timestamp(1)),
        )?;
        let mut state = AppState::new(Ok(store));
        state
            .repository_mut()?
            .unlock(Zeroizing::new("disposable-import-worker".into()))?;
        let lifecycle = crate::platform::lifecycle::Lifecycle::start(|| {})?;
        let draft = parse_totp_uri(
            "otpauth://totp/Public:Fixture?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Public",
        )?;
        assert_eq!(draft.settings(), OtpSettings::default());
        state.begin_image()?;
        assert_eq!(
            state.begin_import(vec![draft.clone()], lifecycle.watch(), || {}),
            Err(AppError::VaultBusy)
        );
        assert_eq!(
            state.begin_capture(0, lifecycle.watch(), || {}),
            Err(AppError::VaultBusy)
        );
        state.finish_image();
        state.capture = Some((state.generation, CaptureTask::queued(None)));
        assert_eq!(
            state.begin_import(vec![draft.clone()], lifecycle.watch(), || {}),
            Err(AppError::VaultBusy)
        );
        state.capture = None;
        let original = std::fs::read(directory.path().join("vault.bin"))?;
        // External replacement must fail closed without losing the unlocked
        // review session. Restore fixture bytes, then retry the approved batch.
        std::fs::write(
            directory.path().join("vault.bin"),
            b"disposable-write-conflict",
        )?;
        state.begin_import(vec![draft.clone(), draft.clone()], lifecycle.watch(), || {})?;
        assert_eq!(
            state.begin_import(vec![draft.clone()], lifecycle.watch(), || {}),
            Err(AppError::VaultBusy)
        );
        assert_eq!(finish(&mut state)?, Some(Err(AppError::StorageUnavailable)));
        assert!(state.is_unlocked());
        assert!(state.vault()?.accounts().is_empty());
        assert_eq!(
            std::fs::read(directory.path().join("vault.bin"))?,
            b"disposable-write-conflict"
        );
        std::fs::write(directory.path().join("vault.bin"), &original)?;
        state.begin_import(vec![draft.clone(), draft.clone()], lifecycle.watch(), || {})?;
        assert_eq!(finish(&mut state)?, Some(Ok(())));
        let ids = state.take_imported().ok_or("missing imported identities")?;
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
        assert_eq!(state.vault()?.accounts().len(), 2);
        assert_eq!(state.vault()?.duplicates(&draft, None).len(), 2);
        // Completed but unpolled imports must never re-open a locked generation.
        state.begin_import(vec![draft.clone(), draft.clone()], lifecycle.watch(), || {})?;
        std::thread::sleep(Duration::from_millis(250));
        state.lock();
        assert!(!state.session_active());
        assert_eq!(finish(&mut state)?, None);
        assert!(!state.is_unlocked());
        assert!(state.take_imported().is_none());
        state
            .repository_mut()?
            .unlock(Zeroizing::new("disposable-import-worker".into()))?;
        assert!(matches!(state.vault()?.accounts().len(), 2 | 4));
        let before_cancel = state.vault()?.accounts().len();
        state.begin_import(vec![draft.clone(), draft], lifecycle.watch(), || {})?;
        state.lock(); // cancel a newly started real worker without waiting for it
        assert_eq!(finish(&mut state)?, None);
        assert!(!state.session_active());
        state
            .repository_mut()?
            .unlock(Zeroizing::new("disposable-import-worker".into()))?;
        let after_cancel = state.vault()?.accounts().len();
        assert!(after_cancel == before_cancel || after_cancel == before_cancel + 2);
        Ok(())
    }
    #[test]
    fn capture_lock_gate_duplicate_and_stale_results() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let mut state = AppState::new(Ok(VaultStore::open_directory(dir.path())?));
        let listener = crate::platform::lifecycle::Lifecycle::start(|| {})?;
        assert_eq!(
            state.begin_capture(0, listener.watch(), || {}),
            Err(AppError::VaultLocked)
        );
        state.capture = Some((
            state.generation,
            CaptureTask::queued(Some(CaptureOutcome::DecodeFailure(
                crate::qr::QrError::NoQr,
            ))),
        ));
        state.lock();
        assert_eq!(state.poll_capture(), Some(CaptureEvent::Cancelled));
        assert!(state.scan_review().is_none());
        assert!(!state.capture_busy());
        // Late native lock must not become an unlocking action.
        state.capture = Some((
            state.generation,
            CaptureTask::queued(Some(CaptureOutcome::Locked)),
        ));
        assert_eq!(state.poll_capture(), Some(CaptureEvent::Cancelled));
        assert!(!state.is_unlocked());
        Ok(())
    }
    fn finish(
        state: &mut AppState,
    ) -> Result<Option<Result<(), AppError>>, Box<dyn std::error::Error>> {
        let deadline = Instant::now() + Duration::from_secs(45);
        let mut outcome = None;
        while state.busy() {
            if Instant::now() > deadline {
                return Err("worker did not complete".into());
            }
            outcome = state.poll(Instant::now()).or(outcome);
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(outcome)
    }
    #[test]
    fn worker_create_unlock_wrong_pass_cancel_idle_and_relaunch()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let pass = || Zeroizing::new("disposable-worker-only".into());
        let mut state = AppState::new(Ok(VaultStore::open_directory(dir.path())?));
        state.begin(VaultAction::Create, pass(), || {})?;
        assert!(state.busy());
        assert!(!state.is_unlocked());
        assert_eq!(
            state.begin(VaultAction::Create, pass(), || {}),
            Err(AppError::VaultBusy)
        );
        assert_eq!(
            state.request(Workflow::AddAccount),
            Err(AppError::VaultLocked)
        );
        assert_eq!(finish(&mut state)?, Some(Ok(())));
        assert_eq!(state.vault()?.preferences().lock_timeout_seconds(), 300);
        let now = Instant::now();
        state.activity(now);
        assert!(!state.idle_due(now + Duration::from_secs(299)));
        assert!(state.idle_due(now + Duration::from_secs(300)));
        assert!(!state.idle_due(now.checked_sub(Duration::from_secs(1)).ok_or("instant")?));
        let listener = crate::platform::lifecycle::Lifecycle::start(|| {})?;
        state.set_idle_timeout(3, listener.watch(), || {})?;
        assert_eq!(finish(&mut state)?, Some(Ok(())));
        state.activity(now);
        assert!(!state.idle_due(now + Duration::from_secs(2)));
        assert!(state.idle_due(now + Duration::from_secs(3)));
        let before = std::fs::read(dir.path().join("vault.bin"))?;
        state.lock();
        state.begin(
            VaultAction::Unlock,
            Zeroizing::new("wrong-worker-only".into()),
            || {},
        )?;
        assert_eq!(finish(&mut state)?, Some(Err(AppError::InvalidVault)));
        assert!(!state.is_unlocked());
        assert_eq!(std::fs::read(dir.path().join("vault.bin"))?, before);
        state.begin(VaultAction::Unlock, pass(), || {})?;
        // Cancel a real in-flight KDF. UI state never owns an unlocked store.
        std::thread::sleep(Duration::from_millis(100));
        state.lock();
        assert_eq!(finish(&mut state)?, None);
        assert!(!state.is_unlocked());
        state.begin(VaultAction::Unlock, pass(), || {})?;
        assert_eq!(finish(&mut state)?, Some(Ok(())));
        assert_eq!(state.vault()?.preferences().lock_timeout_seconds(), 3);
        drop(state);
        let reopened = AppState::new(Ok(VaultStore::open_directory(dir.path())?));
        assert!(!reopened.is_unlocked());
        assert!(reopened.vault_exists()?);
        Ok(())
    }
    #[test]
    fn cancellation_before_create_preserves_absence_and_corrupt_file_cannot_be_initialized()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let mut state = AppState::new(Ok(VaultStore::open_directory(dir.path())?));
        state.begin(
            VaultAction::Create,
            Zeroizing::new("disposable-cancel-only".into()),
            || {},
        )?;
        state.lock();
        assert_eq!(finish(&mut state)?, None);
        assert!(!state.is_unlocked());
        assert!(!state.vault_exists()?);
        std::fs::write(dir.path().join("vault.bin"), b"inert corrupt test file")?;
        state.begin(
            VaultAction::Unlock,
            Zeroizing::new("disposable-cancel-only".into()),
            || {},
        )?;
        assert_eq!(finish(&mut state)?, Some(Err(AppError::InvalidVault)));
        state.begin(
            VaultAction::Create,
            Zeroizing::new("disposable-cancel-only".into()),
            || {},
        )?;
        assert_eq!(finish(&mut state)?, Some(Err(AppError::StorageUnavailable)));
        assert_eq!(
            std::fs::read(dir.path().join("vault.bin"))?,
            b"inert corrupt test file"
        );
        Ok(())
    }
    #[test]
    fn lock_clears_completed_but_unpolled_result_and_stale_generation()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let mut store = VaultStore::open_directory(dir.path())?;
        store.create(
            Zeroizing::new("disposable-completion-only".into()),
            Vault::new(Timestamp(1)),
        )?;
        let slot = Arc::new(Mutex::new(Some(Completion {
            repository: store,
            outcome: Ok(()),
            imported: Vec::new(),
            exported: None,
            backup_result: None,
            trash_review: None,
            startup_result: None,
        })));
        let mut state = AppState::new(Err(AppError::VaultLocked));
        state.repository = None;
        state.worker = Some(Worker {
            generation: 0,
            cancel: OperationCancellation::default(),
            result: slot.clone(),
            importing: false,
            deadline: None,
        });
        state.query = "sensitive search".into();
        state.pending = Some(Workflow::AddAccount);
        state.lock();
        assert!(
            !slot
                .lock()
                .map_err(|_| "poison")?
                .as_ref()
                .ok_or("missing")?
                .repository
                .is_unlocked()
        );
        assert_eq!(state.poll(Instant::now()), None);
        assert!(!state.is_unlocked());
        assert!(state.query.is_empty());
        assert_eq!(state.pending(), None);
        Ok(())
    }
    #[test]
    fn routes_do_not_bypass_lock_or_install_workflows() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let mut state = AppState::new(Ok(VaultStore::open_directory(dir.path())?));
        for route in Route::ALL {
            state.navigate(route);
            assert_eq!(state.route, route);
            assert_eq!(state.vault().err(), Some(AppError::VaultLocked));
            assert_eq!(
                state.request(Workflow::AddAccount),
                Err(AppError::VaultLocked)
            );
            assert_eq!(
                state.request(Workflow::AddBackupCodes),
                Err(AppError::VaultLocked)
            );
            assert_eq!(state.pending(), None);
        }
        state.request(Workflow::VaultLifecycle)?;
        state.cancel_pending();
        assert_eq!(state.pending(), None);
        Ok(())
    }
    #[test]
    fn lock_drops_repository_access_query_and_pending_intent()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let mut store = VaultStore::open_directory(dir.path())?;
        store.create(
            Zeroizing::new("disposable-shell-only".into()),
            Vault::new(Timestamp(1)),
        )?;
        store.transaction(|vault| {
            use crate::domain::{
                otp::{OtpSettings, Secret},
                records::{AccountDraft, BackupCode},
            };
            let id = vault.add_account(
                AccountDraft::new(
                    "Disposable lock sentinel",
                    None,
                    Some("local-test"),
                    Secret::from_base32("MY")?,
                    OtpSettings::default(),
                )?,
                Timestamp(2),
            )?;
            vault.add_backup_set(
                id,
                vec![BackupCode::new("disposable-lock-only", Timestamp(3))?],
                Timestamp(3),
            )?;
            Ok(())
        })?;
        let mut state = AppState::new(Ok(store));
        assert!(!state.is_unlocked());
        state
            .repository_mut()?
            .unlock(Zeroizing::new("disposable-shell-only".into()))?;
        assert_eq!(state.vault()?.accounts().len(), 1);
        assert_eq!(state.vault()?.backup_sets().len(), 1);
        state.query = "private query".into();
        state.backup_query = "private backup query".into();
        state.request(Workflow::AddAccount)?;
        let generation = state.generation();
        state.lock();
        assert!(!state.is_unlocked());
        assert!(state.query.is_empty());
        assert!(state.backup_query.is_empty());
        assert_eq!(state.pending(), None);
        assert_ne!(state.generation(), generation);
        assert_eq!(state.vault().err(), Some(AppError::VaultLocked));
        Ok(())
    }
}
