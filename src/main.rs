#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn install_private_panic_hook() {
    // Native crash dumps and dependency/OS memory remain outside this hook.
    std::panic::set_hook(Box::new(|_| {}));
}

fn main() -> eframe::Result {
    install_private_panic_hook();
    // The inert foundation gallery is a development tool, never a release route.
    #[cfg(debug_assertions)]
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
            #[cfg(debug_assertions)]
            if spike {
                return Ok(Box::new(authenticator::ui::FoundationApp::new(cc, true)));
            }
            Ok(Box::new(authenticator::ui::shell::AuthenticatorApp::new(
                cc,
            )))
        }),
    )
}

#[cfg(test)]
mod tests {
    const PROBE: &str = "AUTHENTICATOR_TEST_PRIVATE_PANIC";
    const SENTINEL: &str = "Public disposable panic payload must stay private";
    const ARMED: &str = "PRIVATE_PANIC_PROBE_ARMED";

    #[test]
    fn production_panic_hook_does_not_emit_payload_in_child_output()
    -> Result<(), Box<dyn std::error::Error>> {
        let child = std::process::Command::new(std::env::current_exe()?)
            .args([
                "--ignored",
                "--exact",
                "tests::private_panic_child",
                "--nocapture",
            ])
            .env(PROBE, "1")
            .output()?;
        assert_eq!(
            child.status.code(),
            Some(101),
            "the child must report a test panic"
        );
        assert!(String::from_utf8_lossy(&child.stdout).contains(ARMED));
        for output in [&child.stdout, &child.stderr] {
            assert!(
                !output
                    .windows(SENTINEL.len())
                    .any(|b| b == SENTINEL.as_bytes()),
                "a panic payload escaped the production hook"
            );
        }
        Ok(())
    }

    #[test]
    #[ignore = "subprocess entrypoint, exercised by production_panic_hook_does_not_emit_payload_in_child_output"]
    fn private_panic_child() {
        assert_eq!(std::env::var(PROBE).as_deref(), Ok("1"));
        super::install_private_panic_hook();
        println!("{ARMED}");
        panic!("{SENTINEL}");
    }
}
