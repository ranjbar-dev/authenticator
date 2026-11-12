//! Authenticated local-only storage. A repository owns the instance lock even
//! while locked. Callers never receive mutable access to committed records.
mod codec;
mod crypto;
mod files;

use crate::{domain::records::Vault, error::AppError};
pub use crypto::KdfParameters;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

/// Cooperative operation cancellation. The guard and atomic rename share a
/// mutex: cancellation before the rename gate prevents a create from committing;
/// cancellation after the gate may leave a complete encrypted, locked vault.
#[derive(Clone, Default)]
pub struct OperationCancellation {
    cancelled: std::sync::Arc<std::sync::Mutex<bool>>,
    guard: Option<std::sync::Arc<dyn Fn() -> bool + Send + Sync>>,
}
impl OperationCancellation {
    /// Additional fail-closed session/idle check at the actual rename gate.
    pub fn guarded(guard: impl Fn() -> bool + Send + Sync + 'static) -> Self {
        Self {
            cancelled: Default::default(),
            guard: Some(std::sync::Arc::new(guard)),
        }
    }
    pub fn cancel(&self) {
        *self.cancelled.lock().unwrap_or_else(|e| e.into_inner()) = true;
    }
    pub fn is_cancelled(&self) -> bool {
        *self.cancelled.lock().unwrap_or_else(|e| e.into_inner())
            || self.guard.as_ref().is_some_and(|g| !g())
    }
    pub(super) fn commit<R>(
        &self,
        action: impl FnOnce() -> Result<R, AppError>,
    ) -> Result<R, AppError> {
        let cancelled = self.cancelled.lock().unwrap_or_else(|e| e.into_inner());
        if *cancelled || self.guard.as_ref().is_some_and(|g| !g()) {
            return Err(AppError::VaultLocked);
        }
        action()
    }
}

struct Session {
    vault: Vault,
    dek: crypto::Key,
    wrapping: Vec<u8>,
}
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Session([REDACTED])")
    }
}

