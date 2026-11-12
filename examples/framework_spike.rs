#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() -> eframe::Result {
    std::panic::set_hook(Box::new(|_| {}));
    eframe::run_native(
        "Authenticator framework spike",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_title("Authenticator framework spike")
                .with_inner_size([420.0, 680.0])
                .with_min_inner_size([320.0, 460.0])
                .with_resizable(true),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(authenticator::ui::FoundationApp::new(cc, true)))),
    )
}
