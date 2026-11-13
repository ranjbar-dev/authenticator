//! Argument-free Task18 preview. Opens only a fresh owned public-fixture TempDir.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use authenticator::{
    domain::{
        otp::{OtpSettings, Secret},
        records::*,
    },
    storage::VaultStore,
    ui::{components as c, shell::AuthenticatorApp},
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
        egui::Panel::top("owned-task18").show(root, |ui| {
            c::secondary(ui, "DISPOSABLE TASK18 · generated public Trash fixture");
            ui.collapsing("Owned preview controls", |ui| {
                c::body(ui, &self.directory.path().to_string_lossy());
                c::secondary(
                    ui,
                    "F6: 340 · F7: 600 · F8: 960 · F10: sanitized actual frame",
                );
            });
        });
        self.app.show(root);
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
        "Authenticator Trash preview",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([600., 800.])
                .with_position([40., 40.]),
            ..Default::default()
        },
        Box::new(|cc| {
            let directory = tempfile::Builder::new()
                .prefix("authenticator-task18-")
                .tempdir()?;
            let mut store = VaultStore::open_directory(directory.path())?;
            store.create(
                Zeroizing::new("disposable-trash-preview".into()),
                public_fixture()?,
            )?;
            let mut app = AuthenticatorApp::with_repository(cc, Ok(store));
            app.state
                .repository_mut()?
                .unlock(Zeroizing::new("disposable-trash-preview".into()))?;
            app.state
                .navigate(authenticator::ui::state::Route::Settings);
            Ok(Box::new(Preview {
                app,
                directory,
                captures: 0,
            }))
        }),
    )
}
fn public_fixture() -> Result<Vault, authenticator::error::AppError> {
    use authenticator::domain::{
        folder_edit::{FolderTarget, FolderUpdate},
        recovery_code::{CodeChange, CodeTarget},
        recovery_edit::RecoveryTarget,
    };
    use std::collections::BTreeSet;
    let mut v = Vault::new(Timestamp(100));
    let folder = v.add_folder("Public work", FolderColor::Blue, Timestamp(100))?;
    v.add_folder("Public work", FolderColor::Purple, Timestamp(100))?;
    let mut first = None;
    let mut active = BTreeSet::new();
    for i in 0..24 {
        let seed = data_encoding::BASE32_NOPAD.encode(&[0x41, i as u8]);
        let account = v.add_account(
            AccountDraft::new(
                &format!("Public Trash service {i:02}"),
                None,
                Some("public@fixture.invalid"),
                Secret::from_base32(&seed)?,
                OtpSettings::default(),
            )?,
            Timestamp(100),
        )?;
        v.add_membership(folder, account, Timestamp(100))?;
        if i < 2 {
            let mut used = BackupCode::new(&format!("public-used-recovery-{i}"), Timestamp(100))?;
            used.set_used(true, Timestamp(100))?;
            let code = used.id();
            let set = v.add_backup_set(
                account,
                vec![
                    used,
                    BackupCode::new(&format!("public-available-recovery-{i}"), Timestamp(100))?,
                ],
                Timestamp(100),
            )?;
            if i == 0 {
                first = Some(account);
                let target = CodeTarget::capture(&v, account, set, code)?;
                v.change_recovery_code(&target, CodeChange::Trash, Timestamp(100))?;
                let target = RecoveryTarget::capture(&v, account)?;
                v.trash_recovery_set(&target, Timestamp(100))?;
            } else {
                active.insert(account);
            }
        } else {
            let state = v
                .accounts()
                .iter()
                .find(|a| a.id() == account)
                .ok_or(authenticator::error::AppError::InvalidRecord)?
                .state();
            v.trash_account(account, state, Timestamp(100))?;
        }
    }
    let target = FolderTarget::capture(&v, Some(folder))?;
    v.save_folder(
        &target,
        FolderUpdate {
            name: "Public work",
            color: FolderColor::Blue,
            selected: &active,
            duplicate_consent: true,
        },
        Timestamp(100),
    )?;
    let target = FolderTarget::capture(&v, Some(folder))?;
    v.trash_folder(&target, Timestamp(100))?;
    let account = first.ok_or(authenticator::error::AppError::InvalidRecord)?;
    let state = v
        .accounts()
        .iter()
        .find(|a| a.id() == account)
        .ok_or(authenticator::error::AppError::InvalidRecord)?
        .state();
    v.trash_account(account, state, Timestamp(100))?;
    v.validate()?;
    Ok(v)
}
#[test]
fn owned_trash_fixture_has_all_five_recoverable_kinds_without_native_windows()
-> Result<(), authenticator::error::AppError> {
    use authenticator::domain::trash::{ItemId, items};
    let v = public_fixture()?;
    let items = items(&v);
    for kind in 0..5 {
        assert!(items.iter().any(|item| matches!(
            (kind, item.id),
            (0, ItemId::Account(_))
                | (1, ItemId::Folder(_))
                | (2, ItemId::Membership(_))
                | (3, ItemId::Set(_))
                | (4, ItemId::Code(_, _))
        )));
    }
    v.validate()
}
