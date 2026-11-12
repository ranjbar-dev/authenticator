//! Argument-free isolated Task10 UI fixture. Never opens default/reference data.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use authenticator::{
    domain::{
        otp::{OtpSettings, Secret},
        records::{AccountDraft, BackupCode, FolderColor, Timestamp, Vault},
    },
    error::AppError,
    storage::VaultStore,
    ui::{components as c, shell::AuthenticatorApp, state::AppState},
};
use eframe::egui;
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;
struct Preview {
    app: AuthenticatorApp,
    directory: Option<tempfile::TempDir>,
    path: PathBuf,
    retained: bool,
    ciphertext: Option<Vec<u8>>,
    #[cfg(debug_assertions)]
    captures: u32,
}
fn pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-codes-preview".into())
}
fn locator() -> PathBuf {
    std::env::temp_dir().join("authenticator-task10-relaunch.locator")
}
fn resume_path() -> Result<Option<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let locator = locator();
    if !locator.try_exists()? {
        return Ok(None);
    }
    if std::fs::metadata(&locator)?.len() > 4096 {
        return Err("unknown test locator".into());
    }
    let path = PathBuf::from(std::fs::read_to_string(locator)?);
    let parent = std::env::temp_dir().canonicalize()?;
    let canonical = path.canonicalize()?;
    let name = canonical
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("invalid fixture")?;
    if !path.is_absolute()
        || canonical.parent() != Some(parent.as_path())
        || !name.starts_with("authenticator-task10-")
    {
        return Err("unowned fixture refused".into());
    }
    let expected = format!("Authenticator Task10 disposable relaunch fixture\n{name}");
    if std::fs::read_to_string(canonical.join("task10-owned.marker"))? != expected {
        return Err("unowned fixture refused".into());
    }
    Ok(Some(canonical))
}
fn retain(path: &Path) -> std::io::Result<()> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or(std::io::ErrorKind::InvalidInput)?;
    let mut marker = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.join("task10-owned.marker"))?;
    marker.write_all(
        format!("Authenticator Task10 disposable relaunch fixture\n{name}").as_bytes(),
    )?;
    marker.sync_all()?;
    let mut locator = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(locator())?;
    locator.write_all(
        path.to_str()
            .ok_or(std::io::ErrorKind::InvalidInput)?
            .as_bytes(),
    )?;
    locator.sync_all()
}
impl eframe::App for Preview {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.logic(ctx, frame);
    }
    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.app.on_exit(gl);
    }
    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
        if root.ctx().input(|i| i.key_pressed(egui::Key::F4)) && !self.app.state.busy() {
            let path = self.path.join("vault.bin");
            if let Some(bytes) = self.ciphertext.take() {
                let _ = std::fs::write(path, bytes);
            } else if let Ok(bytes) = std::fs::read(&path) {
                self.ciphertext = Some(bytes);
                let _ = std::fs::write(path, b"disposable-account-conflict");
            }
        }
        if root.ctx().input(|i| i.key_pressed(egui::Key::F5))
            && !self.app.state.busy()
            && self.ciphertext.is_none()
        {
            self.app.lock();
            // Drop the actual owner/instance lock before reopening the SAME
            // encrypted fixture. No fake success state or shadow record cache.
            self.app.state = AppState::new(Err(AppError::StorageUnavailable));
            self.app.state = AppState::new(VaultStore::open_directory(&self.path));
            if let Ok(store) = self.app.state.repository_mut() {
                let _ = store.unlock(pass());
            }
        }
        if root.ctx().input(|i| i.key_pressed(egui::Key::F9))
            && !self.app.state.busy()
            && self.ciphertext.is_none()
            && !self.retained
            && retain(&self.path).is_ok()
        {
            if let Some(directory) = self.directory.take() {
                let _ = directory.keep();
            }
            self.retained = true;
        }
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
        egui::Panel::top("disposable-notice").show(root, |ui| {
            c::secondary(ui, "DISPOSABLE TASK10 · isolated encrypted fixture");
            ui.collapsing("Preview controls / owned path", |ui| {
                c::secondary(ui, &format!("Owned fixture: {}", self.path.display()));
                c::secondary(
                    ui,
                    if self.ciphertext.is_some() {
                        "F4: restore own ciphertext · conflict armed"
                    } else {
                        "F4: fault own ciphertext · F5: reopen actual repository"
                    },
                );
                c::secondary(
                    ui,
                    if self.retained {
                        "F9: retained for normal process relaunch · starts locked"
                    } else {
                        "F9: retain this owned fixture for a process relaunch"
                    },
                );
                c::secondary(ui, "F6: 340 · F7: 600 · F8: 960 (logical window widths)");
            });
        });
        self.app.show(root);
        #[cfg(debug_assertions)]
        if root.ctx().input(|input| input.key_pressed(egui::Key::F11)) {
            let (unlocked, listener_ok, listener_healthy, owner_nonzero, queued) =
                self.app.copy_diagnostics_for_evidence();
            let queued = match queued {
                None => "null",
                Some(true) => "true",
                Some(false) => "false",
            };
            let value = format!(
                "{{\"unlocked\":{unlocked},\"listener_ok\":{listener_ok},\"listener_healthy\":{listener_healthy},\"owner_nonzero\":{owner_nonzero},\"last_copy_queued\":{queued}}}"
            );
            let _ = std::fs::write(self.path.join("copy-diagnostic.json"), value);
        }
        #[cfg(debug_assertions)]
        if root.ctx().input(|input| input.key_pressed(egui::Key::F10)) && self.captures < 50 {
            self.captures += 1;
            let path = self
                .path
                .join(format!("sanitized-frame-{}.png", self.captures));
            let bounds = self.app.code_bounds_for_evidence();
            let scale = root.ctx().pixels_per_point();
            root.ctx().request_screenshot(move |frame| {
                // Actual native framebuffer, not a reconstruction or test drawing.
                // Redact exact rendered code-label rectangles BEFORE encoding/writing.
                std::thread::spawn(move || {
                    let width = frame.size[0];
                    let height = frame.size[1];
                    let mut pixels = Vec::with_capacity(width * height * 4);
                    for (index, pixel) in frame.pixels.iter().enumerate() {
                        let point = egui::pos2(
                            (index % width) as f32 / scale,
                            (index / width) as f32 / scale,
                        );
                        if bounds.iter().any(|bound| bound.expand(4.).contains(point)) {
                            pixels.extend_from_slice(&[21, 28, 39, 255]);
                        } else {
                            pixels.extend_from_slice(&pixel.to_array());
                        }
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
fn main() -> eframe::Result {
    std::panic::set_hook(Box::new(|_| {}));
    eframe::run_native(
        "Authenticator codes preview",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Authenticator codes preview")
                .with_inner_size([600., 680.])
                .with_position([40., 40.]),
            ..Default::default()
        },
        Box::new(|cc| {
            let resumed = resume_path()?;
            let directory = if resumed.is_none() {
                Some(
                    tempfile::Builder::new()
                        .prefix("authenticator-task10-")
                        .tempdir()?,
                )
            } else {
                None
            };
            let path = resumed
                .clone()
                .or_else(|| directory.as_ref().map(|d| d.path().to_owned()))
                .ok_or("fixture unavailable")?;
            let mut vault = Vault::new(Timestamp(1));
            let account = vault.add_account(
                AccountDraft::new(
                    "Public codes fixture",
                    Some("https://example.invalid"),
                    Some("disposable"),
                    Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")?,
                    OtpSettings::default(),
                )?,
                Timestamp(2),
            )?;
            for (index, (service, identifier)) in [
                ("École", Some("Üser")),
                ("Alpha", Some("zebra")),
                ("Zulu", Some("alpha")),
                ("Alpha", None),
                ("Public mail", Some("disposable")),
            ]
            .into_iter()
            .enumerate()
            {
                vault.add_account(
                    AccountDraft::new(
                        service,
                        None,
                        identifier,
                        Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")?,
                        OtpSettings::default(),
                    )?,
                    Timestamp(2 + index as u64),
                )?;
            }
            let folder =
                vault.add_folder("Public fixture folder", FolderColor::Blue, Timestamp(10))?;
            vault.add_membership(folder, account, Timestamp(11))?;
            let mut used = BackupCode::new("public-used-fixture", Timestamp(12))?;
            used.set_used(true, Timestamp(12))?;
            vault.add_backup_set(
                account,
                vec![
                    used,
                    BackupCode::new("public-unused-fixture", Timestamp(12))?,
                ],
                Timestamp(12),
            )?;
            let mut store = VaultStore::open_directory(&path)?;
            if resumed.is_none() {
                store.create(pass(), vault)?;
            }
            let mut app = AuthenticatorApp::with_repository(cc, Ok(store));
            if resumed.is_none() {
                app.state.repository_mut()?.unlock(pass())?;
            }
            Ok(Box::new(Preview {
                app,
                directory,
                path,
                retained: resumed.is_some(),
                ciphertext: None,
                #[cfg(debug_assertions)]
                captures: 0,
            }))
        }),
    )
}
