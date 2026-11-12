# Architecture and concrete decisions

Task 01, 12 November 2026. Baseline `f7ea0cc` contained only PRD.md, tasks/ and Git metadata. There were no source screens, components, theme, DESIGN.md or reference images. No reuse counts or measured reference tokens are asserted.

## Framework decision and feasibility boundary

Choose egui/eframe **0.37.0**, with explicit `accesskit`, `default_fonts` and `glow` features; default features are off. Rust minimum 1.95, tested compiler 1.98.0. Glow reduces renderer dependencies and works with the available desktop driver. No persistence, links, system-font discovery or web runtime is enabled. This is a native Rust executable, not a JavaScript/web interface.

The `--framework-spike` flag provides a labeled text field, native framework buttons, action feedback, resize and application zoom controls, plus an independent topmost viewport with a drag surface and Escape/cancel. `platform/capture.rs` calls Win32 GetSystemMetrics to prove the native adapter links beside eframe and supports signed virtual-desktop coordinates. The spike never captures pixels, decodes QR or opens/stores a vault. Actual native overlay transparency, pixel selection, pointer capture, per-monitor DPI transforms, OS lifecycle and capture cleanup remain task 07 work, with the integrated user journey in task 08. Do not confuse an application zoom test with a physical Windows DPI/monitor test.

Accessibility uses AccessKit and labeled framework controls. Windows UI Automation/keyboard observations are recorded in task01 evidence. A complete screen reader audit, reduced-motion/high-contrast checks, automatic OTP announcements and cross-screen focus management remain tasks 04/21. Framework selection is based on the small spike, not an unsupported declaration that every final acceptance gate passed.

Official sources checked on 12 November 2026:

