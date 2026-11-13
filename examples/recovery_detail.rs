//! Argument-free owned Task16 preview. Never opens the default/user vault.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use authenticator::{
    domain::{
        otp::{OtpSettings, Secret},
        records::{AccountDraft, BackupCode, Timestamp, Vault},
    },
    storage::VaultStore,
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
        egui::Panel::top("owned-task16").show(root, |ui| {
            c::secondary(ui, "DISPOSABLE TASK16 · generated public recovery fixture");
            ui.collapsing("Owned preview controls", |ui| {
                c::body(ui, &self.directory.path().to_string_lossy());
                c::secondary(
                    ui,
                    "F6: 340 · F7: 600 · F8: 960 · F10: sanitized actual frame",
                );
            });
        });
        self.app.show(root);
        // Release builds do not expose debug redaction bounds; never capture
        // an unredacted frame in that configuration.
        if cfg!(debug_assertions)
            && root.ctx().input(|i| i.key_pressed(egui::Key::F10))
            && self.captures < 30
        {
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
        "Authenticator recovery detail preview",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([600., 800.])
                .with_position([40., 40.]),
            ..Default::default()
        },
        Box::new(|cc| {
            let directory = tempfile::Builder::new()
                .prefix("authenticator-task16-")
                .tempdir()?;
            let vault = public_fixture()?;
            let mut store = VaultStore::open_directory(directory.path())?;
            store.create(Zeroizing::new("disposable-detail-preview".into()), vault)?;
            let mut app = AuthenticatorApp::with_repository(cc, Ok(store));
            app.state
                .repository_mut()?
                .unlock(Zeroizing::new("disposable-detail-preview".into()))?;
            app.state.navigate(Route::BackupCodes);
            Ok(Box::new(Preview {
                app,
                directory,
                captures: 0,
            }))
        }),
    )
}
fn public_fixture() -> Result<Vault, authenticator::error::AppError> {
    let mut vault = Vault::new(Timestamp(100));
    let id = vault.add_account(
        AccountDraft::new(
            "Public recovery detail",
            None,
            Some("public@fixture.invalid"),
            Secret::from_base32("MY")?,
            OtpSettings::default(),
        )?,
        Timestamp(100),
    )?;
    let codes = (0..21)
        .map(|i| {
            let value = if i == 0 {
                "PUBLIC-LONG-".repeat(341)
            } else {
                format!("public-detail-{i:02}")
            };
            let mut code = BackupCode::new(&value, Timestamp(100))?;
            if i == 1 {
                code.set_used(true, Timestamp(100))?;
            }
            Ok(code)
        })
        .collect::<Result<Vec<_>, authenticator::error::AppError>>()?;
    vault.add_backup_set(id, codes, Timestamp(100))?;
    vault.validate()?;
    Ok(vault)
}
#[test]
fn owned_preview_fixture_validates_without_native_windows()
-> Result<(), authenticator::error::AppError> {
    public_fixture()?.validate()
}
