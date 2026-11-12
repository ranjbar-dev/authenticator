//! App-owned clipboard leases. The actor expires even when ROOT is hidden.
//! No clipboard text is read until sequence, window owner and random marker match.
use super::lifecycle::LifecycleWatch;
use crate::domain::clock::{Clock, SystemClock};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipboardError {
    Busy,
    Unavailable,
    Cancelled,
    Expired,
    Changed,
}
#[derive(Clone, Copy)]
pub struct CodeWindow {
    pub counter: u64,
    pub period: u64,
}
impl CodeWindow {
    fn valid(self, clock: &dyn Clock) -> bool {
        self.period != 0
            && clock
                .unix_seconds()
                .is_ok_and(|now| now / self.period == self.counter)
    }
}
trait Backend: Send + 'static {
    type Lease: Send;
    fn copy(
        &mut self,
        owner: usize,
        text: &str,
        admitted: &dyn Fn() -> bool,
    ) -> Result<Self::Lease, ClipboardError>;
    fn clear(&mut self, lease: &Self::Lease) -> Result<(), ClipboardError>;
}
struct Native;
impl Backend for Native {
    type Lease = native::Lease;
    fn copy(
        &mut self,
        owner: usize,
        text: &str,
        admitted: &dyn Fn() -> bool,
    ) -> Result<Self::Lease, ClipboardError> {
        native::copy(owner, text, admitted)
    }
    fn clear(&mut self, lease: &Self::Lease) -> Result<(), ClipboardError> {
        native::clear(lease)
    }
}
enum Command {
    Copy {
        text: Zeroizing<String>,
        lifetime: Duration,
        generation: u64,
        watch: LifecycleWatch,
        window: Option<CodeWindow>,
    },
    Clear,
    Shutdown,
}
pub struct Clipboard {
    sender: mpsc::SyncSender<Command>,
    results: mpsc::Receiver<(u64, Result<(), ClipboardError>)>,
    generation: Arc<AtomicU64>,
    done: mpsc::Receiver<()>,
    thread: Option<std::thread::JoinHandle<()>>,
    stopping: Arc<AtomicBool>,
    settled: Arc<AtomicU64>,
}
impl Clipboard {
    pub fn new(owner: usize, repaint: impl Fn() + Send + 'static) -> Self {
        Self::with_backend(owner, Native, repaint)
    }
    fn with_backend<B: Backend>(
        owner: usize,
        backend: B,
        repaint: impl Fn() + Send + 'static,
    ) -> Self {
        Self::with_backend_clock(owner, backend, Box::new(SystemClock), repaint)
    }
    fn with_backend_clock<B: Backend>(
        owner: usize,
        mut backend: B,
        clock: Box<dyn Clock + Send>,
        repaint: impl Fn() + Send + 'static,
    ) -> Self {
        let (sender, receiver) = mpsc::sync_channel(8);
        let (result_sender, results) = mpsc::channel();
        let generation = Arc::new(AtomicU64::new(0));
        let current = generation.clone();
        let settled = Arc::new(AtomicU64::new(0));
        let actor_settled = settled.clone();
        let stopping_signal = Arc::new(AtomicBool::new(false));
        let stop = stopping_signal.clone();
        let (done_sender, done) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            let mut lease: Option<(B::Lease, Instant, u64, LifecycleWatch)> = None;
            let mut stopping = false;
            let mut stop_deadline = None;
            let mut clear_deadline = None;
            let mut clear_warned = false;
            loop {
                match receiver.recv_timeout(Duration::from_millis(50)) {
                    Ok(Command::Copy {
                        text,
                        lifetime,
                        generation,
                        watch,
                        window,
                    }) => {
                        if current.load(Ordering::SeqCst) == generation && !watch.changed() {
                            let session =
                                || current.load(Ordering::SeqCst) == generation && !watch.changed();
                            let valid = || window.is_none_or(|window| window.valid(clock.as_ref()));
                            let admitted = || session() && valid();
                            let result = backend.copy(owner, &text, &admitted);
                            match result {
                                Ok(value) => {
                                    if admitted() {
                                        let _ = result_sender.send((generation, Ok(())));
                                    } else if session() && !valid() {
                                        let _ = result_sender
                                            .send((generation, Err(ClipboardError::Expired)));
                                    }
                                    let deadline = if admitted() {
                                        Instant::now() + lifetime
                                    } else {
                                        Instant::now()
                                    };
                                    lease = Some((value, deadline, generation, watch));
                                    clear_deadline = None;
                                    clear_warned = false;
                                }
                                Err(ClipboardError::Cancelled) => {
                                    if session() && !valid() {
                                        let _ = result_sender
                                            .send((generation, Err(ClipboardError::Expired)));
                                    }
                                }
                                Err(error) => {
                                    if admitted() {
                                        let _ = result_sender.send((generation, Err(error)));
                                    }
                                }
                            }
                            repaint();
                        }
                    }
                    Ok(Command::Clear) => {}
                    Ok(Command::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        stopping = true;
                        stop_deadline.get_or_insert(Instant::now() + Duration::from_secs(2));
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                if stop.load(Ordering::SeqCst) {
                    stopping = true;
                    stop_deadline.get_or_insert(Instant::now() + Duration::from_secs(2));
                }
                if let Some((value, deadline, generation, watch)) = &lease
                    && (stopping
                        || Instant::now() >= *deadline
                        || current.load(Ordering::SeqCst) != *generation
                        || watch.changed())
                {
                    let current_generation = *generation;
                    let limit =
                        *clear_deadline.get_or_insert(Instant::now() + Duration::from_secs(2));
                    match backend.clear(value) {
                        Ok(()) => {
                            lease = None;
                            clear_deadline = None;
                            clear_warned = false;
                        }
                        Err(ClipboardError::Busy) if Instant::now() < limit => {}
                        Err(ClipboardError::Busy) => {
                            // Lease retains only a salted digest, not plaintext.
                            // Retry after a long-lived external holder releases it.
                            if !clear_warned {
                                clear_warned = true;
                                let _ = result_sender.send((
                                    current.load(Ordering::SeqCst),
                                    Err(ClipboardError::Unavailable),
                                ));
                                repaint();
                            }
                        }
                        Err(_) => {
                            lease = None;
                            clear_deadline = None;
                            let _ = result_sender
                                .send((current_generation, Err(ClipboardError::Unavailable)));
                            repaint();
                        }
                    }
                }
                if lease.is_none() {
                    actor_settled.store(current.load(Ordering::SeqCst), Ordering::SeqCst);
                }
                if stopping
                    && (lease.is_none() || stop_deadline.is_some_and(|d| Instant::now() >= d))
                {
                    break;
                }
            }
            let _ = done_sender.send(());
        });
        Self {
            sender,
            results,
            generation,
            done,
            thread: Some(thread),
            stopping: stopping_signal,
            settled,
        }
    }
    pub fn copy(&self, text: Zeroizing<String>, seconds: u32, watch: LifecycleWatch) -> bool {
        self.enqueue(text, seconds, watch, None)
    }
    pub fn copy_code(
        &self,
        text: Zeroizing<String>,
        seconds: u32,
        watch: LifecycleWatch,
        window: CodeWindow,
    ) -> bool {
        self.enqueue(text, seconds, watch, Some(window))
    }
    fn enqueue(
        &self,
        text: Zeroizing<String>,
        seconds: u32,
        watch: LifecycleWatch,
        window: Option<CodeWindow>,
    ) -> bool {
        if self.stopping.load(Ordering::SeqCst) || text.len() > 1024 * 1024 || text.contains('\0') {
            return false;
        }
        self.sender
            .try_send(Command::Copy {
                text,
                lifetime: Duration::from_secs(seconds.into()),
                generation: self.generation.load(Ordering::SeqCst),
                watch,
                window,
            })
            .is_ok()
    }
    pub fn clear(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        let _ = self.sender.try_send(Command::Clear);
    }
    /// Used by graceful close while Root continues pumping owner messages.
    pub fn cleanup_pending(&self) -> bool {
        self.settled.load(Ordering::SeqCst) != self.generation.load(Ordering::SeqCst)
    }
    pub fn poll(&self) -> Option<Result<(), ClipboardError>> {
        while let Ok((generation, result)) = self.results.try_recv() {
            if generation == self.generation.load(Ordering::SeqCst) {
                return Some(result);
            }
        }
        None
    }
    /// Best effort exit: bounded wait, never wait indefinitely on an OS clipboard
    /// owner. An OS call already admitted cannot be forcibly revoked.
    pub fn shutdown(&mut self) {
        if self.thread.is_none() {
            return;
        }
        self.clear();
        self.stopping.store(true, Ordering::SeqCst);
        let _ = self.sender.try_send(Command::Shutdown);
        if self.done.recv_timeout(Duration::from_millis(300)).is_ok() {
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        } else {
            self.thread.take();
        }
    }
}
impl Drop for Clipboard {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(windows)]
mod native {
    use super::*;
    use sha2::{Digest, Sha256};
    use windows_sys::Win32::{
        Foundation::{GlobalFree, HANDLE, HWND},
        System::{DataExchange::*, Memory::*},
    };
    const UNICODE: u32 = 13;
    pub(super) struct Lease {
        owner: usize,
        sequence: u32,
        marker: [u8; 16],
        format: u32,
        expected_digest: [u8; 32],
        expected_bytes: usize,
    }
    struct Open;
    impl Open {
        fn acquire(owner: usize) -> Result<Self, ClipboardError> {
            // The OS call does not wait for a different clipboard holder.
            if unsafe { OpenClipboard(owner as HWND) } == 0 {
                Err(ClipboardError::Busy)
            } else {
                Ok(Self)
            }
        }
    }
    impl Drop for Open {
        fn drop(&mut self) {
            unsafe {
                CloseClipboard();
            }
        }
    }
    struct Global(HANDLE);
    impl Global {
        fn new(bytes: &[u8]) -> Result<Self, ClipboardError> {
            let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, bytes.len()) };
            if handle.is_null() {
                return Err(ClipboardError::Unavailable);
            }
            let value = Self(handle);
            let ptr = unsafe { GlobalLock(handle) };
            if ptr.is_null() {
                return Err(ClipboardError::Unavailable);
            }
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.cast(), bytes.len());
                GlobalUnlock(handle);
            }
            Ok(value)
        }
        fn publish(mut self, format: u32) -> Result<(), ClipboardError> {
            if unsafe { SetClipboardData(format, self.0) }.is_null() {
                return Err(ClipboardError::Unavailable);
            }
            self.0 = std::ptr::null_mut(); // ownership transferred only on success
            Ok(())
        }
    }
    impl Drop for Global {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    let size = GlobalSize(self.0);
                    let pointer = GlobalLock(self.0);
                    if !pointer.is_null() {
                        if size <= 2 * (1024 * 1024 + 1) + 64 {
                            std::ptr::write_bytes(pointer.cast::<u8>(), 0, size);
                        }
                        GlobalUnlock(self.0);
                    }
                    GlobalFree(self.0);
                }
            }
        }
    }
    fn register(name: &str) -> Result<u32, ClipboardError> {
        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        match unsafe { RegisterClipboardFormatW(wide.as_ptr()) } {
            0 => Err(ClipboardError::Unavailable),
            value => Ok(value),
        }
    }
    fn bytes(values: &[u16]) -> &[u8] {
        unsafe { std::slice::from_raw_parts(values.as_ptr().cast(), values.len() * 2) }
    }
    pub(super) fn copy(
        owner: usize,
        text: &str,
        admitted: &dyn Fn() -> bool,
    ) -> Result<Lease, ClipboardError> {
        copy_after_close(owner, text, admitted, || {})
    }
    fn copy_after_close(
        owner: usize,
        text: &str,
        admitted: &dyn Fn() -> bool,
        after_close: impl FnOnce(),
    ) -> Result<Lease, ClipboardError> {
        if owner == 0 || text.len() > 1024 * 1024 || text.contains('\0') {
            return Err(ClipboardError::Unavailable);
        }
        let expected = Zeroizing::new(text.encode_utf16().chain(Some(0)).collect::<Vec<_>>());
        let mut marker = [0; 16];
        getrandom::fill(&mut marker).map_err(|_| ClipboardError::Unavailable)?;
        let format = register("Authenticator.LocalClipboardLease.v1")?;
        let history = register("CanIncludeInClipboardHistory")?;
        let cloud = register("CanUploadToCloudClipboard")?;
        // Allocate before opening/emptying: allocation failure preserves old data.
        let data = Global::new(bytes(&expected))?;
        let token = Global::new(&marker)?;
        let history_data = Global::new(&0u32.to_le_bytes())?;
        let cloud_data = Global::new(&0u32.to_le_bytes())?;
        let _open = Open::acquire(owner)?;
        if !admitted() {
            return Err(ClipboardError::Cancelled);
        }
        if unsafe { EmptyClipboard() } == 0 {
            return Err(ClipboardError::Unavailable);
        }
        // Publish hints+ownership before text. Failure cannot leave an unleased OTP.
        token.publish(format)?;
        history_data.publish(history)?;
        cloud_data.publish(cloud)?;
        if !admitted() {
            return Err(ClipboardError::Cancelled);
        }
        data.publish(UNICODE)?;
        let written_sequence = unsafe { GetClipboardSequenceNumber() };
        if written_sequence == 0 {
            unsafe {
                EmptyClipboard();
            }
            return Err(ClipboardError::Unavailable);
        }
        // Native tests observe a sequence change at CloseClipboard. Do not
        // assume its cause or adopt a later external writer's sequence.
        drop(_open);
        after_close();
        let sequence = unsafe { GetClipboardSequenceNumber() };
        if sequence == 0 {
            return Err(ClipboardError::Unavailable);
        }
        let expected_digest = digest(&marker, bytes(&expected));
        let mut lease = Lease {
            owner,
            sequence,
            marker,
            format,
            expected_digest,
            expected_bytes: expected.len() * 2,
        };
        match Open::acquire(0) {
            Ok(_open) => {
                if !owned_content(&lease)? {
                    return Err(ClipboardError::Changed);
                }
                lease.sequence = unsafe { GetClipboardSequenceNumber() };
            }
            // Keep provisional evidence for cleanup when the clipboard becomes
            // available. Never rebase this sequence later: an intervening write
            // invalidates it even if text is identical. Every clear still needs
            // exact HWND+nonce+digest proof in one exclusive open interval.
            Err(ClipboardError::Busy) => {}
            Err(error) => return Err(error),
        }
        Ok(lease)
    }
    fn digest(marker: &[u8; 16], bytes: &[u8]) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(marker);
        hash.update(bytes);
        hash.finalize().into()
    }
    fn matches(format: u32, expected: &[u8]) -> bool {
        let handle = unsafe { GetClipboardData(format) };
        if handle.is_null() {
            return false;
        }
        // Exact expected-size bounds before dereferencing; no unrelated data allocation.
        let size = unsafe { GlobalSize(handle) };
        if size < expected.len() || size > expected.len().saturating_add(64) {
            return false;
        }
        let ptr = unsafe { GlobalLock(handle) };
        if ptr.is_null() {
            return false;
        }
        let equal =
            unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), expected.len()) } == expected;
        unsafe {
            GlobalUnlock(handle);
        }
        equal
    }
    #[cfg(test)]
    #[test]
    #[ignore = "entry point used only by isolated_native_clipboard_process"]
    fn isolated_native_helper() -> Result<(), Box<dyn std::error::Error>> {
        use windows_sys::Win32::{
            System::{StationsAndDesktops::*, Threading::GetCurrentThreadId},
            UI::WindowsAndMessaging::*,
        };
        if std::env::var_os("AUTHENTICATOR_ISOLATED_CLIPBOARD_HELPER").is_none() {
            return Err("isolated helper only".into());
        }
        // Separate PROCESS + CREATE_ONLY random noninteractive station. Never
        // open/read/write WinSta0 or switch the active input desktop.
        unsafe {
            let original_station = GetProcessWindowStation();
            let original_desktop = GetThreadDesktop(GetCurrentThreadId());
            let name = format!("AuthenticatorClipboardTest{}", uuid::Uuid::new_v4())
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();
            let station = CreateWindowStationW(name.as_ptr(), 1, 0x000f037f, std::ptr::null());
            if station.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            struct Restore {
                station: *mut std::ffi::c_void,
                original_station: *mut std::ffi::c_void,
                desktop: *mut std::ffi::c_void,
                original_desktop: *mut std::ffi::c_void,
            }
            impl Drop for Restore {
                fn drop(&mut self) {
                    unsafe {
                        SetThreadDesktop(self.original_desktop);
                        SetProcessWindowStation(self.original_station);
                        if !self.desktop.is_null() {
                            CloseDesktop(self.desktop);
                        }
                        CloseWindowStation(self.station);
                    }
                }
            }
            let mut restore = Restore {
                station,
                original_station,
                desktop: std::ptr::null_mut(),
                original_desktop,
            };
            if SetProcessWindowStation(station) == 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            let mut observed = [0u16; 256];
            let mut needed = 0;
            assert_ne!(
                GetUserObjectInformationW(
                    station,
                    UOI_NAME,
                    observed.as_mut_ptr().cast(),
                    512,
                    &mut needed
                ),
                0
            );
            let end = observed
                .iter()
                .position(|v| *v == 0)
                .ok_or("station name bound")?;
            assert_eq!(&observed[..end], &name[..name.len() - 1]);
            let desktop_name = "Fixture\0".encode_utf16().collect::<Vec<_>>();
            restore.desktop = CreateDesktopW(
                desktop_name.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                DESKTOP_CREATEWINDOW | DESKTOP_READOBJECTS | DESKTOP_WRITEOBJECTS,
                std::ptr::null(),
            );
            if restore.desktop.is_null() || SetThreadDesktop(restore.desktop) == 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            let class = "STATIC\0".encode_utf16().collect::<Vec<_>>();
            let window = CreateWindowExW(
                0,
                class.as_ptr(),
                class.as_ptr(),
                0,
                0,
                0,
                1,
                1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            if window.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            struct Window(HWND);
            impl Drop for Window {
                fn drop(&mut self) {
                    unsafe {
                        DestroyWindow(self.0);
                    }
                }
            }
            let window = Window(window);
            let owner = window.0 as usize;
            let second = CreateWindowExW(
                0,
                class.as_ptr(),
                class.as_ptr(),
                0,
                0,
                0,
                1,
                1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            if second.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            let second = Window(second);
            let other = second.0 as usize;
            let first = copy(owner, "123456", &|| true).map_err(|_| "native copy failed")?;
            assert!(first.sequence > 0);
            assert_eq!(GetClipboardSequenceNumber(), first.sequence);
            {
                let _open = Open::acquire(0).map_err(|_| "open")?;
                assert!(matches(
                    UNICODE,
                    &[b'1', 0, b'2', 0, b'3', 0, b'4', 0, b'5', 0, b'6', 0, 0, 0]
                ));
            }
            clear(&first).map_err(|_| "native clear failed")?;
            {
                let _open = Open::acquire(0).map_err(|_| "open")?;
                assert_eq!(IsClipboardFormatAvailable(UNICODE), 0);
            }
            let old = copy(owner, "123456", &|| true).map_err(|_| "copy")?;
            let newer = copy(owner, "123456", &|| true).map_err(|_| "copy")?;
            assert_ne!(old.sequence, newer.sequence);
            clear(&old).map_err(|_| "clear")?;
            {
                let _open = Open::acquire(0).map_err(|_| "open")?;
                assert_eq!(GetClipboardSequenceNumber(), newer.sequence);
            }
            // Corrupt the ownership marker while holding the OS clipboard; the
            // text/sequence alone is never enough authority to clear.
            let mut old = copy(owner, "123456", &|| true).map_err(|_| "copy")?;
            {
                let _open = Open::acquire(0).map_err(|_| "open")?;
                Global::new(&[9; 16])
                    .map_err(|_| "alloc")?
                    .publish(old.format)
                    .map_err(|_| "publish")?;
            }
            old.sequence = GetClipboardSequenceNumber(); // force marker-specific path
            clear(&old).map_err(|_| "clear")?;
            {
                let _open = Open::acquire(0).map_err(|_| "open")?;
                assert_ne!(IsClipboardFormatAvailable(UNICODE), 0);
            }
            let mut old = copy(owner, "123456", &|| true).map_err(|_| "copy")?;
            {
                let _open = Open::acquire(owner).map_err(|_| "open")?;
                Global::new(&[b'7', 0, 0, 0])
                    .map_err(|_| "alloc")?
                    .publish(UNICODE)
                    .map_err(|_| "publish")?;
            }
            old.sequence = GetClipboardSequenceNumber(); // force bounded digest path
            clear(&old).map_err(|_| "clear")?;
            {
                let _open = Open::acquire(0).map_err(|_| "open")?;
                assert_ne!(IsClipboardFormatAvailable(UNICODE), 0);
            }
            for external_text in ["123456", "654321"] {
                let mut replacement = None;
                let result = copy_after_close(owner, "123456", &|| true, || {
                    replacement = copy(other, external_text, &|| true).ok();
                });
                assert_eq!(result.err(), Some(ClipboardError::Changed));
                let replacement = replacement.ok_or("external race fixture copy failed")?;
                let _open = Open::acquire(0).map_err(|_| "open")?;
                assert_eq!(GetClipboardSequenceNumber(), replacement.sequence);
                assert_eq!(GetClipboardOwner() as usize, other);
            }
            let own = copy(owner, "123456", &|| true).map_err(|_| "copy")?;
            let (entered, seen) = mpsc::sync_channel(1);
            let (release, go) = mpsc::channel();
            let desktop = restore.desktop as usize;
            let holding = std::thread::spawn(move || {
                SetThreadDesktop(desktop as _);
                let class = "STATIC\0".encode_utf16().collect::<Vec<_>>();
                let hwnd = CreateWindowExW(
                    0,
                    class.as_ptr(),
                    class.as_ptr(),
                    0,
                    0,
                    0,
                    1,
                    1,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null(),
                );
                let open = Open::acquire(hwnd as usize);
                let _ = entered.send(open.is_ok());
                let _ = go.recv_timeout(Duration::from_secs(2));
                drop(open);
                DestroyWindow(hwnd);
            });
            assert!(seen.recv_timeout(Duration::from_secs(2))?);
            assert_eq!(clear(&own), Err(ClipboardError::Busy));
            let _ = release.send(());
            holding.join().map_err(|_| "holder panicked")?;
            // Destroyed owner must not make a later external copy eligible.
            // Windows clears owner data on destruction; either absent or our
            // exact old ownership proof is safe, and NULL opening remains valid.
            drop(window);
            clear(&own).map_err(|_| "destroyed-owner cleanup")?;
            let external = copy(other, "123456", &|| true).map_err(|_| "external copy")?;
            clear(&own).map_err(|_| "destroyed-owner newer copy")?;
            {
                let _open = Open::acquire(0).map_err(|_| "open")?;
                assert_eq!(GetClipboardSequenceNumber(), external.sequence);
                assert_eq!(GetClipboardOwner() as usize, other);
            }
            let before = GetClipboardSequenceNumber();
            assert_eq!(
                copy(other, "123456", &|| false).err(),
                Some(ClipboardError::Cancelled)
            );
            assert_eq!(GetClipboardSequenceNumber(), before);
            let calls = std::cell::Cell::new(0);
            assert_eq!(
                copy(other, "123456", &|| {
                    let call = calls.get();
                    calls.set(call + 1);
                    call == 0
                })
                .err(),
                Some(ClipboardError::Cancelled)
            );
            {
                let _open = Open::acquire(0).map_err(|_| "open")?;
                assert_eq!(IsClipboardFormatAvailable(UNICODE), 0);
            }
            // Keep the real owner HWND pumping during actor cleanup, as the
            // normal-close nonsensitive frame does in the actual application.
            native_actor_owner_pump(other)?;
        }
        Ok(())
    }
    #[cfg(test)]
    fn native_actor_owner_pump(other: usize) -> Result<(), Box<dyn std::error::Error>> {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        let mut actor = Clipboard::new(other, || {});
        assert!(actor.copy(
            Zeroizing::new("123456".into()),
            30,
            LifecycleWatch::healthy_for_test()
        ));
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut result = None;
        while result.is_none() && Instant::now() < deadline {
            let mut message = MSG::default();
            unsafe {
                while PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            result = actor.poll();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(result, Some(Ok(())));
        actor.clear();
        while actor.cleanup_pending() && Instant::now() < deadline {
            let mut message = MSG::default();
            unsafe {
                while PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(!actor.cleanup_pending());
        {
            let _open = Open::acquire(0).map_err(|_| "open")?;
            assert_eq!(unsafe { IsClipboardFormatAvailable(UNICODE) }, 0);
        }
        actor.shutdown();
        Ok(())
    }
    pub(super) fn clear(lease: &Lease) -> Result<(), ClipboardError> {
        // NULL is valid for a check/clear interval and survives Root destruction.
        let _open = Open::acquire(0)?;
        if unsafe { GetClipboardSequenceNumber() } != lease.sequence
            || unsafe { GetClipboardOwner() } as usize != lease.owner
        {
            return Ok(());
        }
        if !owned_content(lease)? {
            return Ok(());
        }
        // Read/check/clear share this exclusive OpenClipboard interval: external
        // writers cannot race between the ownership check and EmptyClipboard.
        if unsafe { EmptyClipboard() } == 0 {
            Err(ClipboardError::Unavailable)
        } else {
            Ok(())
        }
    }
    fn owned_content(lease: &Lease) -> Result<bool, ClipboardError> {
        if unsafe { GetClipboardOwner() } as usize != lease.owner
            || !matches(lease.format, &lease.marker)
        {
            return Ok(false);
        }
        let handle = unsafe { GetClipboardData(UNICODE) };
        if handle.is_null() {
            return Ok(false);
        }
        let size = unsafe { GlobalSize(handle) };
        if size < lease.expected_bytes
            || size > lease.expected_bytes.saturating_add(64)
            || lease.expected_bytes > 2 * (1024 * 1024 + 1)
        {
            return Ok(false);
        }
        let ptr = unsafe { GlobalLock(handle) };
        if ptr.is_null() {
            return Err(ClipboardError::Unavailable);
        }
        let actual = digest(&lease.marker, unsafe {
            std::slice::from_raw_parts(ptr.cast::<u8>(), lease.expected_bytes)
        });
        unsafe {
            GlobalUnlock(handle);
        }
        Ok(actual == lease.expected_digest)
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::sync::{Mutex, atomic::AtomicBool};
    #[derive(Default)]
    struct FakeState {
        sequence: u64,
        owned: bool,
        value: String,
        clears: usize,
    }
    struct Fake {
        state: Arc<Mutex<FakeState>>,
        busy: Arc<AtomicBool>,
        gate: Option<(mpsc::SyncSender<()>, mpsc::Receiver<()>, bool)>,
    }
    impl Backend for Fake {
        type Lease = u64;
        fn copy(
            &mut self,
            _: usize,
            text: &str,
            admitted: &dyn Fn() -> bool,
        ) -> Result<u64, ClipboardError> {
            let gate = self.gate.take();
            if let Some((entered, release, false)) = &gate {
                let _ = entered.send(());
                let _ = release.recv_timeout(Duration::from_secs(5));
            }
            if !admitted() {
                return Err(ClipboardError::Cancelled);
            }
            if let Some((entered, release, true)) = &gate {
                let _ = entered.send(());
                let _ = release.recv_timeout(Duration::from_secs(5));
            }
            let mut state = self.state.lock().map_err(|_| ClipboardError::Unavailable)?;
            state.sequence += 1;
            state.owned = true;
            state.value = text.into();
            Ok(state.sequence)
        }
        fn clear(&mut self, lease: &u64) -> Result<(), ClipboardError> {
            if self.busy.load(Ordering::SeqCst) {
                return Err(ClipboardError::Busy);
            }
            let mut state = self.state.lock().map_err(|_| ClipboardError::Unavailable)?;
            if state.sequence == *lease && state.owned {
                state.sequence += 1;
                state.value.clear();
                state.owned = false;
                state.clears += 1;
            }
            Ok(())
        }
    }
    fn wait(condition: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(6);
        while !condition() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            condition(),
            "clipboard actor did not reach the expected public fixture state"
        );
    }
    fn fake(
        gate: Option<(mpsc::SyncSender<()>, mpsc::Receiver<()>, bool)>,
    ) -> (Clipboard, Arc<Mutex<FakeState>>, Arc<AtomicBool>) {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let busy = Arc::new(AtomicBool::new(false));
        (
            Clipboard::with_backend(
                1,
                Fake {
                    state: state.clone(),
                    busy: busy.clone(),
                    gate,
                },
                || {},
            ),
            state,
            busy,
        )
    }
    #[test]
    fn actor_expiry_hidden_lifecycle_lock_shutdown_and_newer_identical_external_copy()
    -> Result<(), crate::error::AppError> {
        let lifecycle = super::super::lifecycle::Lifecycle::start(|| {})?;
        let (mut clipboard, state, _) = fake(None);
        clipboard.copy(Zeroizing::new("public-copy".into()), 1, lifecycle.watch());
        wait(|| state.lock().is_ok_and(|s| s.sequence == 1));
        assert_eq!(clipboard.poll(), Some(Ok(())));
        // Actor/timer makes progress without Root frames or UI poll.
        wait(|| state.lock().is_ok_and(|s| s.clears == 1));
        clipboard.copy(Zeroizing::new("public-copy".into()), 60, lifecycle.watch());
        wait(|| state.lock().is_ok_and(|s| s.sequence == 3));
        {
            let mut s = state
                .lock()
                .map_err(|_| crate::error::AppError::StorageUnavailable)?;
            s.sequence += 1;
            s.owned = false;
        }
        clipboard.clear();
        std::thread::sleep(Duration::from_millis(150));
        assert!(
            state
                .lock()
                .is_ok_and(|s| s.value == "public-copy" && s.clears == 1)
        );
        assert_eq!(
            clipboard.poll(),
            None,
            "lock suppresses previous copied success"
        );
        clipboard.copy(Zeroizing::new("public-copy".into()), 60, lifecycle.watch());
        wait(|| state.lock().is_ok_and(|s| s.owned));
        lifecycle.notify_lock_for_test();
        wait(|| state.lock().is_ok_and(|s| s.clears == 2));
        lifecycle.take_lock();
        clipboard.copy(Zeroizing::new("public-copy".into()), 60, lifecycle.watch());
        wait(|| state.lock().is_ok_and(|s| s.owned));
        clipboard.shutdown();
        assert!(state.lock().is_ok_and(|s| s.clears == 3));
        Ok(())
    }
    #[test]
    fn held_copy_admission_and_already_admitted_copy_cannot_publish_stale_success()
    -> Result<(), crate::error::AppError> {
        let lifecycle = super::super::lifecycle::Lifecycle::start(|| {})?;
        for after in [false, true] {
            let (entered, seen) = mpsc::sync_channel(1);
            let (release, go) = mpsc::channel();
            let (mut clipboard, state, _) = fake(Some((entered, go, after)));
            clipboard.copy(Zeroizing::new("public-race".into()), 60, lifecycle.watch());
            assert!(seen.recv_timeout(Duration::from_secs(2)).is_ok());
            clipboard.clear();
            let _ = release.send(());
            if after {
                wait(|| state.lock().is_ok_and(|s| s.clears == 1));
            }
            clipboard.shutdown();
            assert_eq!(clipboard.poll(), None);
            assert!(
                state
                    .lock()
                    .is_ok_and(|s| s.value.is_empty() && s.clears == usize::from(after))
            );
        }
        Ok(())
    }
    #[test]
    fn busy_cleanup_retries_after_two_seconds_without_abandoning_ownership()
    -> Result<(), crate::error::AppError> {
        let lifecycle = super::super::lifecycle::Lifecycle::start(|| {})?;
        let (mut clipboard, state, busy) = fake(None);
        clipboard.copy(Zeroizing::new("public-busy".into()), 1, lifecycle.watch());
        wait(|| state.lock().is_ok_and(|s| s.owned));
        busy.store(true, Ordering::SeqCst);
        clipboard.clear();
        std::thread::sleep(Duration::from_millis(2200));
        assert!(state.lock().is_ok_and(|s| s.owned));
        assert_eq!(clipboard.poll(), Some(Err(ClipboardError::Unavailable)));
        busy.store(false, Ordering::SeqCst);
        wait(|| state.lock().is_ok_and(|s| s.clears == 1));
        clipboard.shutdown();
        Ok(())
    }
    #[test]
    fn queued_code_window_boundary_backward_jump_error_and_full_queue_fail_closed()
    -> Result<(), crate::error::AppError> {
        struct TestClock(Arc<AtomicU64>);
        impl Clock for TestClock {
            fn unix_seconds(&self) -> Result<u64, crate::error::AppError> {
                let value = self.0.load(Ordering::SeqCst);
                if value == u64::MAX {
                    Err(crate::error::AppError::ClockUnavailable)
                } else {
                    Ok(value)
                }
            }
        }
        let lifecycle = super::super::lifecycle::Lifecycle::start(|| {})?;
        for changed in [60, 0, u64::MAX] {
            let (entered, seen) = mpsc::sync_channel(1);
            let (release, go) = mpsc::channel();
            let state = Arc::new(Mutex::new(FakeState::default()));
            let clock = Arc::new(AtomicU64::new(59));
            let mut clipboard = Clipboard::with_backend_clock(
                1,
                Fake {
                    state: state.clone(),
                    busy: Arc::new(AtomicBool::new(false)),
                    gate: Some((entered, go, false)),
                },
                Box::new(TestClock(clock.clone())),
                || {},
            );
            assert!(clipboard.copy_code(
                Zeroizing::new("123456".into()),
                30,
                lifecycle.watch(),
                CodeWindow {
                    counter: 1,
                    period: 30
                }
            ));
            assert!(seen.recv_timeout(Duration::from_secs(2)).is_ok());
            clock.store(changed, Ordering::SeqCst);
            let _ = release.send(());
            let deadline = Instant::now() + Duration::from_secs(2);
            let mut result = None;
            while result.is_none() && Instant::now() < deadline {
                result = clipboard.poll();
                std::thread::sleep(Duration::from_millis(10));
            }
            assert_eq!(result, Some(Err(ClipboardError::Expired)));
            assert!(
                state
                    .lock()
                    .is_ok_and(|s| s.sequence == 0 && s.value.is_empty())
            );
            clipboard.shutdown();
        }
        let (entered, seen) = mpsc::sync_channel(1);
        let (release, go) = mpsc::channel();
        let (mut clipboard, state, _) = fake(Some((entered, go, false)));
        clipboard.copy(
            Zeroizing::new("public-bounded".into()),
            30,
            lifecycle.watch(),
        );
        assert!(seen.recv_timeout(Duration::from_secs(2)).is_ok());
        for _ in 0..8 {
            assert!(clipboard.copy(
                Zeroizing::new("public-bounded".into()),
                30,
                lifecycle.watch()
            ));
        }
        assert!(!clipboard.copy(
            Zeroizing::new("public-bounded".into()),
            30,
            lifecycle.watch()
        ));
        clipboard.clear();
        let _ = release.send(());
        clipboard.shutdown();
        assert!(state.lock().is_ok_and(|s| s.value.is_empty()));
        Ok(())
    }
    #[test]
    fn isolated_native_clipboard_process() -> Result<(), Box<dyn std::error::Error>> {
        let mut child = std::process::Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "platform::clipboard::native::isolated_native_helper",
                "--ignored",
                "--test-threads=1",
            ])
            .env("AUTHENTICATOR_ISOLATED_CLIPBOARD_HELPER", "1")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if child.try_wait()?.is_some() {
                let output = child.wait_with_output()?;
                assert!(
                    output.status.success(),
                    "isolated native clipboard helper failed: {} {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                break;
            }
            if Instant::now() > deadline {
                child.kill()?;
                let _ = child.wait();
                return Err("isolated clipboard helper timed out".into());
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        Ok(())
    }
}
#[cfg(not(windows))]
mod native {
    use super::*;
    pub(super) struct Lease;
    pub(super) fn copy(_: usize, _: &str, _: &dyn Fn() -> bool) -> Result<Lease, ClipboardError> {
        Err(ClipboardError::Unavailable)
    }
    pub(super) fn clear(_: &Lease) -> Result<(), ClipboardError> {
        Err(ClipboardError::Unavailable)
    }
}
