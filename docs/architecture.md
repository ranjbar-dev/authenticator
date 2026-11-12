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

## Task 02 domain and credential decisions

The pure domain now has typed UUID v4 identities for vaults, service credentials, folders, memberships, recovery sets/codes and preferences. Every persisted entity carries UTC creation/update timestamps and explicit active/trashed state; trash has no expiry. Edits replace a validated credential while retaining its identity, creation/deletion state, local icon and relationship IDs. A backward metadata update fails before mutation. These are unlocked memory records, with no file writer or plaintext serialization. Encryption metadata belongs to task 03's envelope, not to account fields.

`Vault::validate` is mandatory after authenticated DTO restoration and before commit: schema version, unique typed IDs, valid references/pairs, normalized bounded fields, timestamp/deletion/used-state ordering, preference ranges and collection limits. Each entity vector and the total recovery-code count are capped at 10,000. This does not replace task 03's byte/allocation limits before deserialization. Account service/account fields are bounded at 256/320 UTF-8 bytes; website at 2,048 bytes. Website validation accepts HTTP(S) with a host, rejects embedded credentials, malformed percent escapes, controls, whitespace/backslashes and repaired empty authorities. No browser or network access occurs. Six folder colors and six sorts are explicit enums; default preferences are list/latest first, startup off, lock 300 seconds, clipboard 30 seconds.

OTP uses pinned `totp-rs 6.0.0` with defaults off and only `alloc`/`zeroize`; its maintained HMAC/SHA/truncation/output primitives generate SHA1/256/512 codes. Domain enums restrict digits to 6/8 and periods to 15/30/60/120. Defaults are SHA1/6/30. The builder's `build_noncompliant` is intentional only to import existing nonempty legacy seeds shorter than RFC 4226's recommended minimum, rather than reject common service QR credentials. The domain independently enforces all other generator preconditions. No new seeds are generated and HOTP accounts are excluded. `current_code` reads injected UTC seconds once per call, recomputes without a cache, reports 1..period remaining seconds and uses checked next-boundary arithmetic (`None` beyond u64). Pre-epoch `SystemTime` returns a safe clock error; timezone never enters the API. UI resume/clock-discontinuity guidance remains the later lifecycle task's responsibility.

Base32 uses pinned `data-encoding 2.11.1`, accepting lowercase/uppercase and only ASCII space/tab/CR/LF removal. Optional padding must be the exact RFC 4648 terminal padding; concatenated padded blocks, aliases, Unicode spaces, invalid lengths/alphabet and nonzero tail bits fail. Raw secret text is capped at 4,096 bytes; decoded seeds at 1..1,024 bytes. Normalized and partially decoded owned buffers zeroize on failure/drop. Seeds/current codes/recovery values use explicit exposure accessors and redacted Debug. `zeroize 1.9.1` is a best-effort owned-buffer measure, not a guarantee about caller, allocator, HMAC internals, OS or clipboard copies.

The local URI parser deliberately avoids permissive URL repair. Input is ASCII URI syntax (Unicode must be UTF-8 percent encoded), at most 8,192 bytes, with exact `otpauth://totp` authority and a nonempty label/query. Percent escapes/UTF-8 are strict; '+' stays literal. Scheme/type are case-insensitive. Parameter names are case-sensitive; duplicate decoded names/case aliases fail. Only secret/issuer/algorithm/digits/period are supported; unknown parameters fail explicitly. Algorithms accept case-insensitive SHA1/SHA256/SHA512; digit/period spellings are canonical decimal. A supplied setting is never silently replaced. Query issuer wins only when it agrees exactly with any trimmed label prefix; conflicting issuers fail for manual review. Prefix/query issuer yields service plus account label. Without either issuer, the bare label becomes the service and account identifier is absent. Issuer/account components cannot contain another colon. The raw URI is never retained.

Duplicate warnings compare the decoded seed plus OTP settings, or trimmed case-insensitive service/account identifiers, within the unlocked vault. Matching includes trash and excludes the identity being edited. Results contain only record IDs and reason flags. No seed fingerprint, hash, external index or raw URI persists. Warnings neither block explicit Save nor silently overwrite a credential; later review/import UI owns presentation and confirmation.

Official sources checked on 12 November 2026, alongside downloaded crate source/API and Cargo feature inspection:

- [totp-rs builder](https://docs.rs/totp-rs/6.0.0/totp_rs/struct.Builder.html) and [secret API](https://docs.rs/totp-rs/6.0.0/totp_rs/struct.Secret.html): explicit settings and caller-supplied timestamps; zeroizing feature checked locally for Builder/Secret/Totp/Token.
- [data-encoding](https://docs.rs/data-encoding/2.11.1/data_encoding/): strict trailing bits and bounded mutable decode; extra terminal-padding rule prevents the library's supported concatenated padding blocks.
- [Zeroizing](https://docs.rs/zeroize/1.9.1/zeroize/struct.Zeroizing.html): owned data zeroed on drop. Aggregate records add their own redacted Debug because Zeroizing's Debug is not a redaction boundary.
- [Google Key URI format](https://github.com/google/google-authenticator/wiki/Key-Uri-Format): issuer/label and TOTP parameters; unsupported formats are rejected rather than guessed.
- [RFC 6238](https://www.rfc-editor.org/rfc/rfc6238) and [RFC 4226](https://www.rfc-editor.org/rfc/rfc4226): published deterministic vector fixtures only. [URL parsing API](https://docs.rs/url/2.5.8/url/struct.Url.html) is used solely for optional website validation, with stricter raw-input checks.

Task 02 implementation evidence is in `docs/qa/task02/implementation-evidence.md`; independent acceptance remains the orchestrator's separate review. There are no task 02 UI/screenshots or real credentials.
