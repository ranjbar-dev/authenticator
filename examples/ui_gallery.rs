//! Disposable UI evidence harness. Never opens the user's default vault.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use authenticator::{
    domain::records::{Timestamp, Vault},
    storage::VaultStore,
    ui::{components as c, shell::AuthenticatorApp, theme},
};
use eframe::egui;
use zeroize::Zeroizing;

struct Gallery {
    app: AuthenticatorApp,
    _directory: tempfile::TempDir,
    controls: bool,
    value: String,
    error: bool,
    validated: bool,
    dialog: Option<c::Dialog>,
    dialog_value: String,
    masked_value: String,
}
impl Gallery {
    fn new(
        cc: &eframe::CreationContext<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let directory = tempfile::tempdir()?;
        let mut store = VaultStore::open_directory(directory.path())?;
        store.create(
            Zeroizing::new("disposable-gallery-only".into()),
            Vault::new(Timestamp(1)),
        )?;
        let app = AuthenticatorApp::with_repository(cc, Ok(store));
        Ok(Self {
            app,
            _directory: directory,
            controls: false,
            value: String::new(),
            error: false,
            validated: false,
            dialog: None,
            dialog_value: String::new(),
            masked_value: String::new(),
        })
    }
}
impl eframe::App for Gallery {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        c::restore_dialog_focus(&ctx);
        egui::Panel::top("demo-tools").show(root, |ui| {
            c::secondary(ui, "DISPOSABLE UI PREVIEW · no real credentials");
            c::toolbar(ui, |ui| {
                if ui.button(theme::LABEL.text("Shell")).clicked() {
                    self.controls = false;
                }
                if ui.button(theme::LABEL.text("Controls")).clicked() {
                    self.controls = true;
                }
                ui.menu_button(theme::LABEL.text("Preview tools"), |ui| {
                    c::secondary(ui, "Window size in logical points");
                    for (name, size) in [
                        ("Narrow 340 × 680", [340.0, 680.0]),
                        ("Intermediate 600 × 740", [600.0, 740.0]),
                        ("Desktop 960 × 800", [960.0, 800.0]),
                    ] {
                        if ui.button(name).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size.into()));
                            ui.close();
                        }
                    }
                    c::secondary(ui, "App zoom · does not change Windows DPI");
                    for (name, scale) in [("100%", 1.0), ("150%", 1.5), ("200%", 2.0)] {
                        if ui.button(name).clicked() {
                            ctx.set_zoom_factor(scale);
                            ui.close();
                        }
                    }
                    if ui.button("Open disposable empty vault").clicked() {
                        let _ = self.app.state.repository_mut().and_then(|store| {
                            store.unlock(Zeroizing::new("disposable-gallery-only".into()))
                        });
                        ui.close();
                    }
                    if ui.button("Lock disposable vault").clicked() {
                        self.app.lock();
                        ui.close();
                    }
                });
            });
        });
        if !self.controls {
            self.app.show(root);
            return;
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BACKGROUND)
                    .inner_margin(theme::PADDING),
            )
            .show(root, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_max_width(theme::CONTENT_MAX_WIDTH.min(ui.available_width()));
                    c::heading(ui, "Shared controls", 1);
                    c::secondary(ui, "Only a component preview. No records are saved.");
                    c::card(ui, |ui| {
                        c::heading(ui, "Labeled field", 2);
                        c::labeled_field(
                            ui,
                            "preview-label",
                            "Preview label",
                            &mut self.value,
                            "Enter any non-sensitive label.",
                            self.error.then_some("Enter a preview label."),
                        );
                        if c::button(ui, "Validate preview", c::ButtonStyle::Primary).clicked() {
                            self.error = self.value.trim().is_empty();
                            self.validated = !self.error;
                        }
                        if self.validated {
                            c::status(
                                ui,
                                "validation",
                                "Preview accepted. Nothing was saved.",
                                c::StatusKind::Success,
                            );
                        }
                    });
                    let opener = c::button(ui, "Open preview dialog", c::ButtonStyle::Secondary);
                    if opener.clicked() {
                        self.dialog = Some(c::Dialog::new("Preview dialog", opener.id));
                    }
                    c::toolbar(ui, |ui| {
                        for (icon, name) in [
                            (c::Icon::Help, "Preview help icon"),
                            (c::Icon::Plus, "Preview add icon"),
                            (c::Icon::Lock, "Preview lock icon"),
                            (c::Icon::Key, "Preview key icon"),
                            (c::Icon::Recovery, "Preview recovery icon"),
                            (c::Icon::Settings, "Preview settings icon"),
                        ] {
                            if c::icon_button(ui, icon, name).clicked() {
                                self.validated = true;
                            }
                        }
                    });
                    ui.add_enabled_ui(false, |ui| {
                        c::button(ui, "Disabled example", c::ButtonStyle::Secondary);
                    });
                    c::status(
                        ui,
                        "error-preview",
                        "Error example: the operation could not be completed.",
                        c::StatusKind::Error,
                    );
                    c::status(
                        ui,
                        "info-preview",
                        "Information example: your existing data is unchanged.",
                        c::StatusKind::Info,
                    );
                    c::progress(ui, "Progress example · 60%", Some(0.6));
                    c::progress(ui, "Waiting example", None);
                    c::empty_state(
                        ui,
                        c::Icon::Recovery,
                        "Empty-state example",
                        "A concise explanation appears near the next action.",
                    );
                });
            });
        if let Some(dialog) = &mut self.dialog {
            let close = dialog.show(&ctx, |ui, first| {
                let field = c::labeled_field(
                    ui,
                    "dialog-field",
                    "Dialog label",
                    &mut self.dialog_value,
                    "This is a non-sensitive preview field.",
                    None,
                );
                if first {
                    field.request_focus();
                }
                c::labeled_secret_field(
                    ui,
                    "masked-preview",
                    "Masked preview",
                    &mut self.masked_value,
                    "Disposable input only. Never saved.",
                    None,
                );
                c::button(ui, "Close preview dialog", c::ButtonStyle::Primary).clicked()
            });
            if close {
                self.dialog = None;
            }
        }
    }
}
fn main() -> eframe::Result {
    std::panic::set_hook(Box::new(|_| {}));
    eframe::run_native(
        "Authenticator UI gallery",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Authenticator UI gallery")
                .with_inner_size([420.0, 740.0])
                .with_min_inner_size([320.0, 460.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(Gallery::new(cc)?))),
    )
}
