#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() -> eframe::Result {
    // Never print panic payloads: later tasks handle credentials in this process.
    std::panic::set_hook(Box::new(|_| {}));
    let spike = std::env::args().any(|arg| arg == "--framework-spike");
    eframe::run_native(
        "Authenticator",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_title("Authenticator")
                .with_inner_size([420.0, 680.0])
                .with_min_inner_size([320.0, 460.0])
                .with_resizable(true),
            ..Default::default()
        },
        Box::new(move |cc| {
            if spike {
                Ok(Box::new(authenticator::ui::FoundationApp::new(cc, true)))
            } else {
                Ok(Box::new(authenticator::ui::shell::AuthenticatorApp::new(
                    cc,
                )))
            }
        }),
    )
}
