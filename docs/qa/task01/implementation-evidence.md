# Task 01 implementation evidence

12 November 2026. Implementer report for independent review, **not acceptance**. Task status remains incomplete until parent acceptance. Initial failed observations below are retained as history; the material-fix follow-up records their resolution and the remaining secondary-viewport accessibility limit.

## Build and checks

Environment: Windows x64, `rustc 1.98.0 (88d9e12ae 2026-08-18)`, `cargo 1.98.0`, stable `x86_64-pc-windows-msvc`. Current eframe registry version confirmed with `rtk proxy cargo info eframe`: 0.37.0, MSRV 1.95, dual MIT/Apache-2.0. Cargo.lock records resolution.

| Command | Actual result |
|---|---|
| `rtk cargo build --locked --bin Authenticator` | Pass, dev build, after correcting the new 0.37 viewport callback's `&mut Ui` signature |
| `rtk cargo fmt --all -- --check` | Pass, exit 0 |
| `rtk cargo clippy --locked --all-targets -- -D warnings` | Pass, no issues; initial collapsible-if warning fixed |
| `rtk cargo test --locked --all-targets --target-dir target/task01-checks` | Pass, 6 tests across 3 test suites, exit 0 |
| `rtk cargo tree -e features -i accesskit_windows` | Confirms accesskit_windows 0.35.2 → accesskit_winit 0.34.2 → egui-winit/eframe 0.37.0; application accesskit feature enabled |
| `rtk proxy powershell -NoProfile -Command 'Start-Process -FilePath D:/Projects/github/authenticator/target/debug/Authenticator.exe -ArgumentList --framework-spike'` | Launched native Authenticator window |

An initial `rtk cargo test --locked --all-targets` failed because Windows denied replacing the running Authenticator.exe. The separate target directory above preserves the currently running inert spike and allows the real test suite to complete. This is a development build-lock issue, not a claimed passing test. Hosted Windows CI has been configured but has **not run** in this implementation child; parent owns commit/push and verifying hosted checks.

Tests cover an injected deterministic clock, signed/invalid/overflowing desktop geometry, a real Win32 display-geometry adapter call, isolated temporary-file cleanup and fixed public error messages. These are foundation tests, not tests of future TOTP/encryption/capture functionality. All fixture content is inert; no account/OTP/seed/passphrase was used.

## Initial UI observations (superseded diagnosis, retained history)

Computer Use `@oai/sky` through node_repl selected exactly one returned process window, activated it, and requested screenshot/UI Automation state. [spike-initial.jpg](spike-initial.jpg) shows the new native app at its initial narrow window, dark theme, no credentials and a visible focus border on the test field. [native-accessibility.txt](native-accessibility.txt) records the actual tree: **only the window and title bar**. No screen-reader/label pass is claimed.

Attempted real input: `sky.click` on the visible Check keyboard action button, `sky.press_key` Tab and Alt+F4, explicit `sky.activate_window`, delayed separate snapshots, and a coordinate-targeting retry. The screenshots did **not** verify action feedback or focus movement. A drag attempting to resize beyond current window bounds was rejected by the helper (`point (600, 710) is outside window bounds ... width:422,height:712`); this does not prove resizing works or fails. We did not automate OS/security/privacy settings or any reference vault.

Diagnosis performed: confirmed native AccessKit feature graph, inspected the eframe/egui-winit activation and WM event integration in downloaded source; removed manual `Context::enable_accesskit()` because eframe handles activation; added spike-only 250ms repaint while diagnosing. Rebuilt/relaunched; the app process reports Responding=True, but actionable UI Automation and input results remain unresolved. Parent will independently probe tooling/framework behavior. Do not claim successful keyboard, pointer, overlay, resize or DPI acceptance based only on this screenshot.

Initial inert instance at first handoff: PID **37736**; returned window ID **194710810**, app `process:D:\Projects\github\authenticator\target\debug\Authenticator.exe`, title `Authenticator`. These handles are historical and must not be reused. It was left running at the initial handoff; the parent subsequently replaced it for diagnosis.

Available display geometry reported by the real Win32 adapter is 6400 × 1440 physical virtual-desktop pixels. Negative-coordinate behavior is unit tested; physical monitor layout, mixed 100/150/200% DPI, OS screen-reader behavior and per-monitor transforms have **not** been verified. Spike scale controls change application zoom only, not Windows DPI.

## Reference and scope

`Get-AppxPackage` found installed `FirstOrder.AI.AuthenticatorApp`; no running reference window or checked-in screenshots. Unknown vault state prevented safe launch/inspection. [reference-ui.md](../../reference-ui.md) documents the installed-but-not-opened distinction and PRD-inventory fallback. No reference screenshot or exact visual-fidelity assertion exists.

Implemented paths: Cargo.toml/Cargo.lock, rust-toolchain.toml, rustfmt.toml, .gitignore, .github/workflows/windows.yml, README.md; src/lib.rs/main.rs, src/domain/{mod,clock}.rs, src/error.rs, src/platform/{mod,capture}.rs, src/storage/mod.rs, src/ui/{mod,theme}.rs; tests/foundation.rs; docs/architecture.md/security-design.md/reference-ui.md and this evidence directory.

