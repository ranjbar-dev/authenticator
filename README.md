# Authenticator

Windows-first desktop authenticator written in Rust. Work is tracked in [tasks/README.md](tasks/README.md).

This is the task 01 foundation, **not a usable authenticator**. It stores no user data. Credential creation, encryption, QR capture and recovery are pending sequential tasks. Test with disposable accounts only once those features exist.

## Development on Windows

Install Rust stable with `rustup`, Visual Studio C++ Build Tools and a Windows SDK. The dependency MSRV is Rust 1.95; this checkout was built with Rust 1.98.0, `x86_64-pc-windows-msvc`. `rust-toolchain.toml` selects stable plus rustfmt/clippy. Dependency versions are locked in Cargo.lock; use `--locked` below. A compiler update is not a dependency-lock update.

```powershell
rtk cargo build --locked --bin Authenticator
rtk cargo run --locked --bin Authenticator
rtk cargo run --locked --bin Authenticator -- --framework-spike
rtk cargo build --locked --example framework_spike
rtk cargo fmt --all -- --check
rtk cargo clippy --locked --all-targets -- -D warnings
rtk cargo test --locked --all-targets
```

RTK is the local command proxy required by this workspace's rules; elsewhere run the underlying Cargo commands directly. The debug executable is `target/debug/Authenticator.exe`. The spike flag exposes inert test controls, resize/zoom actions and a separate region-selection test viewport. It captures no screen pixels. Do not type credentials into the spike.

For automated pointer/keyboard/accessibility checks, launch the no-argument `target/debug/examples/framework_spike.exe` with the computer-use helper's documented `launch_app` API. An elevated terminal's `Start-Process` inherits high integrity; the medium-integrity helper cannot inject input or inspect its controls through Windows UIPI. Authenticator does not require elevation. An unelevated launch exposes labeled controls and supports real input. Application zoom is not a Windows DPI test. Secondary egui viewport accessibility remains a documented limitation; task 07 will use a Windows-native capture overlay adapter with accessible controls.

Runtime requires no registration or network service. Build-time Cargo dependency downloads and GitHub CI require network access. No framework persistence, link launching, telemetry or network client is enabled by this application. Packaging/user instructions follow in task 23.

See [architecture](docs/architecture.md), [security design](docs/security-design.md) and [reference evidence](docs/reference-ui.md). Windows CI runs formatting, warning-free Clippy, tests and a debug build; a local pass does not establish a hosted CI pass.
