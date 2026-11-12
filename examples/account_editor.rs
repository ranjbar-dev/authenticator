//! Argument-free isolated Task09 UI fixture. Never opens default/reference data.
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
}
fn pass() -> Zeroizing<String> {
    Zeroizing::new("disposable-account-preview".into())
}
fn locator() -> PathBuf {
    std::env::temp_dir().join("authenticator-task09-relaunch.locator")
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
        || !name.starts_with("authenticator-task09-")
    {
        return Err("unowned fixture refused".into());
    }
    let expected = format!("Authenticator Task09 disposable relaunch fixture\n{name}");
    if std::fs::read_to_string(canonical.join("task09-owned.marker"))? != expected {
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
        .open(path.join("task09-owned.marker"))?;
    marker.write_all(
        format!("Authenticator Task09 disposable relaunch fixture\n{name}").as_bytes(),
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
            c::secondary(ui, "DISPOSABLE TASK09 · isolated encrypted fixture");
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
        self.app.show(root);
    }
}
fn main() -> eframe::Result {
    std::panic::set_hook(Box::new(|_| {}));
    eframe::run_native(
        "Authenticator account preview",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Authenticator account preview")
                .with_inner_size([600., 800.])
                .with_position([900., 60.]),
            ..Default::default()
        },
        Box::new(|cc| {
            let resumed = resume_path()?;
            let directory = if resumed.is_none() {
                Some(
                    tempfile::Builder::new()
                        .prefix("authenticator-task09-")
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
                    "Public edit fixture",
                    Some("https://example.invalid"),
                    Some("disposable"),
                    Secret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ")?,
                    OtpSettings::default(),
                )?,
                Timestamp(2),
            )?;
            let folder =
                vault.add_folder("Public fixture folder", FolderColor::Blue, Timestamp(3))?;
            vault.add_membership(folder, account, Timestamp(4))?;
            let mut used = BackupCode::new("public-used-fixture", Timestamp(5))?;
            used.set_used(true, Timestamp(5))?;
            vault.add_backup_set(
                account,
                vec![
                    used,
                    BackupCode::new("public-unused-fixture", Timestamp(5))?,
                ],
                Timestamp(5),
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
            }))
        }),
    )
}