Not implemented in this task: real vault/cryptography, TOTP, true selected-region pixel capture/QR, production shared shell or any secondary workflow. Crypto/KDF/file-format/key lifetime decisions are recorded before implementation, with calibration deferred explicitly to task 03. Task 04 owns final shared components and checker calibration.

At the initial handoff, no commits, pushes, task status changes or child agents had been performed by the implementer; material framework feasibility gates remained open. The follow-up below supersedes that unresolved implementation state while preserving the observed failure history.

## Material-fix follow-up: verified main-window/overlay interactions

Parent diagnosis found the shell launched applications at **High Mandatory Level**; the UI helper runs at medium integrity. Windows UIPI prevented its input/accessibility operations. Parent added `examples/framework_spike.rs`, which starts the inert spike without command-line arguments, and launched it through the documented computer-use helper `launch_app`. The child reproduced this unelevated path using `sky.launch_app({ app: 'D:\\Projects\\github\\authenticator\\target\\debug\\examples\\framework_spike.exe' })`. Root controls then exposed full native AccessKit labels and responded to actual input. No security setting was changed or bypassed.

Parent review also found that `drag_delta()` was per frame; parent corrected it to `total_drag_delta()`. The material-fix child changed the viewport from deferred to immediate, retained the selection rectangle after release, and used directly owned boolean state for cancellation. Escape or visible Cancel clears selection, closes the viewport and requests main focus. The deferred viewport had opened but did not verify working cancellation; that defect is now superseded by these observed checks:

| Actual input/check | Observed result and evidence |
|---|---|
| Pointer click `Check keyboard action` | Feedback becomes `Keyboard or pointer action received.` |
| Pointer focus on test field, physical key `a` | Field value `a` visible and exposed in UI Automation; no credentials used |
| Tab from field, then Tab again | Focus moves to Check keyboard action, then Open region viewport test, with visible focus outlines |
| Return on focused region action | Separate native region viewport opens |
| Real `sky.drag` from (80,150) to (450,320) in overlay | Full rectangle persists after pointer release: [region-selection.jpg](region-selection.jpg) |
| Physical Escape in overlay, then fresh window inventory | Overlay disappears; main UI Automation focus returns to Open region viewport test |
| Reopen by real pointer, click visible Cancel at (70,92) | Overlay disappears; main focus returns to the opening control |
| Real Narrow / Intermediate / Desktop button clicks | Outer screenshot sizes 342×612, 602×712 and 962×752; screenshots [narrow](spike-narrow.jpg), [intermediate](spike-intermediate.jpg), [desktop](spike-desktop.jpg) |
| Real 150% / 200% application zoom and pointer scrolling | Text/layout rescale, 200% scroll reaches the bottom controls, default zoom restored; [150%](spike-zoom150.jpg), [200% scrolled](spike-zoom200-scrolled.jpg) |

[accessible-controls.txt](accessible-controls.txt) contains the new **full** main-window accessible tree, including the associated Test label field, buttons and action feedback. `native-accessibility.txt` remains the original elevated-instance failure, not current evidence. One resize check returned `unknown screenshotId screenshot-0`; refreshed the returned window and screenshot before retry, which succeeded. Overlay close observations were taken in fresh separate calls after the action/event frame settled.

Remaining limitation: secondary egui viewport UI Automation still exposes only native title-bar controls. This is not an app-wide accessibility failure: root controls are accessible. The actual eframe Glow source explicitly calls `init_accesskit` for ROOT around lines 382–393, while the per-viewport `initialize_window` creates a new egui_winit State around lines 1396–1408 without it. Architecture records the bounded decision to use a native Windows capture-overlay adapter in task 07 and verify its accessible controls then. Do not call secondary-window Narrator support, actual screen capture, physical DPI/mixed monitors or a complete accessibility audit passed here.

Rechecks after material fixes:

- `rtk cargo build --locked --example framework_spike` — pass.
- `rtk cargo fmt --all -- --check` — pass, exit 0.
- `rtk cargo clippy --locked --all-targets -- -D warnings` — pass, no issues.
- `rtk cargo test --locked --all-targets --target-dir target/task01-checks` — pass, 6 tests across 4 suites (example added), exit 0.

Ready for independent re-review of this task's foundation/spike scope. Current inert example at material-fix handoff: PID **7904**, returned main window **5638976**, `process:D:\Projects\github\authenticator\target\debug\examples\framework_spike.exe`, title `Authenticator framework spike`. Overlay is closed and app scale restored to 100%. Refresh inventory before input. The spike contains only the inert letter `a`; no credentials/pixels/payloads were imported or captured.

## Parent acceptance checkpoint

Independent reviewer review_01 rechecked material fixes, sanitized screenshots and live pointer/keyboard/overlay behavior and approved task01 scope. Hosted Windows CI passed all format/lint/test/build steps on 7502fa4dc82d5904c457f2cfda396d56badf044d: https://github.com/ranjbar-dev/authenticator/actions/runs/43227714011. Parent accepted task01; future requirements and the explicit physical DPI/Narrator/secondary-overlay/reference limitations above remain unverified. No current security certification or usable vault is implied.
