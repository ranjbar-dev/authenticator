//! Disposable native-input clipboard probe; never opens the user's vault.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#[cfg(all(windows, debug_assertions))]
mod probe {
    use authenticator::{
        domain::{
            otp::{OtpSettings, Secret, current_code},
            records::{AccountDraft, Timestamp, Vault},
        },
        storage::VaultStore,
        ui::{codes::FrameClock, components as c, shell::AuthenticatorApp},
    };
    use eframe::egui;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::{
        io::Write,
        path::PathBuf,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    use windows_sys::Win32::{
        Foundation::{GlobalFree, HWND},
        System::{DataExchange::*, Memory::*},
        UI::WindowsAndMessaging::*,
    };
    use zeroize::Zeroizing;

    const UNICODE: u32 = 13;
    const FOLLOW_UP: &str = "Authenticator disposable subsequent copy";

    struct Open;
    impl Drop for Open {
        fn drop(&mut self) {
            unsafe {
                CloseClipboard();
            }
        }
    }
    fn open(owner: usize) -> Result<Open, &'static str> {
        if unsafe { OpenClipboard(owner as HWND) } == 0 {
            Err("busy")
        } else {
            Ok(Open)
        }
    }
    // Never expose or allocate arbitrary clipboard text. Caller must establish
    // this owned fixture's HWND and expected format under the same exclusive open.
    fn matches(expected: &str) -> bool {
        let expected = Zeroizing::new(expected.encode_utf16().chain(Some(0)).collect::<Vec<_>>());
        let handle = unsafe { GetClipboardData(UNICODE) };
        if handle.is_null() {
            return false;
        }
        let length = expected.len() * 2;
        let bytes = unsafe { GlobalSize(handle) };
        if bytes < length || bytes > length + 64 {
            return false;
        }
        let pointer = unsafe { GlobalLock(handle) };
        if pointer.is_null() {
            return false;
        }
        let equal = unsafe { std::slice::from_raw_parts(pointer.cast::<u16>(), expected.len()) }
            == expected.as_slice();
        unsafe {
            GlobalUnlock(handle);
        }
        equal
    }
    struct Probe {
        app: AuthenticatorApp,
        root: PathBuf,
        owner: usize,
        other: usize,
        lease_format: u32,
        last_fixture: Option<(u32, u64)>,
        previous: Option<String>,
        captures: u32,
        observe_after: Instant,
    }
    impl Probe {
        fn record(&mut self, stage: &str) {
            if let Some(proof) = authenticator::platform::clipboard::take_copy_proof_for_evidence()
                && let Ok(mut file) = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(self.root.join("native-proof.txt"))
            {
                let _ = writeln!(file, "{proof:?}");
            }
            let unix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |v| v.as_secs());
            let Ok(_open) = open(0) else {
                return;
            };
            let owner = unsafe { GetClipboardOwner() } as usize;
            let sequence = unsafe { GetClipboardSequenceNumber() };
            let text = unsafe { IsClipboardFormatAvailable(UNICODE) } != 0;
            let lease = unsafe { IsClipboardFormatAvailable(self.lease_format) } != 0;
            let (unlocked, _, _, _, queued) = self.app.copy_diagnostics_for_evidence();
            let mut fixture_match = false;
            if owner == self.owner
                && text
                && lease
                && (queued == Some(true)
                    || self.last_fixture.is_some_and(|(seq, _)| seq == sequence))
                && let Ok(secret) = Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")
            {
                // Copy may have occurred just before a boundary. No values
                // or hashes enter the report, including on mismatch.
                for at in [
                    unix,
                    unix.saturating_sub(30),
                    self.last_fixture.map_or(unix, |(_, at)| at),
                ] {
                    if let Ok(code) =
                        current_code(&secret, OtpSettings::default(), &FrameClock(Ok(at)))
                        && matches(code.expose_value())
                    {
                        fixture_match = true;
                        self.last_fixture = Some((sequence, at));
                    }
                }
            }
            let follow_up = owner == self.other && text && matches(FOLLOW_UP);
            drop(_open);
            let state = format!(
                "\"unlocked\":{unlocked},\"sequence\":{sequence},\"has_unicode\":{text},\"owner_is_app\":{},\"lease_present\":{lease},\"matches_fixture_code\":{fixture_match},\"subsequent_copy_intact\":{follow_up}",
                owner == self.owner
            );
            if stage != "poll" || self.previous.as_ref() != Some(&state) {
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(self.root.join("observations.jsonl"))
                {
                    let _ = writeln!(file, "{{\"stage\":\"{stage}\",\"unix\":{unix},{state}}}");
                }
                self.previous = Some(state);
            }
        }
        fn subsequent_copy(&self) -> Result<(), &'static str> {
            // A separately owned hidden fixture HWND represents a later writer.
            // No production actor API or ownership test is modified.
            let encoded: Vec<u16> = FOLLOW_UP.encode_utf16().chain(Some(0)).collect();
            let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, encoded.len() * 2) };
            if handle.is_null() {
                return Err("allocation");
            }
            let pointer = unsafe { GlobalLock(handle) };
            if pointer.is_null() {
                unsafe {
                    GlobalFree(handle);
                }
                return Err("lock");
            }
            unsafe {
                std::ptr::copy_nonoverlapping(encoded.as_ptr(), pointer.cast(), encoded.len());
                GlobalUnlock(handle);
            }
            let opened = open(self.other);
            if opened.is_err() {
                unsafe {
                    GlobalFree(handle);
                }
                return Err("busy");
            }
            let _open = opened?;
            if unsafe { EmptyClipboard() } == 0
                || unsafe { SetClipboardData(UNICODE, handle) }.is_null()
            {
                unsafe {
                    GlobalFree(handle);
                }
                return Err("publish");
            }
            Ok(())
        }
    }
    impl eframe::App for Probe {
        fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            self.app.logic(ctx, frame);
        }
        fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
            self.app.on_exit(gl);
            self.record("after_exit_cleanup");
            unsafe {
                DestroyWindow(self.other as HWND);
            }
        }
        fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
            // Do not grab the OS clipboard in the same input frame in which
            // the real actor is dispatched. The observer must not create its
            // own predictable Busy failures in the behavior being tested.
            if root.ctx().input(|i| !i.events.is_empty()) {
                self.observe_after = Instant::now() + Duration::from_millis(250);
            }
            egui::Panel::top("owned-clipboard-probe").show(root, |ui| {
                c::secondary(ui, "DISPOSABLE CLIPBOARD · real app/30-second expiry");
                c::secondary(
                    ui,
                    "F3: harmless later writer · F12: observation · F10: sanitized frame",
                );
            });
            for (key, size) in [
                (egui::Key::F6, [340., 780.]),
                (egui::Key::F7, [600., 800.]),
                (egui::Key::F8, [960., 900.]),
            ] {
                if root.ctx().input(|i| i.key_pressed(key)) {
                    root.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::InnerSize(size.into()));
                }
            }
            self.app.show(root);
            if root.ctx().input(|i| i.key_pressed(egui::Key::F3)) {
                let success = self.subsequent_copy().is_ok();
                let _ = std::fs::write(
                    self.root.join("subsequent-write-result.txt"),
                    if success { "true" } else { "false" },
                );
                self.record("subsequent_writer");
            }
            if root.ctx().input(|i| i.key_pressed(egui::Key::F12)) {
                self.record("manual");
            }
            if Instant::now() >= self.observe_after {
                self.record("poll");
            }
            root.ctx().request_repaint_after(Duration::from_millis(100));
            if root.ctx().input(|i| i.key_pressed(egui::Key::F10)) && self.captures < 20 {
                self.captures += 1;
                let path = self.root.join(format!("sanitized-{}.png", self.captures));
                let bounds = self.app.code_bounds_for_evidence();
                let scale = root.ctx().pixels_per_point();
                root.ctx().request_screenshot(move |frame| {
                    std::thread::spawn(move || {
                        let [width, height] = frame.size;
                        let mut pixels = Vec::with_capacity(width * height * 4);
                        for (index, pixel) in frame.pixels.iter().enumerate() {
                            let point = egui::pos2(
                                (index % width) as f32 / scale,
                                (index / width) as f32 / scale,
                            );
                            pixels.extend_from_slice(&if bounds
                                .iter()
                                .any(|r| r.expand(4.).contains(point))
                            {
                                [21, 28, 39, 255]
                            } else {
                                pixel.to_array()
                            });
                        }
                        let _ = image::save_buffer_with_format(
                            path,
                            &pixels,
                            width as u32,
                            height as u32,
                            image::ColorType::Rgba8,
                            image::ImageFormat::Png,
                        );
                    });
                });
            }
        }
    }
    pub fn run() -> eframe::Result {
        std::panic::set_hook(Box::new(|_| {}));
        eframe::run_native(
            "Authenticator disposable clipboard QA",
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([600., 800.])
                    .with_position([40., 40.]),
                ..Default::default()
            },
            Box::new(|cc| {
                let root = tempfile::Builder::new()
                    .prefix("task10-interactive-")
                    .tempdir_in(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"))?
                    .keep();
                let owner = cc
                    .window_handle()
                    .ok()
                    .and_then(|h| match h.as_raw() {
                        RawWindowHandle::Win32(w) => Some(w.hwnd.get() as usize),
                        _ => None,
                    })
                    .ok_or("missing HWND")?;
                let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
                let other = unsafe {
                    CreateWindowExW(
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
                    )
                } as usize;
                if other == 0 {
                    return Err("missing subsequent-writer HWND".into());
                }
                let name: Vec<u16> = "Authenticator.LocalClipboardLease.v1\0"
                    .encode_utf16()
                    .collect();
                let lease_format = unsafe { RegisterClipboardFormatW(name.as_ptr()) };
                if lease_format == 0 {
                    return Err("clipboard format".into());
                }
                std::fs::write(
                    root.join("ownership.txt"),
                    format!(
                        "pid={}\nexecutable={}\nroot={}\nowner={owner}\nother={other}\n",
                        std::process::id(),
                        std::env::current_exe()?.display(),
                        root.display()
                    ),
                )?;
                let mut vault = Vault::new(Timestamp(1));
                vault.add_account(
                    AccountDraft::new(
                        "Public clipboard fixture",
                        None,
                        Some("disposable"),
                        Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")?,
                        OtpSettings::default(),
                    )?,
                    Timestamp(2),
                )?;
                let pass = || Zeroizing::new("disposable-clipboard-probe".into());
                let mut store = VaultStore::open_directory(root.join("vault"))?;
                store.create(pass(), vault)?;
                let mut app = AuthenticatorApp::with_repository(cc, Ok(store));
                app.state.repository_mut()?.unlock(pass())?;
                Ok(Box::new(Probe {
                    app,
                    root,
                    owner,
                    other,
                    lease_format,
                    last_fixture: None,
                    previous: None,
                    captures: 0,
                    observe_after: Instant::now() + Duration::from_millis(250),
                }))
            }),
        )
    }
}
fn main() -> eframe::Result {
    #[cfg(all(windows, debug_assertions))]
    {
        probe::run()
    }
    #[cfg(not(all(windows, debug_assertions)))]
    {
        Ok(())
    }
}