- [eframe 0.37 documentation and feature flags](https://docs.rs/eframe/0.37.0/eframe/): native application, accessibility and optional persistence.
- [egui Context viewport API](https://docs.rs/egui/0.37.0/egui/struct.Context.html): immediate/deferred native viewports and repaint control.
- [Microsoft capturing an image](https://learn.microsoft.com/en-us/windows/win32/gdi/capturing-an-image): bounded in-memory GDI capture is feasible; no file serialization is necessary. The later adapter must capture only the approved rectangle and clean native handles.
- [Microsoft DPI awareness](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setprocessdpiawarenesscontext): prefer a manifest for process DPI policy. Confirm eframe/winit behavior before adding a conflicting process setting.
- [Rust platform support](https://doc.rust-lang.org/rustc/platform-support.html): Windows MSVC target; Windows SDK/C++ linker required for development.

## Spike interaction and viewport decisions

The working spike uses `show_viewport_immediate`, not the original deferred viewport: its callback executes during the parent's pass and mutates the selection/cancellation state directly. Selection uses total drag delta from drag start and is retained after release. Escape and visible Cancel clear it, remove the child viewport and restore main-window focus to the opening button. The root window's labeled controls, physical keyboard input, pointer actions and narrow/intermediate/desktop resize controls were verified through an unelevated helper-launched example. Initial elevated-shell launch failures are retained in the evidence history and superseded by the successful checks; do not launch automated checks at a higher integrity level than the helper.

There is a remaining framework limitation for **secondary native viewports**: UI Automation exposes their title bar but not egui child controls on this tested eframe 0.37.0 Glow backend. Source trace: `eframe-0.37.0/src/native/glow_integration.rs` initializes AccessKit explicitly for `ViewportId::ROOT` around lines 382–393. `GlutinWindowContext::initialize_window` around lines 1396–1408 creates other `egui_winit::State` instances without `init_accesskit`. The immediate viewport event handling works, but changing immediate/deferred scheduling does not install that missing native accessibility adapter. [Official tagged source](https://docs.rs/eframe/0.37.0/src/eframe/native/glow_integration.rs.html) is the trace; downloaded source was also inspected locally. The `wgpu_integration.rs` root setup likewise contains its explicit initialization. No complete secondary-window accessibility/Narrator pass is claimed.

**Task 07 adapter decision:** retain eframe for normal product screens in the main accessible viewport; implement screen-region selection through a Windows-native overlay adapter with native accessible cancellation/instructions, physical coordinate handling and lifecycle cleanup. Do not rely on uninitialized secondary eframe AccessKit for the final capture UI. The task 01 immediate viewport demonstrates native-window/pointer/keyboard integration only. Task 07 must test its actual accessible controls, capture permissions, displays/DPI, Escape/Cancel and selected pixels; no requirement is waived by this spike.

## Windows package model

Task 23 will produce an x64 portable ZIP containing the release executable and license notices, with user-scoped LocalAppData storage and explicit opt-in startup registration. No installer, administrator privilege or paid signing credential is required. Updating replaces application files; removing the portable package leaves the separately stored vault intact. Actual package creation and clean-profile recovery remain unverified until task 23.

## Layer boundaries

| Path | Responsibility | Forbidden responsibilities |
|---|---|---|
| `src/domain/` | Typed data, validation, deterministic OTP; injectable `Clock` in `clock.rs` | UI, OS handles, files, networking |
| `src/storage/` | Task 03 encrypted envelopes, schema validation, transactional repository | Plaintext journals, UI, network |
| `src/platform/` | Windows capture, clipboard ownership, lifecycle, startup adapters | Credential validation or business decisions |
| `src/ui/` | Session state, routes, shared theme/widgets, explicit user decisions | Direct plaintext persistence, secret diagnostics |
| `tests/` | Generated/published disposable fixtures in temporary directories | Real credential samples or actual user-data paths |

There is no network-service layer. UTC seconds are injected for OTP generation; idle timeout must use monotonic time. OS clock discontinuities/resume trigger recomputation rather than cached codes. `AppError` is a closed enum of fixed safe messages and carries no source strings, file contents, user paths or secrets. No application logger is installed; the entry point suppresses Rust panic payload printing. This does not prevent OS/process crash dumps.

## Storage and recovery decisions (task 03/11 implementation pending)

All entities and preferences belong inside authenticated ciphertext. Choose a random 256-bit DEK wrapped under an Argon2id-derived passphrase KEK; XChaCha20-Poly1305 is the proposed reviewed primitive. OS CSPRNG salts/nonces, authenticated envelope headers, bounded KDF parameters, zeroizing owned buffers and atomic encrypted write transactions are mandatory. Pin and verify current crypto libraries and calibrate KDF parameters in task 03; no cryptographic implementation or calibration is claimed here.

Local vault and portable backup are separate versioned format domains: a public magic/type, envelope version, bounded KDF parameters/salt, wrapping nonce/wrapped DEK and payload nonce/ciphertext. Schema version stays in authenticated encrypted content. Define exact binary encoding, size limits and AAD in task 03; use a distinct backup format discriminator/task 11 so a local vault is not accidentally imported as a portable backup. Reject future versions without mutation. Never initialize over an unreadable/corrupt file. Unsupported reference `.vault` compatibility remains an evidence-based task 22 decision.

Per-user Windows LocalAppData holds the encrypted vault, with an app-only generic filename and appropriate current-user ACLs. A dedicated instance lock, serialized transactions, same-directory encrypted temporary writes, flush and atomic replacement must preserve the previous committed file on failure. No user-facing path override is implemented by this spike. Test files use `tempfile` and automatic cleanup; do not point tests at user AppData.

Passphrase unlock is local, not an app account. Relaunch starts locked; idle default five minutes; OS session lock/suspend also lock. Clipboard default thirty seconds and clear only if the app still owns that value. Recovery uses an independently encrypted backup and its passphrase, never a reset/backdoor or required device key. Startup is explicit opt-in; imports cannot silently enable it or weaken locking. Many-to-many folders; trash has no automatic expiry. Text recovery-code import is bounded UTF-8 with optional BOM, at most 1 MiB. PNG/JPEG only for image input, exact allocation/payload bounds to be set by task 06. Screenshot/QR data stays only in memory until explicit reviewed import, and is discarded on lock/cancel/failure.

## Shared visual layer and task 01 checklist

Task 01's inert feasibility page is a new diagnostic kind, with zero source-page candidates at baseline; therefore there are no counted project conventions. PRD section 5 is the named reference inventory for the later product, not a pixel reference for diagnostic controls. `src/ui/theme.rs` proposes the single token source; task 04 will extend it and introduce common holders, cards, fields, buttons, feedback and dialogs. Colors and dimensions in this spike are design choices, not observed measurements.

Checklist supplied for independent task 01 review:

- Window holder: native resizable viewport titled Authenticator; no saved geometry/preferences — task 01 requirement.
- Outer surface/header: shared theme and native framework heading; compact dark presentation — PRD inventory fallback.
- Build state: honest foundation notice, no fake vault records or working credential CTAs — scope requirement.
- Spike field: native text edit linked to its visible test label, no sensitive data — framework feasibility.
- Action: framework button with visible action feedback; keyboard/pointer reachable — task 01 acceptance.
- Region test: distinct topmost native viewport, drag feedback and Escape/Cancel release — capture-integration feasibility only.
- Resize/zoom: explicit narrow/intermediate/desktop and app zoom controls, vertically scrollable overflow — feasibility requirement.
- Theme: foreground/background together, centralized named tokens; task 04 owns final shell — proposed shared values.
- Exclusions: no profile, sign-in, sync, language/translation, remote time or companion promotion — PRD exclusions.

The existing orchestrated plan owns implementation/review. Parent will run an independent reviewer; task 04 owns the planned shell-checker calibration before dependent screens. No extra subagents were started by the task 01 child. UI skill reads: finding-patterns/values/planning/implementing/decisions plus their process/checklist/definitions/empty-baseline support. Baseline project inventory and task/PRD reads only; no invented convention counts. Source and future dependency evidence override this snapshot.
