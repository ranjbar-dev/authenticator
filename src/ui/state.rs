//! UI state owns the repository, never a second decrypted credential cache.
use crate::{domain::records::Vault, error::AppError, storage::VaultStore};
use zeroize::Zeroize;

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

pub struct AppState {
    repository: Result<VaultStore, AppError>,
    pub route: Route,
    pub query: String,
    pending: Option<Workflow>,
    generation: u64,
}
impl AppState {
    pub fn new(mut repository: Result<VaultStore, AppError>) -> Self {
        // A fresh UI owner starts locked even if its caller had a session.
        if let Ok(store) = &mut repository {
            store.lock();
        }
        Self {
            repository,
            route: Route::Codes,
            query: String::new(),
            pending: None,
            generation: 0,
        }
    }
    pub fn is_unlocked(&self) -> bool {
        self.repository.as_ref().is_ok_and(VaultStore::is_unlocked)
    }
    pub fn vault(&self) -> Result<&Vault, AppError> {
        self.repository.as_ref().map_err(|e| *e)?.vault()
    }
    /// Task 05 setup/unlock integration uses the same repository owner.
    pub fn repository_mut(&mut self) -> Result<&mut VaultStore, AppError> {
        self.repository.as_mut().map_err(|e| *e)
    }
    pub fn storage_error(&self) -> Option<AppError> {
        self.repository.as_ref().err().copied()
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
        if let Ok(store) = &mut self.repository {
            store.lock();
        }
        self.query.zeroize();
        self.pending = None;
        self.generation = self.generation.wrapping_add(1);
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
        let mut state = AppState::new(Ok(store));
        assert!(!state.is_unlocked());
        state
            .repository_mut()?
            .unlock(Zeroizing::new("disposable-shell-only".into()))?;
        assert!(state.vault()?.accounts().is_empty());
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
