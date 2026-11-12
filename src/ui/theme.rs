use eframe::egui::{self, Color32};

// Proposed foundation tokens, not measured reference values. Task 04 owns the shell.
pub const BACKGROUND: Color32 = Color32::from_rgb(16, 20, 28);
pub const SURFACE: Color32 = Color32::from_rgb(28, 35, 47);
pub const TEXT: Color32 = Color32::from_rgb(239, 244, 255);
pub const MUTED: Color32 = Color32::from_rgb(174, 187, 207);
pub const BLUE: Color32 = Color32::from_rgb(75, 143, 255);
pub const GAP: f32 = 12.0;
pub const CONTROL_HEIGHT: f32 = 36.0;

pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = SURFACE;
    visuals.override_text_color = Some(TEXT);
    visuals.selection.bg_fill = BLUE;
    visuals.selection.stroke = egui::Stroke::new(1.0, TEXT);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, BLUE);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, BLUE);
    ctx.set_visuals(visuals);
}
