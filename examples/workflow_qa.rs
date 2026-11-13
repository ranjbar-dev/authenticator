//! Argument-free disposable QA window. Never opens the default user vault.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use authenticator::{
    storage::VaultStore,
    ui::{components as c, shell::AuthenticatorApp},
};
use eframe::egui;
use std::path::PathBuf;

struct Preview {
    app: AuthenticatorApp,
    directory: tempfile::TempDir,
    frames: u32,
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
        let ctx = root.ctx().clone();
        egui::Panel::top("owned-task21").show(root, |ui| {
            c::secondary(ui, "DISPOSABLE TASK21 · isolated clean vault");
            ui.collapsing("Owned QA controls", |ui| {
                c::secondary(ui, "F6/F7/F8: window sizes · F10: sanitized frame");
                for scale in [1., 1.5, 2.] {
                    if c::button(
                        ui,
                        &format!("App zoom {}%", (scale * 100.) as u32),
                        c::ButtonStyle::Secondary,
                    )
                    .clicked()
                    {
                        ctx.set_zoom_factor(scale);
                    }
                }
            });
        });
        self.app.show(root);
        if cfg!(debug_assertions)
            && root.ctx().input(|i| i.key_pressed(egui::Key::F10))
            && self.frames < 180
        {
            self.frames += 1;
            let path = self
                .directory
                .path()
                .join(format!("sanitized-frame-{:03}.png", self.frames));
            #[cfg(debug_assertions)]
            let bounds = self.app.code_bounds_for_evidence();
            #[cfg(not(debug_assertions))]
            let bounds: Vec<egui::Rect> = Vec::new();
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
                        if bounds.iter().any(|b| b.expand(4.).contains(point)) {
                            pixels.extend_from_slice(&[21, 28, 39, 255]);
                        } else {
                            pixels.extend_from_slice(&pixel.to_array());
                        }
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
        "Authenticator disposable full QA",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Authenticator disposable full QA")
                .with_inner_size([600., 800.])
                .with_position([40., 40.]),
            ..Default::default()
        },
        Box::new(|cc| {
            let target = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
            let directory = tempfile::Builder::new()
                .prefix("task21-owned-")
                .tempdir_in(target)?;
            let manifest = format!(
                "pid={}\nexecutable={}\nroot={}\n",
                std::process::id(),
                std::env::current_exe()?.display(),
                directory.path().display()
            );
            std::fs::write(directory.path().join("ownership.txt"), manifest)?;
            let store = VaultStore::open_directory(directory.path().join("vault"))?;
            Ok(Box::new(Preview {
                app: AuthenticatorApp::with_repository(cc, Ok(store)),
                directory,
                frames: 0,
            }))
        }),
    )
}
