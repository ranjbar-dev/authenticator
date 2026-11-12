//! Task08 public in-memory QR fixture. No QR image or capture file output.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
struct Fixture {
    codes: Vec<qrcode::QrCode>,
    mode: usize,
    hwnd: usize,
}
impl eframe::App for Fixture {
    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
        for (key, mode) in [
            (egui::Key::F5, 0),
            (egui::Key::F6, 1),
            (egui::Key::F7, 2),
            (egui::Key::F9, 3),
        ] {
            if root.ctx().input(|i| i.key_pressed(key)) {
                self.mode = mode;
            }
        }
        egui::CentralPanel::default().show(root, |ui| {
            ui.heading("Public disposable QR batch fixture");
            ui.label("RFC6238 public seed · no file output · do not persist source screenshots");
            ui.label("F5: one TOTP · F6: two TOTP · F7: unsupported · F9: blank");
            let mut bounds = egui::Rect::NOTHING;
            ui.horizontal(|ui| {
                let indexes: &[usize] = match self.mode {
                    0 => &[0],
                    1 => &[0, 1],
                    2 => &[2],
                    _ => &[],
                };
                for index in indexes {
                    let qr = &self.codes[*index];
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(360., 360.), egui::Sense::hover());
                    bounds = bounds.union(rect);
                    ui.painter().rect_filled(rect, 0., egui::Color32::WHITE);
                    let n = qr.width();
                    let module = (360. / (n + 8) as f32).floor();
                    let offset = (360. - module * (n + 8) as f32) / 2.;
                    for y in 0..n {
                        for x in 0..n {
                            if qr[(x, y)] == qrcode::Color::Dark {
                                let min = rect.min
                                    + egui::vec2(
                                        offset + (x + 4) as f32 * module,
                                        offset + (y + 4) as f32 * module,
                                    );
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_size(min, egui::vec2(module, module)),
                                    0.,
                                    egui::Color32::BLACK,
                                );
                            }
                        }
                    }
                }
                if indexes.is_empty() {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(360., 360.), egui::Sense::hover());
                    bounds = rect;
                    ui.painter().rect_filled(rect, 0., egui::Color32::WHITE);
                }
            });
            #[cfg(windows)]
            unsafe {
                let mut p = windows_sys::Win32::Foundation::POINT::default();
                windows_sys::Win32::Graphics::Gdi::ClientToScreen(self.hwnd as _, &mut p);
                let scale = ui.ctx().pixels_per_point();
                ui.label(format!(
                    "Select physical region: {},{} .. {},{}",
                    p.x + (bounds.left() * scale).round() as i32,
                    p.y + (bounds.top() * scale).round() as i32,
                    p.x + (bounds.right() * scale).round() as i32,
                    p.y + (bounds.bottom() * scale).round() as i32
                ));
            }
        });
    }
}
fn main() -> eframe::Result {
    std::panic::set_hook(Box::new(|_| {}));
    eframe::run_native(
        "Authenticator public batch fixture",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Authenticator public batch fixture")
                .with_inner_size([780., 510.])
                .with_position([60., 220.]),
            ..Default::default()
        },
        Box::new(|cc| {
            let hwnd = cc
                .window_handle()
                .ok()
                .and_then(|h| match h.as_raw() {
                    RawWindowHandle::Win32(w) => Some(w.hwnd.get() as usize),
                    _ => None,
                })
                .unwrap_or(0);
            let codes = ["otpauth://totp/Public%20Fixture:disposable?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Public%20Fixture", "otpauth://totp/Second%20Fixture:batch?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Second%20Fixture&algorithm=SHA512&digits=8&period=120", "https://example.invalid/unsupported"].into_iter().map(qrcode::QrCode::new).collect::<Result<Vec<_>,_>>().map_err(|_|"public fixture unavailable")?;
            Ok(Box::new(Fixture {
                codes,
                mode: 0,
                hwnd,
            }))
        }),
    )
}
