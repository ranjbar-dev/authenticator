//! Argument-free task08 harness. Only a fresh disposable encrypted empty vault;
//! never resolves the default repository or embeds production business records.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use authenticator::{
    domain::records::{Timestamp, Vault},
    storage::VaultStore,
    ui::{components as c, shell::AuthenticatorApp},
};
use eframe::egui;
use zeroize::Zeroizing;
struct Preview {
    app: AuthenticatorApp,
    _directory: tempfile::TempDir,
    arm_lock: bool,
    saved_ciphertext: Option<Vec<u8>>,
    _source_directory: tempfile::TempDir,
    source_image: std::path::PathBuf,
}
#[cfg(windows)]
fn schedule_private_lock() {
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(2));
        unsafe extern "system" fn find(
            hwnd: windows_sys::Win32::Foundation::HWND,
            _: isize,
        ) -> windows_sys::core::BOOL {
            unsafe {
                let mut process = 0;
                windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(
                    hwnd,
                    &mut process,
                );
                if process != windows_sys::Win32::System::Threading::GetCurrentProcessId() {
                    return 1;
                }
                let mut class = [0u16; 128];
                let count = windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW(
                    hwnd,
                    class.as_mut_ptr(),
                    128,
                );
                if count > 0
                    && String::from_utf16_lossy(&class[..count as usize])
                        .starts_with("AuthenticatorLifecycle")
                {
                    // Only this isolated preview's own registered notification
                    // window. Never broadcasts or calls LockWorkStation.
                    windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW(
                        hwnd,
                        windows_sys::Win32::UI::WindowsAndMessaging::WM_WTSSESSION_CHANGE,
                        7,
                        0,
                    );
                    return 0;
                }
                1
            }
        }
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows(Some(find), 0);
        }
    });
}
impl eframe::App for Preview {
    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
        if root.ctx().input(|i| i.key_pressed(egui::Key::F4)) {
            let path = self._directory.path().join("vault.bin");
            if let Some(bytes) = self.saved_ciphertext.take() {
                let _ = std::fs::write(path, bytes);
            } else if let Ok(bytes) = std::fs::read(&path) {
                self.saved_ciphertext = Some(bytes);
                let _ = std::fs::write(path, b"disposable-only external conflict");
            }
        }
        for (key, size) in [
            (egui::Key::F6, [340., 780.]),
            (egui::Key::F7, [600., 800.]),
            (egui::Key::F8, [960., 900.]),
        ] {
            if root.ctx().input(|i| i.key_pressed(key)) {
                root.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        size[0], size[1],
                    )));
            }
        }
        egui::Panel::top("disposable-notice").show(root, |ui| {
            c::secondary(ui, "DISPOSABLE TASK08 · isolated encrypted test vault");
            c::secondary(
                ui,
                &format!(
                    "Original public QR input image: {}",
                    self.source_image.display()
                ),
            );
            c::secondary(
                ui,
                if self.saved_ciphertext.is_some() {
                    "F4: restore ciphertext · write conflict armed"
                } else {
                    "F4: simulate external write conflict (owned fixture only)"
                },
            );
            if let Some(region) = self.app.state.capture_region() {
                c::secondary(
                    ui,
                    &format!(
                        "Selected physical region: {},{} / {} × {}",
                        region.left, region.top, region.width, region.height
                    ),
                );
            }
            #[cfg(windows)]
            if ui
                .button(if self.arm_lock {
                    "Private WTS lock armed for next scan"
                } else {
                    "Arm private WTS lock for next scan"
                })
                .clicked()
            {
                self.arm_lock = true;
            }
            c::secondary(
                ui,
                "F6: 340 · F7: 600 · F8: 960 wide (window sizes, not physical DPI)",
            );
        });
        self.app.show(root);
        #[cfg(windows)]
        if self.arm_lock && self.app.state.capture_busy() {
            self.arm_lock = false;
            schedule_private_lock();
        }
    }
}
fn main() -> eframe::Result {
    std::panic::set_hook(Box::new(|_| {}));
    eframe::run_native(
        "Authenticator import preview",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Authenticator import preview")
                .with_inner_size([480., 780.])
                .with_position([900., 100.]),
            ..Default::default()
        },
        Box::new(|cc| {
            // An ORIGINAL generated image input, never a screenshot/capture.
            // It lives in a separate owned temp directory and is deleted on exit.
            let source_directory =
                tempfile::tempdir().map_err(|_| "source fixture directory unavailable")?;
            let source_image = source_directory.path().join("public-input.png");
            let qr = qrcode::QrCode::new("otpauth://totp/Public%20Image:fixture?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Public%20Image").map_err(|_|"public fixture unavailable")?;
            let side = (qr.width() + 8) * 6;
            let mut image =
                image::GrayImage::from_pixel(side as u32, side as u32, image::Luma([255]));
            for y in 0..qr.width() {
                for x in 0..qr.width() {
                    if qr[(x, y)] == qrcode::Color::Dark {
                        for dy in 0..6 {
                            for dx in 0..6 {
                                image.put_pixel(
                                    ((x + 4) * 6 + dx) as u32,
                                    ((y + 4) * 6 + dy) as u32,
                                    image::Luma([0]),
                                );
                            }
                        }
                    }
                }
            }
            image
                .save(&source_image)
                .map_err(|_| "public original image unavailable")?;
            let directory = tempfile::tempdir().map_err(|_| "test directory unavailable")?;
            let mut store = VaultStore::open_directory(directory.path())
                .map_err(|_| "test repository unavailable")?;
            store
                .create(
                    Zeroizing::new("disposable-capture-only".into()),
                    Vault::new(Timestamp(1)),
                )
                .map_err(|_| "test vault unavailable")?;
            let mut app = AuthenticatorApp::with_repository(cc, Ok(store));
            app.state
                .repository_mut()
                .map_err(|_| "test repository unavailable")?
                .unlock(Zeroizing::new("disposable-capture-only".into()))
                .map_err(|_| "test unlock unavailable")?;
            Ok(Box::new(Preview {
                app,
                _directory: directory,
                arm_lock: false,
                saved_ciphertext: None,
                _source_directory: source_directory,
                source_image,
            }))
        }),
    )
}
