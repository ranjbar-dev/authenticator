//! Separate, generated public fixture window. QR/pixels never written to disk.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
struct Fixture {
    qr: qrcode::QrCode,
    blank: bool,
    hwnd: usize,
    geometry: String,
}
impl eframe::App for Fixture {
    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
        if root.ctx().input(|i| i.key_pressed(egui::Key::F11)) {
            root.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                    !root
                        .ctx()
                        .input(|i| i.viewport().fullscreen.unwrap_or(false)),
                ));
        }
        if root.ctx().input(|i| i.key_pressed(egui::Key::F9)) {
            self.blank = !self.blank;
        }
        if root.ctx().input(|i| i.key_pressed(egui::Key::F10))
            && let Ok(displays) = authenticator::platform::capture::display_bounds()
            && let Some(left) = displays.iter().min_by_key(|m| m.left)
        {
            root.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            root.ctx()
                .send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                    (left.left + 80) as f32 / root.ctx().pixels_per_point(),
                    (left.top + 80) as f32 / root.ctx().pixels_per_point(),
                )));
        }
        egui::CentralPanel::default().show(root,|ui| {
            ui.heading("Public disposable QR fixture");
            ui.label("RFC test seed only · no user credential · no file output");
            ui.label("F9: blank · F10: left display · F11: fullscreen");
            if ui.button(if self.blank {"Show public QR"} else {"Show blank fixture"}).clicked() {self.blank = !self.blank;}
            let (rect,response)=ui.allocate_exact_size(egui::vec2(432.,432.),egui::Sense::hover());
            ui.painter().rect_filled(rect,0.,egui::Color32::WHITE);
            if !self.blank {
                let n=self.qr.width(); let module=(432./(n+8) as f32).floor();
                let offset=(432.-module*(n+8) as f32)/2.;
                for y in 0..n { for x in 0..n { if self.qr[(x,y)]==qrcode::Color::Dark {
                    let min=rect.min+egui::vec2(offset+(x+4) as f32*module,offset+(y+4) as f32*module);
                    ui.painter().rect_filled(egui::Rect::from_min_size(min,egui::vec2(module,module)),0.,egui::Color32::BLACK);
                } } }
            }
            response.widget_info(||egui::WidgetInfo::labeled(egui::Role::Image,true,"Public QR fixture area"));
            #[cfg(windows)] unsafe {
                let mut p=windows_sys::Win32::Foundation::POINT::default();
                windows_sys::Win32::Graphics::Gdi::ClientToScreen(self.hwnd as _,&mut p);
                let scale=ui.ctx().pixels_per_point();
                self.geometry=format!("Fixture physical screen region: left {} top {} right {} bottom {}. Physical DPI {}.",p.x+(rect.left()*scale).round() as i32,p.y+(rect.top()*scale).round() as i32,p.x+(rect.right()*scale).round() as i32,p.y+(rect.bottom()*scale).round() as i32,windows_sys::Win32::UI::HiDpi::GetDpiForWindow(self.hwnd as _));
            }
            ui.label(&self.geometry);
            ui.label(if self.blank {"Blank fixture: safe for persisted overlay evidence."} else {"QR visible: do not persist source or overlay screenshots."});
        });
    }
}
fn main() -> eframe::Result {
    std::panic::set_hook(Box::new(|_| {}));
    eframe::run_native(
        "Authenticator public QR fixture",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Authenticator public QR fixture")
                .with_inner_size([480., 580.])
                .with_position([80., 220.]),
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
            let qr=qrcode::QrCode::new("otpauth://totp/Public%20Fixture:disposable?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Public%20Fixture").map_err(|_|"fixture unavailable")?;
            Ok(Box::new(Fixture {
                qr,
                blank: false,
                hwnd,
                geometry: String::new(),
            }))
        }),
    )
}
