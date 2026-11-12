//! UI state owns the repository, never a second decrypted credential cache.
use crate::{
    domain::{
        clock::{Clock, SystemClock},
        records::{Timestamp, Vault},
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

pub struct AppState {
    repository: Option<Result<VaultStore, AppError>>,
    pub route: Route,
    pub query: String,
    pending: Option<Workflow>,
    generation: u64,
    worker: Option<Worker>,
    last_activity: Instant,
    capture: Option<(u64, CaptureTask)>,
    review: Option<ScanReview>,
    capture_region: Option<crate::platform::capture::Region>,
    scan_alternative: Option<ScanAlternative>,
}
struct Completion {
    repository: VaultStore,
    outcome: Result<(), AppError>,
}
struct Worker {
    generation: u64,
    cancel: OperationCancellation,
    result: Arc<Mutex<Option<Completion>>>,
}
impl Worker {
    fn cancel(&self) {
        self.cancel.cancel();
        if let Some(result) = self
            .result
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_mut()
        {
            result.repository.lock();
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
            pending: None,
            generation: 0,
            worker: None,
            last_activity: Instant::now(),
            capture: None,
            review: None,
            capture_region: None,
            scan_alternative: None,
        }
    }
    pub fn is_unlocked(&self) -> bool {
        self.repository
            .as_ref()
            .is_some_and(|r| r.as_ref().is_ok_and(VaultStore::is_unlocked))
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
        self.query.zeroize();
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
        if let Some((_, capture)) = &self.capture {
            capture.cancel();
        }
        self.review = None;
        self.capture_region = None;
        self.scan_alternative = None;
        if let Some(Ok(store)) = &mut self.repository {
            store.lock();
        }
        if let Some(worker) = &self.worker {
            worker.cancel();
        }
        self.query.zeroize();
        self.pending = None;
        self.generation = self.generation.wrapping_add(1);
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
        if self.capture_busy() || self.busy() {
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
    /// Task08 moves these validated drafts into editable review; task07 never saves.
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
        if self.capture_busy() {
            return Err(AppError::VaultBusy);
        }
        self.scan_alternative = Some(alternative);
        Ok(())
    }
    /// Typed handoff only; image/manual editors are owned by later tasks.
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
        if self.busy() {
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
            });
            drop(slot);
            repaint();
        });
        self.worker = Some(worker);
        Ok(())
    }
    pub fn poll(&mut self, now: Instant) -> Option<Result<(), AppError>> {
        let worker = self.worker.as_ref()?;
        let mut done = worker
            .result
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()?;
        let current = worker.generation == self.generation && !worker.cancel.is_cancelled();
        if !current {
            done.repository.lock();
        }
        self.repository = Some(Ok(done.repository));
        self.worker = None;
        self.last_activity = now;
        current.then_some(done.outcome)
    }
    pub fn idle_due(&self, now: Instant) -> bool {
        self.vault().is_ok_and(|v| {
            now.saturating_duration_since(self.last_activity)
                >= Duration::from_secs(v.preferences().lock_timeout_seconds().into())
        })
    }
    pub fn activity(&mut self, now: Instant) {
        self.last_activity = now;
    }
    pub fn set_idle_timeout(&mut self, seconds: u32) -> Result<(), AppError> {
        let at = Timestamp(SystemClock.unix_seconds()?);
        self.repository_mut()?
            .transaction(|v| v.set_lock_timeout(seconds, at))
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
        state.set_idle_timeout(3)?;
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
        })));
        let mut state = AppState::new(Err(AppError::VaultLocked));
        state.repository = None;
        state.worker = Some(Worker {
            generation: 0,
            cancel: OperationCancellation::default(),
            result: slot.clone(),
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
        state.request(Workflow::AddAccount)?;
        let generation = state.generation();
        state.lock();
        assert!(!state.is_unlocked());
        assert!(state.query.is_empty());
        assert_eq!(state.pending(), None);
        assert_ne!(state.generation(), generation);
        assert_eq!(state.vault().err(), Some(AppError::VaultLocked));
        Ok(())
    }
}
