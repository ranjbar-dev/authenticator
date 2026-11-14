//! Argument-free final QA fixture. Never opens the default vault or writes raw captures.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use authenticator::{
    backup::{self, Destination},
    domain::{
        otp::{OtpSettings, Secret},
        records::{AccountDraft, BackupCode, FolderColor, Timestamp, Vault},
    },
    storage::{OperationCancellation, VaultStore},
    ui::{components as c, shell::AuthenticatorApp},
};
use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::path::PathBuf;
use zeroize::Zeroizing;

struct Preview {
    app: AuthenticatorApp,
    root: PathBuf,
    frames: u32,
    hwnd: usize,
}
impl eframe::App for Preview {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.logic(ctx, frame);
    }
    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.app.on_exit(gl);
    }
    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        for (key, zoom) in [
            (egui::Key::F3, 1.),
            (egui::Key::F4, 1.5),
            (egui::Key::F5, 2.),
        ] {
            if ctx.input(|i| i.key_pressed(key)) {
                ctx.set_zoom_factor(zoom);
            }
        }
        // Native window movement through visible QA hotkeys; production UI is unchanged.
        if ctx.input(|i| i.key_pressed(egui::Key::F9))
            && let Ok(displays) = authenticator::platform::capture::display_bounds()
            && let Some(left) = displays.iter().min_by_key(|m| m.left)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                (left.left + 80) as f32 / ctx.pixels_per_point(),
                (left.top + 80) as f32 / ctx.pixels_per_point(),
            )));
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                40. / ctx.pixels_per_point(),
                40. / ctx.pixels_per_point(),
            )));
        }
        for (key, size) in [
            (egui::Key::F6, [340., 780.]),
            (egui::Key::F7, [600., 800.]),
            (egui::Key::F8, [960., 900.]),
        ] {
            if ctx.input(|i| i.key_pressed(key)) {
                // Keep the whole fixture on this desktop when app zoom scales
                // logical dimensions. This is app zoom, never a physical DPI test.
                let fit = egui::vec2(size[0], size[1].min(1000. / ctx.zoom_factor()));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(fit));
            }
        }
        egui::Panel::top("owned-task24").show(root, |ui| {
            c::secondary(
                ui,
                "DISPOSABLE QA · F3/F4/F5 zoom · F6/F7/F8 size · F10 safe frame",
            );
        });
        self.app.show(root);
        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            // Rectangle integers only: never pixels, QR payloads or credentials.
            let region = self.app.state.capture_region().map_or_else(
                || "null".to_string(),
                |r| {
                    format!(
                        "{{\"left\":{},\"top\":{},\"width\":{},\"height\":{}}}",
                        r.left, r.top, r.width, r.height
                    )
                },
            );
            let _ = std::fs::write(self.root.join("capture-bounds.json"), region);
        }
        if cfg!(debug_assertions)
            && ctx.input(|i| i.key_pressed(egui::Key::F10))
            && self.frames < 240
        {
            self.frames += 1;
            let path = self
                .root
                .join(format!("sanitized-frame-{:03}.png", self.frames));
            #[cfg(debug_assertions)]
            let bounds = self.app.code_bounds_for_evidence();
            #[cfg(not(debug_assertions))]
            let bounds: Vec<egui::Rect> = Vec::new();
            let scale = ctx.pixels_per_point();
            #[cfg(windows)]
            let dpi = unsafe { windows_sys::Win32::UI::HiDpi::GetDpiForWindow(self.hwnd as _) };
            #[cfg(not(windows))]
            let dpi = 0;
            let metadata = format!(
                "{{\"pixels_per_point\":{},\"app_zoom\":{},\"physical_dpi\":{},\"sanitized\":true}}",
                scale,
                ctx.zoom_factor(),
                dpi
            );
            let _ = std::fs::write(path.with_extension("json"), &metadata);
            ctx.request_screenshot(move |frame| {
                std::thread::spawn(move || {
                    let [width, height] = frame.size;
                    let mut pixels = Vec::with_capacity(width * height * 4);
                    for (index, pixel) in frame.pixels.iter().enumerate() {
                        let point = egui::pos2(
                            (index % width) as f32 / scale,
                            (index / width) as f32 / scale,
                        );
                        let rgba = if bounds.iter().any(|b| b.expand(4.).contains(point)) {
                            [21, 28, 39, 255]
                        } else {
                            pixel.to_array()
                        };
                        pixels.extend_from_slice(&rgba);
                    }
                    if image::save_buffer_with_format(
                        &path,
                        &pixels,
                        width as u32,
                        height as u32,
                        image::ColorType::Rgba8,
                        image::ImageFormat::Png,
                    )
                    .is_ok()
                    {
                        let _ = std::fs::write(path.with_extension("ready"), b"sanitized");
                    }
                });
            });
        }
    }
}
fn main() -> eframe::Result {
    std::panic::set_hook(Box::new(|_| {}));
    eframe::run_native(
        "Authenticator final matrix QA",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Authenticator final matrix QA")
                .with_inner_size([600., 800.])
                .with_position([40., 40.]),
            ..Default::default()
        },
        Box::new(|cc| {
            let hwnd = cc
                .window_handle()
                .ok()
                .and_then(|h| match h.as_raw() {
                    RawWindowHandle::Win32(w) => Some(w.hwnd.get() as usize),
                    _ => None,
                })
                .unwrap_or(0);
            let target = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
            let target = target.canonicalize()?;
            let directory = tempfile::Builder::new()
                .prefix("task24-owned-")
                .tempdir_in(&target)?;
            let root = directory.path().canonicalize()?;
            if !root.starts_with(&target) {
                return Err("QA root escaped workspace".into());
            }
            // Preserve this uniquely owned fixture/evidence for review; no recursive cleanup.
            let _ = directory.keep();
            std::fs::write(
                root.join("ownership.txt"),
                format!(
                    "pid={}\nexecutable={}\nroot={}\n",
                    std::process::id(),
                    std::env::current_exe()?.display(),
                    root.display()
                ),
            )?;
            let mut vault = Vault::new(Timestamp(1));
            let mut first = None;
            for index in 0..7 {
                let at = Timestamp(2 + index);
                let id = vault.add_account(
                    AccountDraft::new(
                        if index == 0 {
                            "Public Matrix Mail"
                        } else if index == 6 {
                            "Public account with a long service name for narrow layout verification"
                        } else {
                            "Public Matrix Account"
                        },
                        Some("https://example.invalid"),
                        Some(&format!("disposable-{index}@public.invalid")),
                        Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")?,
                        OtpSettings::default(),
                    )?,
                    at,
                )?;
                if index == 0 {
                    first = Some(id);
                    let mut used = BackupCode::new("public-used-matrix-fixture", at)?;
                    used.set_used(true, at)?;
                    vault.add_backup_set(
                        id,
                        vec![used, BackupCode::new("public-unused-matrix-fixture", at)?],
                        at,
                    )?;
                }
                if index == 5 {
                    vault.trash_account(id, vault.accounts()[index as usize].state(), at)?;
                }
            }
            let folder =
                vault.add_folder("Public Matrix Folder", FolderColor::Purple, Timestamp(10))?;
            if let Some(id) = first {
                vault.add_membership(folder, id, Timestamp(11))?;
            }
            // An encrypted, independent-key source fixture for native import
            // review/pagination and large-vault journeys. No plaintext file or
            // QR pixels are written. This example never resolves default data.
            let mut incoming = vault.clone();
            for index in 0..250 {
                incoming.add_account(
                    AccountDraft::new(
                        &format!("Public large fixture {index:03}"),
                        Some("https://example.invalid"),
                        Some("disposable-large@public.invalid"),
                        Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")?,
                        OtpSettings::default(),
                    )?,
                    Timestamp(20 + index),
                )?;
            }
            let mut source = VaultStore::open_directory(root.join("import-source"))?;
            source.create(Zeroizing::new("disposablematrixsource".into()), incoming)?;
            // The production backup adapter deliberately accepts ordinary drive
            // paths only, while Windows canonicalize adds a verbatim prefix.
            let backup_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join(root.file_name().ok_or("QA root has no name")?);
            if backup_directory.canonicalize()? != root {
                return Err("QA backup directory identity changed".into());
            }
            backup::export(
                &source,
                Destination::create_new(&backup_directory.join("public-import.authpbk"))?,
                Zeroizing::new("disposablematrixbackup".into()),
                &OperationCancellation::default(),
            )?;
            drop(source);
            let mut store = VaultStore::open_directory(root.join("vault"))?;
            store.create(Zeroizing::new("disposablematrixpassphrase".into()), vault)?;
            let mut app = AuthenticatorApp::with_repository(cc, Ok(store));
            app.state
                .repository_mut()?
                .unlock(Zeroizing::new("disposablematrixpassphrase".into()))?;
            Ok(Box::new(Preview {
                app,
                root,
                frames: 0,
                hwnd,
            }))
        }),
    )
}
