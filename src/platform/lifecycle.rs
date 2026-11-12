//! Passive current-session/power notifications. Never locks or suspends the OS.
use crate::error::AppError;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

struct Signal {
    pending: AtomicBool,
    healthy: AtomicBool,
    repaint: Box<dyn Fn() + Send + Sync>,
}
impl Signal {
    fn lock(&self) {
        self.pending.store(true, Ordering::SeqCst);
        (self.repaint)();
    }
}

pub struct Lifecycle {
    signal: Arc<Signal>,
    #[cfg(windows)]
    hwnd: usize,
    #[cfg(windows)]
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Lifecycle {
    pub fn start(repaint: impl Fn() + Send + Sync + 'static) -> Result<Self, AppError> {
        let signal = Arc::new(Signal {
            pending: AtomicBool::new(false),
            healthy: AtomicBool::new(true),
            repaint: Box::new(repaint),
        });
        #[cfg(windows)]
        {
            let (tx, rx) = std::sync::mpsc::sync_channel(1);
            let copy = signal.clone();
            let thread = std::thread::spawn(move || native::run(copy, tx));
            match rx.recv() {
                Ok(Ok(hwnd)) => Ok(Self {
                    signal,
                    hwnd,
                    thread: Some(thread),
                }),
                _ => {
                    let _ = thread.join();
                    Err(AppError::StorageUnavailable)
                }
            }
        }
        #[cfg(not(windows))]
        {
            let _ = signal;
            Err(AppError::StorageUnavailable)
        }
    }
    pub fn take_lock(&self) -> bool {
        self.signal.pending.swap(false, Ordering::SeqCst)
    }
    pub fn healthy(&self) -> bool {
        self.signal.healthy.load(Ordering::SeqCst)
    }
}
#[cfg(windows)]
impl Drop for Lifecycle {
    fn drop(&mut self) {
        // SAFETY: Owned native window; closing it releases registrations on its thread.
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                self.hwnd as _,
                windows_sys::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                0,
                0,
            );
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(windows)]
mod native {
    use super::*;
    use windows_sys::Win32::{
        Foundation::*,
        System::{LibraryLoader::GetModuleHandleW, Power::*, RemoteDesktop::*},
        UI::WindowsAndMessaging::*,
    };
    unsafe extern "system" fn procedure(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // SAFETY: CREATESTRUCT carries a stable Arc-owned Signal pointer. run
        // keeps that Arc alive until after DestroyWindow and message-loop exit.
        unsafe {
            if message == WM_NCCREATE {
                let create = &*(lparam as *const CREATESTRUCTW);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
            }
            let signal = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Signal;
            if !signal.is_null()
                && ((message == WM_WTSSESSION_CHANGE && wparam == WTS_SESSION_LOCK as usize)
                    || (message == WM_POWERBROADCAST && matches!(wparam, 4 | 7 | 18)))
            {
                (&*signal).lock();
            }
            match message {
                WM_CLOSE => {
                    DestroyWindow(hwnd);
                    0
                }
                WM_DESTROY => {
                    if !signal.is_null() {
                        (&*signal).healthy.store(false, Ordering::SeqCst);
                        (&*signal).lock();
                    }
                    PostQuitMessage(0);
                    0
                }
                _ => DefWindowProcW(hwnd, message, wparam, lparam),
            }
        }
    }
    pub(super) fn run(
        signal: Arc<Signal>,
        ready: std::sync::mpsc::SyncSender<Result<usize, AppError>>,
    ) {
        // SAFETY: Each registration is paired on this same owning thread. This
        // is an unshown top-level HWND, NOT HWND_MESSAGE (no power broadcasts).
        unsafe {
            let instance = GetModuleHandleW(std::ptr::null());
            let name: Vec<u16> = format!("AuthenticatorLifecycle{}", uuid::Uuid::new_v4())
                .encode_utf16()
                .chain(Some(0))
                .collect();
            let class = WNDCLASSW {
                lpfnWndProc: Some(procedure),
                hInstance: instance,
                lpszClassName: name.as_ptr(),
                ..Default::default()
            };
            if RegisterClassW(&class) == 0 {
                let _ = ready.send(Err(AppError::StorageUnavailable));
                return;
            }
            let hwnd = CreateWindowExW(
                0,
                name.as_ptr(),
                name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                Arc::as_ptr(&signal).cast(),
            );
            if hwnd.is_null() {
                UnregisterClassW(name.as_ptr(), instance);
                let _ = ready.send(Err(AppError::StorageUnavailable));
                return;
            }
            let wts = WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) != 0;
            let power = RegisterSuspendResumeNotification(hwnd, DEVICE_NOTIFY_WINDOW_HANDLE);
            if !wts || power == 0 {
                if wts {
                    WTSUnRegisterSessionNotification(hwnd);
                }
                if power != 0 {
                    UnregisterSuspendResumeNotification(power);
                }
                DestroyWindow(hwnd);
                UnregisterClassW(name.as_ptr(), instance);
                let _ = ready.send(Err(AppError::StorageUnavailable));
                return;
            }
            if ready.send(Ok(hwnd as usize)).is_ok() {
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            WTSUnRegisterSessionNotification(hwnd);
            UnregisterSuspendResumeNotification(power);
            if IsWindow(hwnd) != 0 {
                DestroyWindow(hwnd);
            }
            UnregisterClassW(name.as_ptr(), instance);
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    #[test]
    fn native_window_routes_lock_suspend_resume_and_cleans_up() -> Result<(), AppError> {
        let listener = Lifecycle::start(|| {})?;
        let hwnd = listener.hwnd;
        assert!(!listener.take_lock());
        // SAFETY: Sends only to this test's private notification HWND. Does not
        // broadcast, invoke LockWorkStation, or suspend the user's desktop.
        unsafe {
            SendMessageW(hwnd as _, WM_WTSSESSION_CHANGE, 8, 0);
            assert!(!listener.take_lock());
            for (message, code) in [
                (WM_WTSSESSION_CHANGE, 7),
                (WM_POWERBROADCAST, 4),
                (WM_POWERBROADCAST, 7),
                (WM_POWERBROADCAST, 18),
            ] {
                SendMessageW(hwnd as _, message, code, 0);
                assert!(listener.take_lock());
                assert!(!listener.take_lock());
            }
            drop(listener);
            assert_eq!(IsWindow(hwnd as _), 0);
        }
        Ok(())
    }
}
