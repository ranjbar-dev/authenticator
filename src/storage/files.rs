use super::crypto::{MAX_FILE, random};
use crate::error::AppError;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub(super) fn read(path: &Path) -> Result<Vec<u8>, AppError> {
    let file = File::open(path).map_err(|_| AppError::StorageUnavailable)?;
    if file
        .metadata()
        .map_err(|_| AppError::StorageUnavailable)?
        .len()
        > MAX_FILE as u64
    {
        return Err(AppError::InvalidVault);
    }
    // A bounded take also covers growth between metadata and read.
    let mut b = Vec::new();
    file.take((MAX_FILE + 1) as u64)
        .read_to_end(&mut b)
        .map_err(|_| AppError::StorageUnavailable)?;
    if b.len() > MAX_FILE {
        return Err(AppError::InvalidVault);
    }
    Ok(b)
}
pub(super) fn exclusive(dir: &Path) -> Result<File, AppError> {
    reject_links(dir)?;
    let path = dir.join("session.lock");
    reject_links(&path)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .map_err(|_| AppError::StorageUnavailable)?;
    file.try_lock().map_err(|_| AppError::VaultBusy)?;
    protect(&path)?;
    Ok(file)
}
pub(super) fn reject_links(path: &Path) -> Result<(), AppError> {
    match fs::symlink_metadata(path) {
        Ok(m) => {
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if m.file_attributes() & 0x400 != 0 {
                    return Err(AppError::StorageUnavailable);
                }
            }
            if m.file_type().is_symlink() {
                return Err(AppError::StorageUnavailable);
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(AppError::StorageUnavailable),
    }
}
#[derive(Clone, Copy, Default)]
pub(super) enum WriteFault {
    #[default]
    None,
    #[cfg(test)]
    AfterPartial,
    #[cfg(test)]
    BeforeReplace,
    #[cfg(test)]
    TerminateBeforeReplace,
}
struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
pub(super) fn atomic(
    dir: &Path,
    bytes: &[u8],
    replace: bool,
    fault: WriteFault,
) -> Result<(), AppError> {
    atomic_guarded(
        dir,
        bytes,
        replace,
        fault,
        &super::OperationCancellation::default(),
    )
}
pub(super) fn atomic_guarded(
    dir: &Path,
    bytes: &[u8],
    replace: bool,
    fault: WriteFault,
    cancel: &super::OperationCancellation,
) -> Result<(), AppError> {
    let destination = dir.join("vault.bin");
    reject_links(&destination)?;
    let temp = Temporary(dir.join(format!(
        "pending-{}.bin",
        uuid::Uuid::from_bytes(random::<16>()?)
    )));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp.0)
        .map_err(|_| AppError::StorageUnavailable)?;
    protect(&temp.0)?;
    #[cfg(test)]
    if matches!(fault, WriteFault::AfterPartial) {
        file.write_all(&bytes[..bytes.len() / 2])
            .map_err(|_| AppError::StorageUnavailable)?;
        return Err(AppError::StorageUnavailable);
    }
    let _ = fault;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| AppError::StorageUnavailable)?;
    drop(file);
    #[cfg(test)]
    if matches!(fault, WriteFault::BeforeReplace) {
        return Err(AppError::StorageUnavailable);
    }
    #[cfg(test)]
    if matches!(fault, WriteFault::TerminateBeforeReplace) {
        std::process::exit(73);
    }
    // The temp and destination are same-volume. No delete-original/rename pair.
    cancel.commit(|| replace_file(&temp.0, &destination, replace))?;
    // No fallible operation after the commit point: memory can now adopt candidate.
    Ok(())
}

#[cfg(windows)]
fn wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}
#[cfg(windows)]
fn replace_file(from: &Path, to: &Path, replace: bool) -> Result<(), AppError> {
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };
    let flags = MOVEFILE_WRITE_THROUGH
        | if replace {
            MOVEFILE_REPLACE_EXISTING
        } else {
            0
        };
    // SAFETY: Both NUL-terminated UTF-16 paths live for the synchronous call.
    if unsafe { MoveFileExW(wide(from).as_ptr(), wide(to).as_ptr(), flags) } == 0 {
        return Err(AppError::StorageUnavailable);
    }
    Ok(())
}
#[cfg(not(windows))]
fn replace_file(from: &Path, to: &Path, replace: bool) -> Result<(), AppError> {
    if !replace {
        fs::hard_link(from, to).map_err(|_| AppError::StorageUnavailable)?;
    } else {
        fs::rename(from, to).map_err(|_| AppError::StorageUnavailable)?;
    }
    Ok(())
}
#[cfg(windows)]
pub(super) fn default_directory() -> Result<PathBuf, AppError> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::{
        System::Com::CoTaskMemFree,
        UI::Shell::{FOLDERID_LocalAppData, SHGetKnownFolderPath},
    };
    let mut p = std::ptr::null_mut();
    // SAFETY: API allocates a NUL-terminated known-folder path; freed with its allocator.
    unsafe {
        if SHGetKnownFolderPath(&FOLDERID_LocalAppData, 0, std::ptr::null_mut(), &mut p) < 0 {
            // The API requires freeing the returned allocation even on failure;
            // CoTaskMemFree also accepts the initialized null pointer.
            CoTaskMemFree(p.cast());
            return Err(AppError::StorageUnavailable);
        }
        let mut len = 0;
        while *p.add(len) != 0 {
            len += 1;
        }
        let path = PathBuf::from(std::ffi::OsString::from_wide(std::slice::from_raw_parts(
            p, len,
        )));
        CoTaskMemFree(p.cast());
        Ok(path.join("Authenticator"))
    }
}
#[cfg(not(windows))]
pub(super) fn default_directory() -> Result<PathBuf, AppError> {
    Err(AppError::StorageUnavailable)
}

