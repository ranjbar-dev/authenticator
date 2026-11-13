//! Native local Save dialog. Selection creates no file and grants no overwrite
//! consent: the ROOT review obtains that consent against the inspected bytes.
#[cfg(windows)]
pub fn choose(
    cancelled: &dyn Fn() -> bool,
) -> Result<Option<std::path::PathBuf>, crate::backup::BackupError> {
    use crate::backup::BackupError;
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::UI::{Controls::Dialogs::*, WindowsAndMessaging::*};
    struct Context<'a> {
        cancelled: &'a dyn Fn() -> bool,
    }
    unsafe extern "system" fn hook(
        hwnd: windows_sys::Win32::Foundation::HWND,
        message: u32,
        _: usize,
        lparam: isize,
    ) -> usize {
        // SAFETY: the hook and synchronous dialog share the live stack Context;
        // only this hook window stores/reads the pointer, on its owning thread.
        unsafe {
            if message == WM_INITDIALOG {
                let dialog = &*(lparam as *const OPENFILENAMEW);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, dialog.lCustData);
                SetTimer(hwnd, 1, 100, None);
            }
            if message == WM_TIMER {
                let context = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Context<'_>;
                if !context.is_null() && ((&*context).cancelled)() {
                    PostMessageW(GetParent(hwnd), WM_COMMAND, 2, 0);
                }
            }
            if message == WM_DESTROY {
                KillTimer(hwnd, 1);
            }
        }
        0
    }
    if cancelled() {
        return Ok(None);
    }
    let context = Context { cancelled };
    let mut buffer = zeroize::Zeroizing::new(vec![0u16; 32768]);
    let default: Vec<u16> = "Authenticator-backup.authpbk\0".encode_utf16().collect();
    buffer[..default.len()].copy_from_slice(&default);
    let filter: Vec<u16> = "Encrypted Authenticator backup\0*.authpbk\0\0"
        .encode_utf16()
        .collect();
    let title: Vec<u16> = "Choose encrypted backup destination\0"
        .encode_utf16()
        .collect();
    let extension: Vec<u16> = "authpbk\0".encode_utf16().collect();
    // SAFETY: all buffers/pointers outlive this bounded synchronous call. No
    // owner keeps ROOT available for Lock; cancellation is checked by the hook.
    unsafe {
        let mut dialog: OPENFILENAMEW = std::mem::zeroed();
        dialog.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        dialog.lpstrFile = buffer.as_mut_ptr();
        dialog.nMaxFile = buffer.len() as u32;
        dialog.lpstrFilter = filter.as_ptr();
        dialog.lpstrTitle = title.as_ptr();
        dialog.lpstrDefExt = extension.as_ptr();
        dialog.Flags = OFN_PATHMUSTEXIST
            | OFN_NOCHANGEDIR
            | OFN_DONTADDTORECENT
            | OFN_HIDEREADONLY
            | OFN_EXPLORER
            | OFN_ENABLEHOOK
            | OFN_NOTESTFILECREATE
            | OFN_NODEREFERENCELINKS;
        dialog.lpfnHook = Some(hook);
        dialog.lCustData = (&context as *const Context<'_>) as isize;
        if GetSaveFileNameW(&mut dialog) == 0 {
            return if cancelled() || CommDlgExtendedError() == 0 {
                Ok(None)
            } else {
                Err(BackupError::Unavailable)
            };
        }
    }
    if cancelled() {
        return Ok(None);
    }
    let length = buffer
        .iter()
        .position(|&v| v == 0)
        .ok_or(BackupError::Unavailable)?;
    let path = std::path::PathBuf::from(std::ffi::OsString::from_wide(&buffer[..length]));
    if !path.is_absolute() {
        return Err(BackupError::Unavailable);
    }
    Ok(Some(path))
}
#[cfg(not(windows))]
pub fn choose(
    _: &dyn Fn() -> bool,
) -> Result<Option<std::path::PathBuf>, crate::backup::BackupError> {
    Err(crate::backup::BackupError::Unavailable)
}
