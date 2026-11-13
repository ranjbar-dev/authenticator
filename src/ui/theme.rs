//! Proposed inventory-based tokens, not measurements of the unavailable reference.
use eframe::egui::{self, Color32, FontFamily, FontId, RichText, Stroke};

pub const BACKGROUND: Color32 = Color32::from_rgb(16, 20, 28);
pub const SURFACE: Color32 = Color32::from_rgb(28, 35, 47);
pub const FIELD: Color32 = Color32::from_rgb(21, 28, 39);
pub const HOVER: Color32 = Color32::from_rgb(40, 52, 70);
pub const TEXT: Color32 = Color32::from_rgb(239, 244, 255);
pub const MUTED: Color32 = Color32::from_rgb(174, 187, 207);
pub const BLUE: Color32 = Color32::from_rgb(75, 143, 255);
// Blue, purple, pink, red, orange, green: opaque metadata accents, never text-only cues.
pub const FOLDER_COLORS: [Color32; 6] = [
    BLUE,
    Color32::from_rgb(190, 160, 255),
    Color32::from_rgb(255, 166, 212),
    Color32::from_rgb(255, 153, 153),
    Color32::from_rgb(255, 190, 117),
    Color32::from_rgb(139, 222, 172),
];
pub const ON_BLUE: Color32 = Color32::from_rgb(9, 20, 38);
pub const ERROR: Color32 = Color32::from_rgb(255, 177, 177);
pub const ERROR_SURFACE: Color32 = Color32::from_rgb(57, 31, 40);
pub const SUCCESS: Color32 = Color32::from_rgb(155, 225, 191);
pub const BORDER: Color32 = Color32::from_rgb(108, 126, 151);
pub const FOCUS: Color32 = Color32::from_rgb(143, 188, 255);
pub const SCRIM: Color32 = Color32::from_black_alpha(190);

// A new four-point rhythm, expressly proposed for the shared shell.
pub const UNIT: f32 = 4.0;
pub const SMALL_GAP: f32 = UNIT * 2.0;
pub const GAP: f32 = UNIT * 3.0;
pub const PADDING: i8 = 16;
pub const CONTROL_HEIGHT: f32 = UNIT * 9.0;
pub const ICON_SIZE: f32 = UNIT * 5.0;
pub const ICON_STROKE: f32 = 2.0;
pub const FOCUS_STROKE: f32 = 2.0;
pub const RADIUS: u8 = 10;
pub const CONTROL_RADIUS: u8 = 6;
pub const CONTENT_MAX_WIDTH: f32 = 640.0;
pub const CODES_GRID_MIN_WIDTH: f32 = 600.0;
pub const CODES_TOOLBAR_WIDE: f32 = 500.0;
pub const CODE_CARD_CONTENT_HEIGHT: f32 = 240.0;
pub const COMPACT_TABS_WIDTH: f32 = 292.0;
pub const TABS_SINGLE_LINE_WIDTH: f32 = 380.0;
pub const WRAPPED_TAB_HEIGHT: f32 = BODY.line_height * 2.0 + SMALL_GAP * 2.0;
pub const CLIPBOARD_FEEDBACK_HEIGHT: f32 = BODY.line_height * 3.0 + PADDING as f32 * 2.0;
pub const DIALOG_WIDTH: f32 = 360.0;
pub const PROGRESS_HEIGHT: f32 = UNIT * 2.0;
// Native selector geometry, in the same logical-pixel scale as ROOT controls.
pub const CAPTURE_PANEL_WIDTH: f32 = 560.0;
pub const CAPTURE_PANEL_HEIGHT: f32 = 160.0;
pub const CAPTURE_INSTRUCTIONS_HEIGHT: f32 = 72.0;
pub const CAPTURE_CANCEL_TOP: f32 = 108.0;
pub const CAPTURE_DIM_ALPHA: u8 = 48;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Weight {
    Regular,
    Semibold,
}
#[derive(Clone, Copy, Debug)]
pub enum TypeFamily {
    NotoSans,
    Monospace,
}
#[derive(Clone, Copy, Debug)]
pub enum TextCase {
    AsWritten,
}
/// Whole role bundle, in logical points. Weight selects an actual bundled font.
#[derive(Clone, Copy, Debug)]
pub struct Typography {
    pub size: f32,
    pub weight: Weight,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub family: TypeFamily,
    pub case: TextCase,
}
pub const TITLE: Typography = Typography {
    size: 24.0,
    weight: Weight::Semibold,
    line_height: 32.0,
    letter_spacing: 0.0,
    family: TypeFamily::NotoSans,
    case: TextCase::AsWritten,
};
pub const SECTION: Typography = Typography {
    size: 18.0,
    weight: Weight::Semibold,
    line_height: 24.0,
    letter_spacing: 0.0,
    family: TypeFamily::NotoSans,
    case: TextCase::AsWritten,
};
pub const BODY: Typography = Typography {
    size: 14.0,
    weight: Weight::Regular,
    line_height: 20.0,
    letter_spacing: 0.0,
    family: TypeFamily::NotoSans,
    case: TextCase::AsWritten,
};
/// Shared recovery editor viewport: six BODY rows plus control breathing room.
pub const MULTILINE_HEIGHT: f32 = BODY.line_height * 6.0 + SMALL_GAP;
pub const LABEL: Typography = Typography {
    weight: Weight::Semibold,
    ..BODY
};
pub const CAPTION: Typography = Typography {
    size: 13.0,
    weight: Weight::Regular,
    line_height: 20.0,
    letter_spacing: 0.0,
    family: TypeFamily::NotoSans,
    case: TextCase::AsWritten,
};
pub const CODE: Typography = Typography {
    size: 28.0,
    weight: Weight::Regular,
    line_height: 36.0,
    letter_spacing: 1.0,
    family: TypeFamily::Monospace,
    case: TextCase::AsWritten,
};
impl Typography {
    pub fn font(self) -> FontId {
        let family = match self.family {
            TypeFamily::Monospace => FontFamily::Monospace,
            TypeFamily::NotoSans => FontFamily::Name(
                match self.weight {
                    Weight::Regular => "Authenticator Regular",
                    Weight::Semibold => "Authenticator Semibold",
                }
                .into(),
            ),
        };
        FontId::new(self.size, family)
    }
    pub fn text(self, text: impl Into<String>) -> RichText {
        RichText::new(text)
            .font(self.font())
            .line_height(Some(self.line_height))
            .extra_letter_spacing(self.letter_spacing)
    }
    /// Editable text needs the same whole bundle as labels. TextEdit's default
    /// layouter only reads its font, so supply the remaining metrics explicitly.
    pub fn singleline_job(self, text: String, color: Color32) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::simple_singleline(text, self.font(), color);
        job.keep_trailing_whitespace = true;
        for section in &mut job.sections {
            section.format.line_height = Some(self.line_height);
            section.format.extra_letter_spacing = self.letter_spacing;
        }
        job
    }
}

