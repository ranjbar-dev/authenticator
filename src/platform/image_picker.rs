//! Unowned native picker on its own worker. ROOT stays usable for lock/cancel.
//! Returned paths are consumed by the bounded decoder and never logged/cached.
#[cfg(windows)]
pub fn choose(
    cancelled: &dyn Fn() -> bool,
) -> Result<Option<std::path::PathBuf>, crate::qr::QrError> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::UI::Controls::Dialogs::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    struct Context<'a> {
        cancelled: &'a dyn Fn() -> bool,
    }
    unsafe extern "system" fn hook(
        hwnd: windows_sys::Win32::Foundation::HWND,
        message: u32,
        _: usize,
        lparam: isize,
    ) -> usize {
        // SAFETY: lCustData points to the stack Context living through the
        // synchronous dialog. Only this hook's child stores/uses that pointer.
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
    let context = Context { cancelled };
    let mut buffer = zeroize::Zeroizing::new(vec![0u16; 32768]);
    let filter: Vec<u16> = "QR images (PNG, JPEG)\0*.png;*.jpg;*.jpeg\0\0"
        .encode_utf16()
        .collect();
    let title: Vec<u16> = "Import a local QR image\0".encode_utf16().collect();
    // SAFETY: All pointers refer to live, bounded storage for the synchronous
    // common-dialog call. No COM apartment is required by this Win32 API.
    unsafe {
        let mut dialog: OPENFILENAMEW = std::mem::zeroed();
        dialog.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        dialog.lpstrFile = buffer.as_mut_ptr();
        dialog.nMaxFile = buffer.len() as u32;
        dialog.lpstrFilter = filter.as_ptr();
        dialog.lpstrTitle = title.as_ptr();
        dialog.Flags = OFN_FILEMUSTEXIST
            | OFN_PATHMUSTEXIST
            | OFN_NOCHANGEDIR
            | OFN_DONTADDTORECENT
            | OFN_HIDEREADONLY
            | OFN_EXPLORER
            | OFN_ENABLEHOOK;
        dialog.lpfnHook = Some(hook);
        dialog.lCustData = (&context as *const Context<'_>) as isize;
        if GetOpenFileNameW(&mut dialog) == 0 {
            return if CommDlgExtendedError() == 0 {
                Ok(None)
            } else {
                Err(crate::qr::QrError::FileUnavailable)
            };
        }
    }
    let length = buffer
        .iter()
        .position(|&v| v == 0)
        .ok_or(crate::qr::QrError::FileUnavailable)?;
    let path = std::path::PathBuf::from(std::ffi::OsString::from_wide(&buffer[..length]));
    use zeroize::Zeroize;
    buffer.zeroize();
    Ok(Some(path))
}
#[cfg(not(windows))]
pub fn choose(_: &dyn Fn() -> bool) -> Result<Option<std::path::PathBuf>, crate::qr::QrError> {
    Err(crate::qr::QrError::FileUnavailable)
}
