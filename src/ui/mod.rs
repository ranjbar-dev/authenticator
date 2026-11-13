//! Shared product shell. The original feasibility spike remains diagnostic-only.
#[cfg(all(test, windows))]
mod account_tests;
pub mod backup_codes;
mod backup_import;
pub mod codes;
pub mod components;
mod export;
pub mod folders;
pub mod import_review;
mod recovery_detail;
mod recovery_editor;
pub mod shell;
pub mod state;
pub mod theme;

use crate::platform::capture;
use eframe::egui::{self, Color32};

pub struct FoundationApp {
    spike: bool,
    field: String,
    response: &'static str,
    overlay_open: bool,
    selection: Option<egui::Rect>,
}

impl FoundationApp {
    pub fn new(cc: &eframe::CreationContext<'_>, spike: bool) -> Self {
        theme::apply(&cc.egui_ctx);
        Self {
            spike,
            field: String::new(),
            response: "No action yet.",
            overlay_open: false,
            selection: None,
        }
    }

    fn overlay(&mut self, ctx: &egui::Context) {
        if !self.overlay_open {
            return;
        }
        // Immediate viewports run in the parent pass and share its frame scheduling.
        // This avoids the deferred test viewport becoming unresponsive on this host.
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("capture-feasibility"),
            egui::ViewportBuilder::default()
                .with_title("Authenticator · Region feasibility")
                .with_inner_size([600.0, 380.0])
                .with_window_level(egui::WindowLevel::AlwaysOnTop),
            |root, _| {
                let ctx = root.ctx().clone();
                let cancel = ctx
                    .input(|i| i.key_pressed(egui::Key::Escape) || i.viewport().close_requested());
                egui::CentralPanel::default().show(root, |ui| {
                    ui.heading("Region-selection feasibility");
                    ui.label("Drag in this test surface. Escape cancels. No pixels are captured.");
                    if ui.button("Cancel region test").clicked() || cancel {
                        self.overlay_open = false;
                        self.selection = None;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ctx.send_viewport_cmd_to(
                            egui::ViewportId::ROOT,
                            egui::ViewportCommand::Focus,
                        );
                    }
                    let (rect, response) =
                        ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());
                    ui.painter().rect_filled(rect, 8.0, theme::SURFACE);
                    if response.dragged()
                        && let Some(p) = response.interact_pointer_pos()
                        && let Some(delta) = response.total_drag_delta()
                    {
                        let start = p - delta;
                        self.selection = Some(egui::Rect::from_two_pos(start, p).intersect(rect));
                    }
                    if let Some(selection) = self.selection {
                        ui.painter().rect_stroke(
                            selection,
                            0.0,
                            egui::Stroke::new(2.0, theme::BLUE),
                            egui::StrokeKind::Inside,
                        );
                    }
                });
            },
        );
    }
}

impl eframe::App for FoundationApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.spike {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
        egui::CentralPanel::default().show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = theme::GAP;
            ui.heading("Authenticator");
            ui.label(egui::RichText::new("Local. Offline. Under your control.").color(theme::MUTED));
            ui.separator();
            if !self.spike {
                ui.heading("Foundation build");
                ui.label("Vault setup and credential workflows are not available in this development build.");
                ui.label("No user data is stored.");
                return;
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Framework feasibility");
                ui.label("Inert controls for keyboard, accessibility and window checks.");
                let label = ui.label("Test label (no credentials)");
                ui.text_edit_singleline(&mut self.field).labelled_by(label.id);
                if ui.add_sized([ui.available_width(), theme::CONTROL_HEIGHT], egui::Button::new("Check keyboard action")).clicked() {
                    self.response = "Keyboard or pointer action received.";
                }
                ui.label(self.response);
                if ui.button("Open region viewport test").clicked() {
                    self.overlay_open = true;
                    self.selection = None;
                }
                match capture::desktop_bounds() {
                    Ok(b) => { ui.label(format!("Windows desktop geometry: {} × {} physical pixels", b.width, b.height)); }
                    Err(_) => { ui.colored_label(Color32::LIGHT_RED, "Display geometry unavailable."); }
                }
                ui.separator();
                ui.label("Resize test");
                ui.horizontal_wrapped(|ui| {
                    for (name, size) in [("Narrow", [340.0, 580.0]), ("Intermediate", [600.0, 680.0]), ("Desktop", [960.0, 720.0])] {
                        if ui.button(name).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size.into()));
                        }
                    }
                });
                ui.label("UI scale test (does not change Windows settings)");
                ui.horizontal_wrapped(|ui| {
                    for (label, scale) in [("100%", 1.0), ("150%", 1.5), ("200%", 2.0)] {
                        if ui.button(label).clicked() { ctx.set_zoom_factor(scale); }
                    }
                });
            });
        });
        self.overlay(&ctx);
    }
}
