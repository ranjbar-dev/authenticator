//! Isolated task05 UI evidence harness. Never opens the default user vault.
//! Its explicit test directory survives this executable's relaunch; ciphertext
//! is never silently overwritten or removed. No pre-created business records.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use authenticator::{
    storage::VaultStore,
    ui::{components as c, shell::AuthenticatorApp, theme},
};
use eframe::egui;
struct Preview {
    app: AuthenticatorApp,
}
impl eframe::App for Preview {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        egui::Panel::top("isolated-preview-tools").show(root, |ui| {
            c::secondary(ui, "DISPOSABLE VAULT · task 05 preview");
            ui.menu_button(theme::LABEL.text("Preview tools"), |ui| {
                for (label, size) in [
                    ("Narrow 340 × 740", [340., 740.]),
                    ("Intermediate 600 × 800", [600., 800.]),
                    ("Desktop 960 × 900", [960., 900.]),
                ] {
                    if ui.button(label).clicked() {
                        // Keep the requested window size stable across app zoom.
                        // This is not a physical monitor/DPI simulation.
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                            egui::vec2(size[0], size[1]) / ctx.zoom_factor(),
                        ));
                        ui.close();
                    }
                }
                for (label, zoom) in [
                    ("100% app zoom", 1.),
                    ("150% app zoom", 1.5),
                    ("200% app zoom", 2.),
                ] {
                    if ui.button(label).clicked() {
                        ctx.set_zoom_factor(zoom);
                        ui.close();
                    }
                }
                if ui
                    .add_enabled(
                        self.app.state.is_unlocked(),
                        egui::Button::new("Short idle: 3 seconds"),
                    )
                    .clicked()
                {
                    self.app.set_idle_timeout(3);
                    self.app.state.activity(std::time::Instant::now());
                    ui.close();
                }
            });
        });
        self.app.show(root);
    }
}
fn main() -> eframe::Result {
    std::panic::set_hook(Box::new(|_| {}));
    let directory = std::env::temp_dir().join("authenticator-task05-isolated-ui-20261112");
    eframe::run_native(
        "Authenticator vault preview",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Authenticator vault preview")
                .with_inner_size([420., 840.])
                .with_min_inner_size([320., 460.]),
            ..Default::default()
        },
        Box::new(move |cc| {
            Ok(Box::new(Preview {
                app: AuthenticatorApp::with_repository(cc, VaultStore::open_directory(&directory)),
            }))
        }),
    )
}
