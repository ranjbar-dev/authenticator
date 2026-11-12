//! Portable encrypted backups. Synchronous APIs belong on the sole store worker.
//! Preview/plan own authenticated snapshots; no plaintext export API exists.

use crate::{
    domain::records::*,
    error::AppError,
    storage::{KdfParameters, OperationCancellation, VaultStore, codec},
};
pub use files::Destination;
pub use merge::{
    Conflict, Counts, FolderCopy, ImportMode, ImportOptions, ImportSummary, MergeChoice,
};
use sha2::{Digest, Sha256};
use std::{fmt, path::Path};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackupError {
    Invalid,
    Unsupported,
    Unavailable,
    Changed,
    Cancelled,
    Limit,
    InvalidDecision,
    Locked,
}
impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Invalid => "The backup passphrase is incorrect or the backup is damaged.",
            Self::Unsupported => "This backup version is unsupported.",
            Self::Unavailable => "The backup operation could not access the selected file.",
            Self::Changed => "The reviewed file or vault changed. Review again before continuing.",
            Self::Cancelled => "The backup operation was cancelled.",
            Self::Limit => "The backup exceeds supported processing limits.",
            Self::InvalidDecision => "Review the import choices before continuing.",
            Self::Locked => "Unlock the vault to continue.",
        })
    }
}
impl std::error::Error for BackupError {}
type Result<T> = std::result::Result<T, BackupError>;
fn cancelled(c: &OperationCancellation) -> Result<()> {
    if c.is_cancelled() {
        Err(BackupError::Cancelled)
    } else {
        Ok(())
    }
}
fn fingerprint(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}
fn local_error(e: AppError) -> BackupError {
    match e {
        AppError::VaultLocked => BackupError::Locked,
        _ => BackupError::Unavailable,
    }
}
macro_rules! redacted {
    ($($name:ty),+) => {$(impl fmt::Debug for $name {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("Backup([REDACTED])") }
    })+};
}
mod envelope;
mod files;
mod merge;

/// Consumes the separately entered exact passphrase (12..1024 UTF-8 bytes).
/// Destination must have been explicitly reviewed, including overwrite consent.
pub fn export(
    store: &VaultStore,
    destination: Destination,
    passphrase: Zeroizing<String>,
    cancel: &OperationCancellation,
) -> Result<Counts> {
    cancelled(cancel)?;
    store.vault().map_err(local_error)?;
    crate::storage::crypto::check_passphrase(passphrase.as_bytes())
        .map_err(|_| BackupError::InvalidDecision)?;
    export_with_parameters(
        store,
        destination,
        passphrase,
        KdfParameters::calibrate().map_err(local_error)?,
        cancel,
    )
}
fn export_with_parameters(
    store: &VaultStore,
    destination: Destination,
    passphrase: Zeroizing<String>,
    parameters: KdfParameters,
    cancel: &OperationCancellation,
) -> Result<Counts> {
    cancelled(cancel)?;
    let vault = store.vault().map_err(local_error)?;
    merge::unique_ids(vault)?;
    let counts = Counts::of(vault);
    let bytes = envelope::encrypt(vault, passphrase.as_bytes(), parameters)?;
    cancelled(cancel)?;
    files::write(destination, &bytes, cancel, files::Fault::None)?;
    Ok(counts)
}

pub struct ImportPreview {
    source: files::Source,
    revision: [u8; 32],
    incoming: Vault,
    local: Vault,
    conflicts: Vec<Conflict>,
}
pub struct ImportPlan {
    source: files::Source,
    revision: [u8; 32],
    candidate: Vault,
    summary: ImportSummary,
}
redacted!(ImportPreview, ImportPlan);

pub fn preview(
    store: &VaultStore,
    path: &Path,
    passphrase: Zeroizing<String>,
    cancel: &OperationCancellation,
) -> Result<ImportPreview> {
    cancelled(cancel)?;
    let revision = fingerprint(store.backup_revision().map_err(local_error)?);
    let (source, bytes) = files::source(path)?;
    let incoming = envelope::decrypt(&bytes, passphrase.as_bytes())?;
    cancelled(cancel)?;
    merge::unique_ids(&incoming)?;
    let local = store.vault().map_err(local_error)?.clone();
    let conflicts = merge::conflicts(&local, &incoming)?;
    Ok(ImportPreview {
        source,
        revision,
        incoming,
        local,
        conflicts,
    })
}
impl ImportPreview {
    /// Sensitive authenticated review data. Show only while unlocked; never log
    /// or retain after Lock. The caller cannot change the reviewed snapshot.
    pub fn incoming_vault(&self) -> &Vault {
        &self.incoming
    }
    pub fn counts(&self) -> Counts {
        Counts::of(&self.incoming)
    }
    pub fn conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }
    /// Final reviewable candidate. Every decision is bound to this authenticated
    /// snapshot, not to subsequently supplied IDs, filenames or mutable records.
    pub fn plan(self, options: ImportOptions) -> Result<ImportPlan> {
        let (candidate, summary) =
            merge::build(&self.local, &self.incoming, &self.conflicts, options)?;
        Ok(ImportPlan {
            source: self.source,
            revision: self.revision,
            candidate,
            summary,
        })
    }
}
impl ImportPlan {
    /// Sensitive final candidate for the final confirmation view. Read-only and
    /// bound to this plan; folder resolutions refer to these resulting IDs.
    pub fn candidate_for_review(&self) -> &Vault {
        &self.candidate
    }
    pub fn summary(&self) -> &ImportSummary {
        &self.summary
    }
    /// Consuming apply rejects stale local revisions and changed source bytes.
    /// Only the existing transactional local writer can commit the candidate.
    pub fn apply(
        self,
        store: &mut VaultStore,
        cancel: &OperationCancellation,
    ) -> Result<ImportSummary> {
        cancelled(cancel)?;
        if fingerprint(store.backup_revision().map_err(local_error)?) != self.revision {
            return Err(BackupError::Changed);
        }
        self.source.check()?;
        let result = store.transaction_cancellable(cancel, |v| {
            *v = self.candidate;
            Ok(())
        });
        match result {
            Ok(()) => Ok(self.summary),
            Err(_) if cancel.is_cancelled() => Err(BackupError::Cancelled),
            Err(e) => Err(local_error(e)),
        }
    }
}

#[cfg(test)]
mod tests;