#[cfg(windows)]
pub(super) fn protect(path: &Path) -> Result<(), AppError> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, LocalFree},
        Security::{Authorization::*, *},
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    // SAFETY: Sized aligned token storage, checked native return values, synchronous
    // UTF-16 calls, and paired handle/LocalAlloc frees. No user data enters SDDL.
    unsafe {
        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(AppError::StorageUnavailable);
        }
        let mut data = [0usize; 512];
        let mut used = 0;
        let ok = GetTokenInformation(
            token,
            TokenUser,
            data.as_mut_ptr().cast(),
            std::mem::size_of_val(&data) as u32,
            &mut used,
        );
        CloseHandle(token);
        if ok == 0 || used as usize > std::mem::size_of_val(&data) {
            return Err(AppError::StorageUnavailable);
        }
        let user = &*data.as_ptr().cast::<TOKEN_USER>();
        let mut sid = std::ptr::null_mut();
        if ConvertSidToStringSidW(user.User.Sid, &mut sid) == 0 {
            return Err(AppError::StorageUnavailable);
        }
        let mut len = 0;
        while *sid.add(len) != 0 {
            len += 1;
        }
        let sid_text = String::from_utf16_lossy(std::slice::from_raw_parts(sid, len));
        LocalFree(sid.cast());
        let sddl: Vec<u16> = format!("D:P(A;OICI;FA;;;{sid_text})")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mut sd = std::ptr::null_mut();
        if ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut sd,
            std::ptr::null_mut(),
        ) == 0
        {
            return Err(AppError::StorageUnavailable);
        }
        let ok = SetFileSecurityW(
            wide(path).as_ptr(),
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            sd,
        );
        LocalFree(sd);
        if ok == 0 {
            return Err(AppError::StorageUnavailable);
        }
        Ok(())
    }
}
#[cfg(unix)]
pub(super) fn protect(path: &Path) -> Result<(), AppError> {
    use std::os::unix::fs::PermissionsExt;
    let mode = if path.is_dir() { 0o700 } else { 0o600 };
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|_| AppError::StorageUnavailable)
}
#[cfg(not(any(unix, windows)))]
pub(super) fn protect(_: &Path) -> Result<(), AppError> {
    Err(AppError::StorageUnavailable)
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    use crate::storage::OperationCancellation;
    #[test]
    fn cancelled_precommit_removes_pending_and_preserves_destination()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        fs::write(dir.path().join("vault.bin"), b"inert prior ciphertext")?;
        let cancel = OperationCancellation::default();
        cancel.cancel();
        assert_eq!(
            atomic_guarded(
                dir.path(),
                b"inert candidate ciphertext",
                true,
                WriteFault::None,
                &cancel
            ),
            Err(AppError::VaultLocked)
        );
        assert_eq!(
            fs::read(dir.path().join("vault.bin"))?,
            b"inert prior ciphertext"
        );
        assert_eq!(fs::read_dir(dir.path())?.count(), 1);
        Ok(())
    }
    #[test]
    fn cancellation_and_rename_gate_have_one_linearization_order()
    -> Result<(), Box<dyn std::error::Error>> {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc,
        };
        let token = OperationCancellation::default();
        let commit_token = token.clone();
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        let commit = std::thread::spawn(move || {
            commit_token.commit(|| {
                let _ = entered_tx.send(());
                let _ = release_rx.recv();
                Ok(())
            })
        });
        entered_rx.recv()?;
        let done = Arc::new(AtomicBool::new(false));
        let cancellation_done = done.clone();
        let cancel_token = token.clone();
        let (attempt_tx, attempt_rx) = mpsc::sync_channel(1);
        let cancellation = std::thread::spawn(move || {
            let _ = attempt_tx.send(());
            cancel_token.cancel();
            cancellation_done.store(true, Ordering::SeqCst);
        });
        attempt_rx.recv()?;
        assert!(!done.load(Ordering::SeqCst));
        release_tx.send(())?;
        assert_eq!(commit.join().map_err(|_| "commit panic")?, Ok(()));
        cancellation.join().map_err(|_| "cancel panic")?;
        assert!(token.is_cancelled());
        assert_eq!(token.commit(|| Ok(())), Err(AppError::VaultLocked));
        Ok(())
    }
}
