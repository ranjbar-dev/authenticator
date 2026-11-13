//! Argument-free Task13 preview; fresh independently keyed source/destination.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use authenticator::{
    backup::{self, Destination},
    domain::{
        otp::{OtpSettings, Secret},
        records::{AccountDraft, BackupCode, FolderColor, Timestamp, Vault},
    },
    storage::{OperationCancellation, VaultStore},
    ui::{components as c, shell::AuthenticatorApp, state::Route},
};
use eframe::egui;
use zeroize::Zeroizing;
struct Preview {
    app: AuthenticatorApp,
    directory: tempfile::TempDir,
    captures: u32,
}
impl eframe::App for Preview {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.logic(ctx, frame);
    }
    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.app.on_exit(gl);
    }
    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
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
        egui::Panel::top("disposable-export-notice").show(root, |ui| {
            c::secondary(
                ui,
                "DISPOSABLE TASK13 · fresh local vault / public backup fixture",
            );
            ui.collapsing("Owned destination / preview controls", |ui| {
                c::body(
                    ui,
                    &self
                        .directory
                        .path()
                        .join("public-import.authpbk")
                        .to_string_lossy(),
                );
                c::secondary(
                    ui,
                    "F6: 340 · F7: 600 · F8: 960 · F10: sanitized framebuffer",
                );
            });
        });
        self.app.show(root);
        if root.ctx().input(|i| i.key_pressed(egui::Key::F10)) && self.captures < 30 {
            self.captures += 1;
            let path = self
                .directory
                .path()
                .join(format!("sanitized-frame-{}.png", self.captures));
            #[cfg(debug_assertions)]
            let bounds = self.app.code_bounds_for_evidence();
            #[cfg(not(debug_assertions))]
            let bounds: Vec<egui::Rect> = Vec::new();
            let scale = root.ctx().pixels_per_point();
            root.ctx().request_screenshot(move |frame| {
                std::thread::spawn(move || {
                    let width = frame.size[0];
                    let height = frame.size[1];
                    let mut pixels = Vec::with_capacity(width * height * 4);
                    for (index, pixel) in frame.pixels.iter().enumerate() {
                        let point = egui::pos2(
                            (index % width) as f32 / scale,
                            (index / width) as f32 / scale,
                        );
                        if bounds.iter().any(|b| b.expand(4.).contains(point)) {
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
        "Authenticator import preview",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Authenticator import preview")
                .with_inner_size([600., 800.])
                .with_position([40., 40.]),
            ..Default::default()
        },
        Box::new(|cc| {
            let directory = tempfile::Builder::new()
                .prefix("authenticator-task13-")
                .tempdir()?;
            let mut vault = Vault::new(Timestamp(1));
            let account = vault.add_account(
                AccountDraft::new(
                    "Public import fixture",
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
            let mut used = BackupCode::new("public-used-export-fixture", Timestamp(5))?;
            used.set_used(true, Timestamp(6))?;
            vault.add_backup_set(
                account,
                vec![
                    used,
                    BackupCode::new("public-unused-export-fixture", Timestamp(6))?,
                ],
                Timestamp(7),
            )?;
            vault.trash_account(account, vault.accounts()[0].state(), Timestamp(8))?;
            let source_path = directory.path().join("source");
            std::fs::create_dir(&source_path)?;
            let mut source = VaultStore::open_directory(&source_path)?;
            source.create(Zeroizing::new("disposable-source-only".into()), vault)?;
            backup::export(
                &source,
                Destination::create_new(&directory.path().join("public-import.authpbk"))?,
                Zeroizing::new("disposablebackupfixture".into()),
                &OperationCancellation::default(),
            )?;
            drop(source);
            let local_path = directory.path().join("destination");
            std::fs::create_dir(&local_path)?;
            let mut store = VaultStore::open_directory(&local_path)?;
            store.create(
                Zeroizing::new("disposableimportpreview".into()),
                Vault::new(Timestamp(1)),
            )?;
            let mut app = AuthenticatorApp::with_repository(cc, Ok(store));
            app.state
                .repository_mut()?
                .unlock(Zeroizing::new("disposableimportpreview".into()))?;
            app.state.navigate(Route::Settings);
            Ok(Box::new(Preview {
                app,
                directory,
                captures: 0,
            }))
        }),
    )
}
