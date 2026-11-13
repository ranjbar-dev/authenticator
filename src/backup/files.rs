//! Portable file handling deliberately does not use local directory ACLs/locks.
use super::*;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, PathBuf},
};

pub struct Destination {
    path: PathBuf,
    expected: Option<[u8; 32]>,
}
/// Inspected on a worker without granting overwrite consent. The UI must show
/// the selected path and obtain explicit consent before consuming `approve`.
pub struct DestinationReview(Destination);
redacted!(DestinationReview);
impl DestinationReview {
    pub fn needs_overwrite(&self) -> bool {
        self.0.expected.is_some()
    }
    /// Call only after the user's explicit export/overwrite action. No I/O.
    pub fn approve(self) -> Destination {
        self.0
    }
}
#[derive(Clone)]
pub(super) struct Source {
    path: PathBuf,
    digest: [u8; 32],
}
redacted!(Destination, Source);

fn safe(path: &Path) -> Result<()> {
    if !path.is_absolute() || path.file_name().is_none() {
        return Err(BackupError::Unavailable);
    }
    #[cfg(windows)]
    {
        use std::path::Prefix;
        // Only ordinary drive paths: exclude devices, UNC, ADS and ambiguous names.
        if !matches!(path.components().next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_)))
        {
            return Err(BackupError::Unavailable);
        }
        for c in path.components() {
            if let Component::Normal(n) = c {
                let s = n.to_str().ok_or(BackupError::Unavailable)?;
                let stem = s.split('.').next().unwrap_or_default().to_ascii_uppercase();
                if s.contains(':')
                    || s.ends_with([' ', '.'])
                    || s.contains('\0')
                    || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                    || (stem.len() == 4
                        && (stem.starts_with("COM") || stem.starts_with("LPT"))
                        && stem.as_bytes()[3].is_ascii_digit())
                {
                    return Err(BackupError::Unavailable);
                }
            }
        }
    }
    for p in path.ancestors() {
        match fs::symlink_metadata(p) {
            Ok(m) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if m.file_attributes() & 0x400 != 0 {
                        return Err(BackupError::Unavailable);
                    }
                }
                if m.file_type().is_symlink()
                    || (p != path && !m.is_dir())
                    || (p == path && !m.is_file())
                {
                    return Err(BackupError::Unavailable);
                }
            }
            Err(e) if p == path && e.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err(BackupError::Unavailable),
        }
    }
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(BackupError::Unavailable);
    }
    Ok(())
}
fn read(path: &Path) -> Result<Vec<u8>> {
    safe(path)?;
    let file = File::open(path).map_err(|_| BackupError::Unavailable)?;
    let m = file.metadata().map_err(|_| BackupError::Unavailable)?;
    if !m.is_file() {
        return Err(BackupError::Unavailable);
    }
    if m.len() > envelope::MAX_FILE as u64 {
        return Err(BackupError::Limit);
    }
    let mut bytes = Vec::new();
    file.take((envelope::MAX_FILE + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| BackupError::Unavailable)?;
    if bytes.len() > envelope::MAX_FILE {
        return Err(BackupError::Limit);
    }
    Ok(bytes)
}
fn digest(path: &Path) -> Result<Option<[u8; 32]>> {
    safe(path)?;
    match fs::symlink_metadata(path) {
        Ok(m) if m.is_file() => Ok(Some(fingerprint(&read(path)?))),
        Ok(_) => Err(BackupError::Unavailable),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(BackupError::Unavailable),
    }
}
impl Destination {
    /// Binds a future user decision to inspected bytes without granting consent.
    /// File inspection/hashing is synchronous and belongs on a worker.
    pub fn review(path: &Path) -> Result<DestinationReview> {
        Ok(DestinationReview(Self::new(path, digest(path)?)?))
    }
    /// A missing destination is explicit create-only intent. Existing files fail.
    pub fn create_new(path: &Path) -> Result<Self> {
        if digest(path)?.is_some() {
            return Err(BackupError::Changed);
        }
        Self::new(path, None)
    }
    /// Call only after explicit overwrite consent. Binds consent to exact bytes;
    /// revalidation at rename rejects changes since the review.
    pub fn overwrite_reviewed(path: &Path) -> Result<Self> {
        let expected = digest(path)?.ok_or(BackupError::Changed)?;
        Self::new(path, Some(expected))
    }
    fn new(path: &Path, expected: Option<[u8; 32]>) -> Result<Self> {
        // Prevent accidental replacement of native local storage artifacts.
        if path.file_name().is_some_and(|n| {
            n.eq_ignore_ascii_case("vault.bin") || n.eq_ignore_ascii_case("session.lock")
        }) {
            return Err(BackupError::Unavailable);
        }
        Ok(Self {
            path: path.to_owned(),
            expected,
        })
    }
}
pub(super) fn source(path: &Path) -> Result<(Source, Vec<u8>)> {
    let bytes = read(path)?;
    Ok((
        Source {
            path: path.to_owned(),
            digest: fingerprint(&bytes),
        },
        bytes,
    ))
}
impl Source {
    pub(super) fn check(&self) -> Result<()> {
        if fingerprint(&read(&self.path)?) != self.digest {
            return Err(BackupError::Changed);
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
pub(super) enum Fault {
    None,
    #[cfg(test)]
    Partial,
    #[cfg(test)]
    BeforeRename,
    #[cfg(test)]
    Terminate,
}
struct Pending(PathBuf, bool, Option<File>);
impl Drop for Pending {
    fn drop(&mut self) {
        drop(self.2.take());
        if self.1 {
            let _ = fs::remove_file(&self.0);
        }
    }
}
pub(super) fn write(
    destination: Destination,
    bytes: &[u8],
    cancel: &OperationCancellation,
    fault: Fault,
) -> Result<()> {
    let parent = destination.path.parent().ok_or(BackupError::Unavailable)?;
    let pending_path = parent.join(format!("pending-{}.bin", uuid::Uuid::new_v4()));
    write_at(destination, bytes, cancel, fault, pending_path)
}
pub(super) fn write_at(
    destination: Destination,
    bytes: &[u8],
    cancel: &OperationCancellation,
    fault: Fault,
    pending_path: PathBuf,
) -> Result<()> {
    cancelled(cancel)?;
    safe(&destination.path)?;
    let f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending_path)
        .map_err(|_| BackupError::Unavailable)?;
    // Activate cleanup only once create_new proves ownership of this name.
    let mut temp = Pending(pending_path, true, Some(f));
    let f = temp.2.as_mut().ok_or(BackupError::Unavailable)?;
    #[cfg(test)]
    if matches!(fault, Fault::Partial) {
        f.write_all(&bytes[..bytes.len() / 2])
            .map_err(|_| BackupError::Unavailable)?;
        return Err(BackupError::Unavailable);
    }
    let _ = fault;
    f.write_all(bytes).map_err(|_| BackupError::Unavailable)?;
    f.sync_all().map_err(|_| BackupError::Unavailable)?;
    drop(temp.2.take());
    #[cfg(test)]
    if matches!(fault, Fault::BeforeRename) {
        return Err(BackupError::Unavailable);
    }
    #[cfg(test)]
    if matches!(fault, Fault::Terminate) {
        std::process::exit(74);
    }
    let result = cancel.commit(|| {
        if digest(&destination.path).map_err(|_| AppError::StorageUnavailable)?
            != destination.expected
        {
            return Err(AppError::StaleAccount);
        }
        rename(&temp.0, &destination.path, destination.expected.is_some())
    });
    match result {
        Ok(()) => {
            // Native move consumed the name; do not delete an unrelated file
            // that another process might subsequently create at that path.
            if cfg!(windows) || destination.expected.is_some() {
                temp.1 = false;
            }
            Ok(())
        }
        Err(AppError::StaleAccount) => Err(BackupError::Changed),
        Err(_) if cancel.is_cancelled() => Err(BackupError::Cancelled),
        Err(_) => Err(BackupError::Unavailable),
    }
}
#[cfg(windows)]
fn rename(from: &Path, to: &Path, replace: bool) -> std::result::Result<(), AppError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };
    let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    let flags = MOVEFILE_WRITE_THROUGH
        | if replace {
            MOVEFILE_REPLACE_EXISTING
        } else {
            0
        };
    // SAFETY: bounded owned NUL-terminated paths live throughout the call.
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), flags) } == 0 {
        Err(AppError::StorageUnavailable)
    } else {
        Ok(())
    }
}
#[cfg(not(windows))]
fn rename(from: &Path, to: &Path, replace: bool) -> std::result::Result<(), AppError> {
    if replace {
        fs::rename(from, to)
    } else {
        fs::hard_link(from, to)
    }
    .map_err(|_| AppError::StorageUnavailable)
}
