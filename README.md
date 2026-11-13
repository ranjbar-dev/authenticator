# Authenticator

Windows-first desktop authenticator written in Rust. Work is tracked in [tasks/README.md](tasks/README.md).

Implementation is in progress. The repository now includes an encrypted local vault, TOTP and QR entry, portable encrypted backups, recovery-code management and folders. Remaining features and unverified native/manual acceptance checks are tracked in [the task checkpoint](tasks/README.md). Use disposable test data while required acceptance remains open; no production-readiness claim is made.

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

For automated pointer/keyboard/accessibility checks, launch the no-argument `target/debug/examples/framework_spike.exe` with the computer-use helper's documented `launch_app` API. An elevated terminal's `Start-Process` inherits high integrity; the medium-integrity helper cannot inject input or inspect its controls through Windows UIPI. Authenticator does not require elevation. An unelevated launch exposes labeled controls and supports real input. Application zoom is not a Windows DPI test. Production screen selection uses the Windows-native capture overlay adapter.

Runtime requires no registration or network service. Build-time Cargo dependency downloads and GitHub CI require network access. No framework persistence, link launching, telemetry or network client is enabled by this application.

## Windows portable package

Build the unsigned x64 ZIP with `rtk proxy python scripts/package_windows.py`.
The allowlisted archive and SHA256 file appear under `dist/`; `--replace` only
replaces a verified prior build. [Build/package details](docs/windows-build.md)
describe static CRT linkage, Windows resources, notices and source-dirty metadata.
End users extract and run `Authenticator.exe` without Rust/Python/Node.

[User guide](docs/user-guide.md) covers every shipped workflow and deliberate
update/uninstall handling. [Security and validation limits](docs/security-limits.md)
distinguish implemented behavior from unperformed release gates. App-folder
removal preserves the per-user encrypted vault; backups are separately managed.

Backups use this app's native encrypted `AUTHPBK1` format, normally `.authpbk`.
The reference app's private `.vault` format is unsupported in v1; renaming a file
does not convert it. [Compatibility and migration evidence](docs/migration-compatibility.md)
explains supported inputs and the safe native export/import path.

See [architecture](docs/architecture.md), [security design](docs/security-design.md) and [reference evidence](docs/reference-ui.md). Windows CI runs formatting, warning-free Clippy, tests, a debug build and release/package validation; a local pass does not establish a hosted CI pass.