/// Use behind one application owner (or a Mutex). `&mut self` serializes all
/// writes/unlock/lock operations; an OS file lock rejects additional instances.
pub struct VaultStore {
    directory: PathBuf,
    _instance_lock: File,
    session: Option<Session>,
    committed: Option<Vec<u8>>,
    fault: files::WriteFault,
}
impl std::fmt::Debug for VaultStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VaultStore([REDACTED])")
    }
}
impl VaultStore {
    pub fn open_default() -> Result<Self, AppError> {
        Self::open_directory(files::default_directory()?)
    }
    /// Explicit directory boundary for packaging/tests. Never pass a user vault
    /// to tests; production UI uses `open_default` and exposes no path override.
    pub fn open_directory(directory: impl AsRef<Path>) -> Result<Self, AppError> {
        let directory = directory.as_ref().to_owned();
        files::reject_links(&directory)?;
        fs::create_dir_all(&directory).map_err(|_| AppError::StorageUnavailable)?;
        files::protect(&directory)?;
        let lock = files::exclusive(&directory)?;
        files::reject_links(&directory.join("vault.bin"))?;
        if directory.join("vault.bin").exists() {
            files::protect(&directory.join("vault.bin"))?;
        }
        Ok(Self {
            directory,
            _instance_lock: lock,
            session: None,
            committed: None,
            fault: files::WriteFault::None,
        })
    }
    pub fn exists(&self) -> Result<bool, AppError> {
        match fs::symlink_metadata(self.directory.join("vault.bin")) {
            Ok(m) if m.is_file() => Ok(true),
            Ok(_) => Err(AppError::StorageUnavailable),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(_) => Err(AppError::StorageUnavailable),
        }
    }
    pub fn is_unlocked(&self) -> bool {
        self.session.is_some()
    }
    pub fn lock(&mut self) {
        self.session = None;
        self.committed = None;
    }
    /// Takes ownership of passphrase storage so it clears on every return path.
    pub fn create(&mut self, passphrase: Zeroizing<String>, vault: Vault) -> Result<(), AppError> {
        self.create_cancellable(passphrase, vault, &OperationCancellation::default())
    }
    pub fn create_cancellable(
        &mut self,
        passphrase: Zeroizing<String>,
        vault: Vault,
        cancel: &OperationCancellation,
    ) -> Result<(), AppError> {
        if cancel.is_cancelled() {
            return Err(AppError::VaultLocked);
        }
        crypto::check_passphrase(passphrase.as_bytes())?;
        if self.exists()? {
            return Err(AppError::StorageUnavailable);
        }
        let parameters = KdfParameters::calibrate()?;
        self.create_guarded(passphrase, vault, parameters, cancel)
    }
    #[cfg(test)]
    fn create_with_parameters(
        &mut self,
        passphrase: Zeroizing<String>,
        vault: Vault,
        p: KdfParameters,
    ) -> Result<(), AppError> {
        self.create_guarded(passphrase, vault, p, &OperationCancellation::default())
    }
    fn create_guarded(
        &mut self,
        passphrase: Zeroizing<String>,
        vault: Vault,
        p: KdfParameters,
        cancel: &OperationCancellation,
    ) -> Result<(), AppError> {
        if cancel.is_cancelled() {
            return Err(AppError::VaultLocked);
        }
        if self.exists()? {
            return Err(AppError::StorageUnavailable);
        }
        let plain = codec::encode(&vault)?;
        let dek = Zeroizing::new(crypto::random::<32>()?);
        let wrapping = crypto::new_wrapping(passphrase.as_bytes(), &dek, p)?;
        let bytes = crypto::encrypt(&wrapping, &dek, plain)?;
        files::atomic_guarded(&self.directory, &bytes, false, self.fault, cancel)?;
        self.session = Some(Session {
            vault,
            dek,
            wrapping,
        });
        self.committed = Some(bytes);
        Ok(())
    }
    /// No writes or migrations occur on unlock. Unknown schemas fail closed;
    /// schema upgrades must later use the same detached transactional boundary.
    pub fn unlock(&mut self, passphrase: Zeroizing<String>) -> Result<(), AppError> {
        self.lock();
        let bytes = files::read(&self.directory.join("vault.bin"))?;
        let (dek, plain) = crypto::unlock(&bytes, passphrase.as_bytes())?;
        let vault = codec::decode(&plain)?;
        let wrapping = bytes[..116].to_vec();
        self.session = Some(Session {
            vault,
            dek,
            wrapping,
        });
        self.committed = Some(bytes);
        Ok(())
    }
    pub fn vault(&self) -> Result<&Vault, AppError> {
        self.session
            .as_ref()
            .map(|s| &s.vault)
            .ok_or(AppError::VaultLocked)
    }
    fn check_unchanged(&self) -> Result<(), AppError> {
        let current = files::read(&self.directory.join("vault.bin"))?;
        if self.committed.as_deref() != Some(current.as_slice()) {
            return Err(AppError::StorageUnavailable);
        }
        Ok(())
    }
    /// Transaction/import boundary: a closure works on a detached clone. Error,
    /// cancellation (return Err), panic, validation or write failure leaves both
    /// committed memory and disk unchanged. Do not return secrets in errors.
    pub fn transaction<R>(
        &mut self,
        edit: impl FnOnce(&mut Vault) -> Result<R, AppError>,
    ) -> Result<R, AppError> {
        self.transaction_cancellable(&OperationCancellation::default(), edit)
    }
    pub fn transaction_cancellable<R>(
        &mut self,
        cancel: &OperationCancellation,
        edit: impl FnOnce(&mut Vault) -> Result<R, AppError>,
    ) -> Result<R, AppError> {
        if cancel.is_cancelled() {
            return Err(AppError::VaultLocked);
        }
        let session = self.session.as_ref().ok_or(AppError::VaultLocked)?;
        let mut candidate = session.vault.clone();
        let result = edit(&mut candidate)?;
        let plain = codec::encode(&candidate)?;
        let bytes = crypto::encrypt(&session.wrapping, &session.dek, plain)?;
        self.check_unchanged()?;
        files::atomic_guarded(&self.directory, &bytes, true, self.fault, cancel)?;
        if let Some(session) = self.session.as_mut() {
            session.vault = candidate;
        }
        self.committed = Some(bytes);
        Ok(result)
    }
    /// Rewraps the same DEK under a fresh salt/nonce and calibrated KEK; data
    /// ciphertext and data nonce remain byte-identical. Requires an unlock.
    pub fn change_passphrase(&mut self, new_passphrase: Zeroizing<String>) -> Result<(), AppError> {
        crypto::check_passphrase(new_passphrase.as_bytes())?;
        if !self.is_unlocked() {
            return Err(AppError::VaultLocked);
        }
        let p = KdfParameters::calibrate()?;
        self.rewrap_with_parameters(new_passphrase, p)
    }
    fn rewrap_with_parameters(
        &mut self,
        pass: Zeroizing<String>,
        p: KdfParameters,
    ) -> Result<(), AppError> {
        let session = self.session.as_ref().ok_or(AppError::VaultLocked)?;
        let wrapping = crypto::new_wrapping(pass.as_bytes(), &session.dek, p)?;
        let old = self.committed.as_ref().ok_or(AppError::VaultLocked)?;
        let mut bytes = wrapping.clone();
        bytes.extend_from_slice(&old[116..]);
        self.check_unchanged()?;
        files::atomic(&self.directory, &bytes, true, self.fault)?;
        if let Some(session) = self.session.as_mut() {
            session.wrapping = wrapping;
        }
        self.committed = Some(bytes);
        Ok(())
    }
}
impl Drop for VaultStore {
    fn drop(&mut self) {
        self.lock();
    }
}

#[cfg(test)]
mod tests;