pub fn apply(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "NotoSans Regular".into(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/NotoSans-Regular.ttf"))
            .into(),
    );
    fonts.font_data.insert(
        "NotoSans Semibold".into(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/NotoSans-SemiBold.ttf"))
            .into(),
    );
    for (family, font) in [
        ("Authenticator Regular", "NotoSans Regular"),
        ("Authenticator Semibold", "NotoSans Semibold"),
    ] {
        fonts
            .families
            .insert(FontFamily::Name(family.into()), vec![font.into()]);
    }
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "NotoSans Regular".into());
    ctx.set_fonts(fonts);
    let mut style = egui::Style::default();
    style.spacing.item_spacing = egui::vec2(SMALL_GAP, GAP);
    style.spacing.button_padding = egui::vec2(GAP, SMALL_GAP);
    style.spacing.interact_size = egui::vec2(CONTROL_HEIGHT, CONTROL_HEIGHT);
    style.spacing.window_margin = egui::Margin::same(PADDING);
    style
        .text_styles
        .insert(egui::TextStyle::Heading, TITLE.font());
    style.text_styles.insert(egui::TextStyle::Body, BODY.font());
    style
        .text_styles
        .insert(egui::TextStyle::Button, LABEL.font());
    style
        .text_styles
        .insert(egui::TextStyle::Small, CAPTION.font());
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, CODE.font());
    style.visuals = egui::Visuals::dark();
    let v = &mut style.visuals;
    v.panel_fill = BACKGROUND;
    v.window_fill = SURFACE;
    v.extreme_bg_color = FIELD;
    v.faint_bg_color = SURFACE;
    v.override_text_color = Some(TEXT);
    v.selection.bg_fill = BLUE;
    v.selection.stroke = Stroke::new(ICON_STROKE, ON_BLUE);
    v.hyperlink_color = BLUE;
    v.error_fg_color = ERROR;
    v.window_corner_radius = egui::CornerRadius::same(RADIUS);
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.bg_fill = SURFACE;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.inactive.bg_fill = SURFACE;
    v.widgets.inactive.weak_bg_fill = SURFACE;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.inactive.fg_stroke = Stroke::new(ICON_STROKE, TEXT);
    v.widgets.hovered.bg_fill = HOVER;
    v.widgets.hovered.weak_bg_fill = HOVER;
    v.widgets.hovered.bg_stroke = Stroke::new(FOCUS_STROKE, FOCUS);
    v.widgets.hovered.fg_stroke = Stroke::new(ICON_STROKE, TEXT);
    v.widgets.active = v.widgets.hovered;
    for widget in [
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.noninteractive,
    ] {
        widget.corner_radius = egui::CornerRadius::same(CONTROL_RADIUS);
    }
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_style_of(egui::Theme::Dark, style);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn luminance(c: Color32) -> f64 {
        let channel = |n: u8| {
            let s = f64::from(n) / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
    }
    fn contrast(a: Color32, b: Color32) -> f64 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
    #[test]
    fn actual_text_and_essential_graphic_pairs_meet_project_thresholds() {
        for (name, fg, bg) in [
            ("body", TEXT, BACKGROUND),
            ("card", TEXT, SURFACE),
            ("field", TEXT, FIELD),
            ("hover", TEXT, HOVER),
            ("secondary", MUTED, SURFACE),
            ("secondary hover", MUTED, HOVER),
            ("primary action", ON_BLUE, BLUE),
            ("error", ERROR, ERROR_SURFACE),
            ("success", SUCCESS, SURFACE),
            ("accent", BLUE, SURFACE),
        ] {
            let ratio = contrast(fg, bg);
            println!("{name}: {ratio:.2}:1");
            assert!(ratio >= 4.5, "{name}: {ratio}");
        }
        for bg in [BACKGROUND, SURFACE, FIELD, HOVER] {
            let ratio = contrast(FOCUS, bg);
            println!("focus: {ratio:.2}:1");
            assert!(ratio >= 3.0);
            assert!(contrast(BORDER, bg) >= 3.0);
        }
        assert!(contrast(ON_BLUE, BLUE) >= 3.0);
    }
}
