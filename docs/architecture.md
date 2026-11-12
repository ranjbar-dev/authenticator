# Architecture and concrete decisions

## Task09 account form and save intents

Review09 correction: `OperationCancellation` uses atomic REQUESTED/ADMITTED bits
to linearize cancellation against admission of one final replace action. Cancel
is nonblocking. A separate mutex serializes background actions sharing a token;
cancel never acquires it. Tokens remain cloneable/defaultable/reusable until
cancelled, and an unwind guard clears admission after success/error/panic. A
cancel winning admission prevents the action; an admitted action may subsequently
complete or fail under the existing atomic file semantics. Admission is not the
durable commit point: native rename success still is. Lifecycle/idle guards are
checked just before admission. No schema/crypto/envelope change is involved.

Shell clears its UI-owned review/form/widget buffers before cancelling state;
state invalidates generation/query/intent first. Cancellation and polling use
nonblocking completion-slot access. Any late result is checked/locked before UI
adoption, so UI Lock does not wait for admitted filesystem I/O. Workers recheck
cancellation after publishing/releasing the slot, closing the race when UI cancel
could not acquire it. Cancellation does not interrupt native I/O or Argon2, or
force-clear a worker-owned vault, DEK,
candidate or scratch while it is using them. Those owners are released/locked
when the worker returns; framework/allocator/OS copies remain outside guarantees.

`ui::import_review::Editor` now renders the shared account field tree for QR,
manual creation and stable-ID editing. `ReviewMode` distinguishes these operations;
the Account edit holds an ephemeral original draft/state for conflict checking,
starts with a blank replacement-seed buffer and preserves the existing seed unless
Change secret is deliberately enabled. The save worker verifies the complete
original snapshot, including same-second metadata/settings/seed changes, and
rejects missing/trashed targets. Edit updates only that ID, retaining creation,
icon, memberships and existing recovery sets/used states. New duplicate entries
remain explicitly separate; an edit's duplicate warning excludes itself.

`AppState::begin_save(SaveIntent)` generalizes the existing sole-owner cancellable
worker; `begin_import` delegates to Import intent. Create can add an account and
new recovery set atomically. Edit and approved QR batches use the same detached
whole-vault validation/encryption/rename gate and stale lifecycle handling. No
repository/key clone, second committed-data cache or format migration is added.
Ordinary errors retain the unlocked owner and form; lock discards late results.

`domain::recovery_input::RecoveryInput` provides Task15's reusable text rules:
optional initial UTF-8 BOM, LF/CRLF, <=1MiB, trim before domain validation, omit
blanks, <=10,000 nonblank lines and <=4096 bytes per value. Exact duplicate removal
requires explicit review tied to unchanged input. New codes begin unused. Pure
`BackupCode::normalized_value` avoids creating random IDs during preview. Account
Edit does not implicitly replace a recovery set; its standalone editor remains
Task15. Details, actual relaunch/HMAC proof and review limits: task09 QA evidence.

## Task08 editable review and atomic import

The existing ROOT shell now completes capture → bounded QR decode → explicit
candidate selection → editable review → encrypted batch save → committed account
and current code. `ui/import_review.rs` owns only ephemeral zeroizing form strings,
selection and safe errors. Multiple supported candidates start unselected; one
supported candidate still requires review and explicit Save. Unsupported results
never carry raw payloads into the editor. Service, optional website/identifier,
Base32 secret, algorithm/digits/period are revalidated for every selected editor
at Save, including editors not currently visible. Supported URI settings survive.
Secret fields are masked by default, reveal is explicit, and form exits/lock drop
owned buffers and native editor/undo state. Framework/allocator/OS copies remain
outside complete erasure guarantees.

Save creates **separate new entries only**. Existing records, including trash,
are never overwritten. `AccountDraft::duplicate_criteria` shares the same seed+
settings and Unicode-lowercased service+identifier warning policy between the
vault and approved batch. Warnings are nonblocking and disclose no secret.
The Save label and explanatory text make creating separate records explicit.
Future account editing/backup merge must implement their own deliberate policies.

`AppState::begin_import` rejects locked, capture/image/worker overlap, empty or
oversized batches and invalid drafts before handing its sole repository to one
worker. No store/key clone is created. `transaction_cancellable` applies the whole
batch to a detached candidate and uses the existing authenticated codec and atomic
ciphertext writer. Its atomic cancellation admission and lifecycle/idle guard are
checked before the final replace action. Before admission, cancellation leaves
committed memory/disk unchanged. Once action admission has won, lock/cancel can leave the **whole**
encrypted batch committed; no false rollback promise or postcommit deletion exists.
Generations/cancellation reject late results and lock the worker-owned repository.
Recoverable write conflicts/failures return the unlocked repository and retain the
editable form for correction/retry. Panic fails closed. Busy UI removes editor
widgets and repeat Save, keeps Lock/Cancel available, and describes atomic timing.
Error feedback and the successful imported account/code scroll into view.

`platform/image_picker.rs` uses legacy `GetOpenFileNameW` on its own worker, with
no owner that would disable ROOT. No COM apartment is needed for this API. Its
native hook timer closes this dialog on cancellation/lifecycle change; paths are
consumed by `qr::decode_file`, never logged or retained in application state.
OFN_DONTADDTORECENT avoids adding the image to Recent Documents. Native OS dialog
history and dependency/OS allocations are not guaranteed erased. Reading/decoding
also stays off the event thread. Generation/watch/cancel checks discard stale
results; PNG/JPEG use task06's unchanged bounds. Manual entry starts the same blank
editor as a basic fallback; task09 extends account editing and recovery-code input.

Code cards read committed repository identities, metadata and TOTP settings; no
business fixture exists in production. Imported IDs order the actual new entries
first. The code uses the domain clock/engine and ordinary, non-live accessibility
text rather than repeatedly announcing OTP changes. Task10 still owns the full
search/sort/views/copy/list feature set. Argument-free `import_flow` and
`qr_batch_fixture` examples create only owned disposable fixtures. The former's
original PNG image input is separately generated in an owned temporary directory;
it is **not a screenshot** and is removed on example exit. Its F4 conflict hook
is diagnostic only and never touches the default vault.

See `docs/qa/task08/implementation-evidence.md` and `independent-review.md` for exact
checks, sanitized evidence and the distinct functional approval. Offline-with-network-disabled,
physical mixed-DPI, real OS lock/suspend and reference pixel fidelity remain open.

The parent checked Microsoft's current [GetOpenFileNameW contract](https://learn.microsoft.com/en-us/windows/win32/api/commdlg/nf-commdlg-getopenfilenamew),
[OPENFILENAMEW members/flags](https://learn.microsoft.com/en-us/windows/win32/api/commdlg/ns-commdlg-openfilenamew)
and [hook callback](https://learn.microsoft.com/en-us/windows/win32/api/commdlg/nc-commdlg-lpofnhookproc).
The owner may be null and the Explorer hook is supported. This legacy API remains
supported, while Microsoft recommends Common Item Dialog for new applications.
`OFN_NOCHANGEDIR` is not a guarantee for this Open API; production vault/image
operations use absolute paths and must not rely on the process working directory.
The Recent Documents flag is not a promise to erase every OS dialog history/cache.

## Task07 selected-region Windows adapter

`platform/capture.rs` now extends the virtual-desktop foundation with checked
physical `Point`/`Region`, monitor enumeration, a cancellable worker and closed
capture/decoder outcomes. `capture/windows.rs` uses a dedicated PMv2 thread;
signed WM_MOUSE event client coordinates become physical desktop pixels through
ClientToScreen. GetCursorPos only chooses the initial instruction-panel monitor:
its latest cursor position must not replace queued button-event coordinates.
Controls convert theme logical pixels using GetDpiForWindow/96, independently of
the main egui scale. Either drag direction clips against signed virtual bounds.
To avoid signed 16-bit mouse-message wrap, virtual desktops beyond 32,767 pixels on an
axis fail closed. This limit does not expand task06's selected-image cap.

The overlay paints a translucent geometry-only scrim, an opaque border and an
opaque themed native instruction panel with bundled Noto Sans and a real named
Cancel button. SetCapture/ReleaseCapture and focus/cancel/display/DPI handling
live on that native thread. No secondary egui viewport accessibility is assumed.
Main ROOT renders a nonsensitive preparation frame and AccessKit tree before
hiding; query/count/review widgets remain absent while selecting/decoding.
Restoration follows lifecycle and generation processing, never a native worker
blindly re-showing an old unlocked framebuffer.

On approval, validate nonzero sides<=4096, total<=4,194,304 pixels and four-byte
size<=16,777,216 before any bitmap/owned pixel allocation. Hide/destroy the
overlay and DwmFlush its queued changes, compare monitor snapshots before
allocation/BitBlt, and capture only that RECT using a top-down 32-bit DIB. Reserved
GDI alpha becomes 255 before the owned zeroizing packed BGRA buffer enters the
task06 decoder. There is no desktop-background screenshot, screenshot file or
clipboard path. RAII restores GDI selections, clears mapped DIB bits after
GdiFlush, releases DCs/bitmaps/fonts/windows and releases pointer capture.

`LifecycleWatch` adds a non-consuming atomic epoch/health watch beside task05's
pending UI flag. A25ms native timer observes it, explicit operation cancellation,
the existing session idle deadline and owner shutdown even while ROOT is hidden.
Native pointer activity does not extend that deadline. Queued review publication
and cancellation share a mutex; producer and consumer recheck cancellation/native
watch, and AppState also checks its session generation/unlocked gate. Decode is
bounded/cooperative, not forcibly interruptible. AppState owns the single task and
redacted ScanReview, clears it on lock/cancel and exposes `take_scan_review` for
task08. Safe image/manual intents are handed off without pretending those editors
exist. No storage-format/import operation was added.

Real public-QR selection succeeded on primary and negative-coordinate 96 DPI
monitors, including reverse drag, plus actual Cancel/Escape/empty-region recovery.
Private HWND notification/pointer-release tests cover lifecycle routing without
changing OS state. Physical 150/200%/mixed-DPI, real hot-unplug/permission-denial,
OS desktop lock/suspend, full accessibility and task08 import remain open. Exact
commands, safe evidence, Win32 sources, geometry and memory/driver limits:
`docs/qa/task07/implementation-evidence.md`. Independent review is pending.

Native selector typography deliberately keeps GDI/OS row metrics for its real
STATIC/Button controls; ROOT's complete egui typography bundle remains unchanged.
`native_font` consumes role size/family/`theme::Weight`; regular resolves to private
**Noto Sans / 400**, semibold to the bundled Win32 family **Noto Sans SemiBold / 600**.
The regular-family request at 600 was measured resolving 700, so the explicit
semibold family prevents that synthesis. A private memory-DC probe measures the
real realized font/extent without screen capture or OS font installation. At
requested 96 DPI both cells are 19 pixels (ascent 15/descent 4, internal leading 5,
external leading 0), with zero native tracking. Actual instruction block 375×57
fits 528×72 and Cancel 102×19 fits 528×36. This is an intentional native-kind
exception to ROOT BODY/LABEL line-height 20 and future nonzero tracking, rather
than a claim of full bundle equivalence. See task07 evidence for exact commands
and the focused review resolution; physical mixed-DPI gates remain open.

## Task 05 local vault lifecycle

The ROOT shell now creates and unlocks the real local repository with exact
12..1024 UTF-8 byte passphrases and setup confirmation. No app registration,
optional device protection, network service or reset key was added. Existing
unreadable ciphertext always selects Unlock, never initialization.

`AppState` transfers its sole `VaultStore` into one worker during passphrase
work. It does not clone the repository or retain another decrypted view model.
A mutex-protected completion slot, cooperative cancellation token and session
generation prevent a cancelled/old completion from unlocking the UI. Lock clears
query/pending intent, dialogs/status, form strings and native widget memory.
Secret editors clear their undo history after every pass; the shared focus helper
scrolls newly focused controls into view. Form failure/clear/completion/close also
release form buffers. History allocations are dropped, not claimed securely
erased by egui.

Argon2 runs off the paint/event thread. It cannot be safely interrupted mid-call;
its bounded scratch/passphrase/temporary keys drop when that call returns. The
UI remains cancellable and never adopts a cancelled unlocked result. A completed
but unpolled result is locked when its slot is available without waiting;
generation/token checks also reject it before adoption. Closing requests cancel
without waiting for the KDF/filesystem worker. The former shared cancellation/
rename mutex was corrected by Task09 review: atomic admission now prevents work
if cancel wins, while an admitted final replace may leave a complete encrypted
vault requiring unlock. Active worker-owned key/plaintext lifetimes are not
forcibly interrupted. Native rename remains the commit point; format is intact.

Idle expiry uses `Instant`, checks before recording new input and requests a
200ms repaint tick. Default remains 300 seconds; Settings saves 60/300/900/1800
seconds inside the authenticated encrypted preferences. The diagnostic example
can explicitly save 3 seconds to its isolated vault for a real expiry check.
UTC wall time remains only a domain record timestamp, never the idle clock.

`platform/lifecycle.rs` owns a dedicated message thread and unshown **top-level**
HWND, current-session WTS registration and suspend/resume registration. It handles
WTS_SESSION_LOCK and WM_POWERBROADCAST suspend/resume notifications, requests a
repaint and makes the next UI pass lock before processing worker results. A
message-only HWND is deliberately not used. Registration failure or unexpected
notification-window destruction keeps the UI locked and asks for restart. Drop
closes/joins the listener and unregisters native resources. The tests send only
to that private HWND; actual desktop lock and hardware sleep were not performed.

`examples/vault_lifecycle.rs` is an argument-free, explicit isolated preview under
the OS temporary directory. Its ciphertext survives relaunch, no existing vault
is silently removed and it never resolves the default user vault. Its size tools
compensate for app zoom; they are not physical DPI tests. Task05 evidence and
remaining manual/hardware limits are in `docs/qa/task05/implementation-evidence.md`.

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

## Storage and recovery decisions (task 03 implemented; task 11 backup pending)

Task 03 now implements `VaultStore`: a random 256-bit DEK wrapped under an Argon2id-derived passphrase KEK; RustCrypto argon2 0.6.0 and chacha20poly1305 0.11.0, both with zeroize enabled. OS randomness comes from getrandom 0.4.3/Windows ProcessPrng. All domain entities, IDs, timestamps, seeds, recovery codes, trash and preferences are inside authenticated ciphertext. Exact binary encoding, AAD, calibration, bounds and honest limitations are specified in [storage-format.md](storage-format.md). No UI integration is claimed until task 05.

The local format discriminator is AUTHLOC1, envelope version1 and current encrypted payload schema2. The bounded custom codec has no general-purpose deserialize allocations: file/plaintext cap64MiB plus envelope/tag, per-field byte limits, collection/count/minimum-byte checks, total recovery count and whole `Vault::validate` before session installation or commit. Schema1 migrates explicitly in detached memory to2 with empty query/All folder; unlock never writes, and the next successful atomic transaction serializes2. Schema0/future/corrupt schemas fail without writes. Never initialize over an existing unreadable/corrupt file. Task11 must define a distinct independent backup format/passphrase/keys; reference `.vault` compatibility remains Task22 discovery.

Windows SHGetKnownFolderPath resolves per-user LocalAppData/Authenticator. `vault.bin`, empty `session.lock` and random pending files use protected current-user-only DACLs; failure is fatal. A lifetime OS instance lock and exclusive mutable repository API serialize access. Transactions edit detached clones and preserve prior memory/disk on cancellation, invalid aggregates, panic and precommit failures. A ciphertext snapshot rejects unexpected external edits. Ciphertext temporary files are flushed before same-volume MoveFileExW replacement; rename success is the commit point and no fallible postcommit step reports a false rollback. Real subprocess termination and native sharing-violation tests verify preservation. Hardware power-loss durability is not certified. Tests use only disposable tempfile directories, never user AppData.

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

## Task 06 local QR service

`src/qr.rs` is a synchronous local service with no UI, capture, vault/store access,
clipboard, logging, network, icon lookup or writer. It accepts an explicitly
selected path through `decode_file`, owned encoded bytes through `decode_image`,
or owned selected-region pixels through `decode_pixels`. Byte/pixel arguments
are `Zeroizing<Vec<u8>>`, consumed even on failure. No source path, filename,
image, raw URI or unvalidated text is returned. It does not open the default
profile or read any vault. The existing storage transaction API remains unchanged;
this service has no save/import operation.

| Boundary | Exact limit and behavior |
|---|---|
| Formats | Content-sniffed PNG and JPEG only; extensions ignored; image defaults/rayon/other format features disabled. PNG palette/low-bit data expands; 8/16-bit grayscale, gray-alpha, RGB and RGBA output accepted. PNG's initial image only; no animation traversal or metadata/orientation application. QR quarter-turn rotations are handled by recognition. |
| Encoded input | 8,388,608 bytes (8 MiB). Path metadata rejects nonregular files/symlinks and overcap lengths before open; file handle metadata must also describe a regular file within the cap, then a `take(cap+1)` read catches growth. A single cap+1 reserved, zeroizing buffer prevents growth reallocations of the app-owned file copy. No file content is changed or copied to another path. Filesystem I/O timing itself is OS-controlled; this is not a file-open timeout guarantee against a malicious concurrent path replacement or unresponsive device. |
| Dimensions | Both nonzero and at most 4,096, with checked multiplication and at most 4,194,304 pixels. PNG IHDR and JPEG segment/SOF dimensions checked before creating a format decoder; decoder dimensions must agree. JPEG segment ranges are checked against the bounded input before traversal. |
| Decoder allocation | At most 33,554,432 bytes (32 MiB) in `image::Limits::max_alloc`, set before `into_decoder` (including PNG header/ancillary handling). Width/height limits are strict; max_alloc is an upstream best-effort decoder budget, **not** a total process-memory limit. The app independently checks decoded `total_bytes` before allocating its output; worst supported output is 8 bytes/pixel. Encoded copies, grayscale, QR workspace and codec scratch are additional bounded buffers. |
| Capture layout | Tightly packed top-to-bottom `Gray8`, `Rgb8`, `Rgba8` or `Bgra8`; exact length = width × height × channels, no row padding/stride, premultiplication or orientation flag. Straight alpha composites over white. The task07 adapter must convert padded/bottom-up GDI data and set its unused alpha byte to 255; passing GDI's reserved zero byte as alpha would whiten the image. The producer must enforce these dimensions/bytes **before** capture allocation as well. |
| Recognition | `quircs 0.10.3` internally caps regions at 65,534, capstones at 32 and grids at 8; grouping is limited by those capstones. Recognition runs once over bounded pixels. At most eight extracted grids are collected in a pre-reserved buffer using `take(8)`; saturation (8 grids, 32 capstones or 65,534 regions) fails the whole scan before any payload decode. Review maximum is 7 codes. Saturation may conservatively reject a busy image; retry with a smaller region. Never offer a silently truncated batch. |
| Payload | At most 2,048 decoded bytes per code, strict UTF-8, no controls; then the existing strict `parse_totp_uri` runs without changing its URI policy. Quircs' own payload assembly is independently bounded at 8,896 bytes before the service's tighter check. No library error text is formatted. |

`ScanReview::candidates` preserves per-code outcomes: `Supported(AccountDraft)`,
`Unsupported`, `MalformedCredential(AppError)`, `InvalidPayload`, or
`DecodeFailure`. Unsupported schemes/types have their own outcome; malformed
TOTP settings/secret/issuer/unknown parameters preserve the parser's safe fixed
error. No detected code returns `QrError::NoQr`; bad image data returns
`DecodeFailure`, distinct from format, file, size and candidate-limit errors.
A damaged detected QR can coexist with a valid draft. Review and every candidate
have completely redacted Debug, including metadata. Draft secrets and metadata
already zeroize on drop through task02's owners. `requires_choice` is true when
more than one supported credential exists, including repeated QR credentials;
there is no default selection, deduplication, automatic save or import.

App-owned encoded, decoded-image, grayscale and recognition pixel buffers zeroize
on exit. Extracted grid bitmaps have a clearing Drop wrapper; successful decoded
payload ownership transfers immediately into Zeroizing and drops after parsing.
Dependency internals are an explicit limitation: `image`'s JPEG decoder copies
encoded input; PNG metadata/scratch and JPEG codec scratch are library-owned;
quircs allocates/reallocates payloads internally and may drop partial payloads
on decode failure, with non-zeroizing stack/ECC/flood-fill/geometry scratch.
Copies moved by Rust/compiler/allocator and OS pagefile/dumps also cannot be
guaranteed cleared. Nothing is persisted by this service, and no complete memory
erasure claim is made. Defensive `catch_unwind` maps upstream decoder assertions
to the fixed decode error; the app's existing panic hook suppresses payload
printing. This is not a sandbox or a hard wall-clock deadline. CPU/input work is
bounded by encoded bytes, pixels and detector/candidate caps, not interruptible
mid-codec. Later tasks should run it off the UI thread and discard stale results.

Task07 owns actual capture and cancellation. Task08 must require an unlocked
session and explicit selection/editable review/Save, use the existing generation
guard on lock/cancel to discard pending results, and commit only selected validated
drafts atomically through storage. The service alone does not enforce vault state.
No UI/hardware acceptance is claimed in task06.

Dependency/API review on 12 November 2026 used downloaded pinned source plus the
official [image API](https://docs.rs/image/0.25.10/image/),
[decoder limits](https://docs.rs/image/0.25.10/image/struct.Limits.html),
[image repository](https://github.com/image-rs/image),
[quircs API](https://docs.rs/quircs/0.10.3/quircs/) and
[maintainer repository](https://github.com/dignifiedquire/quircs).
`image =0.25.10` with only png/jpeg resolves png0.18.1, zune-jpeg0.5.15 and
zune-core0.5.3; `quircs =0.10.3` supplies existing QR detection/decoding primitives.
`qrcode =0.14.1` is dev-only, defaults off, for disposable in-memory fixtures.
Do not copy upstream README examples that print QR payloads or open arbitrary
paths through a global image loader. Source inspection verified allocation,
region/capstone/grid/payload limits and identified the memory-copy limits above.
`cargo audit --json` using RustSec database commit
`af7b3ec33460b8e2de8d422e16d628518891a519` reported zero known vulnerabilities;
three existing unmaintained notices concern zbus_names/zvariant/zvariant_derive
in the Unix accessibility dependency tree, not the new decoding path. No quircs
advisory matched the current RustSec listing. This is a point-in-time dependency
check, not certification of decoder security or future maintenance.
## Codes list and clipboard (Task10)

`ui/codes.rs` derives a cached list of stable account IDs from the single unlocked
Vault; it does not own credentials or clone a store/DEK. Only active accounts are
shown. Search and metadata sorts use locale-independent Unicode16 full case
folding (`caseless0.2.2`), without accent removal, normalization or email aliases.
Latest/Earliest use creation time, not edit time. All ties use ascending stable ID;
missing account identifiers sort last in both account directions. All/Unfiled/
existing-folder navigation reads actual many-to-many memberships. A recently saved
ID receives a highlight/scroll target without overriding the requested order.

Codes and Backup Codes keep separate search buffers and widget identities. Only
the Codes query projects the encrypted Codes preferences; editing the other tab
cannot change its visible query or filtering. Both session buffers clear on Lock.
Query validation is derived inline from the current input, so correction removes
its error immediately without overwriting unrelated operation feedback.

The shared header retains Help, Lock, Screenshot to Scan and Add Account; photo
input stays local. Shared native labeled choices provide all six exact reference
sort labels and List/Grid. Grid uses two columns when the content holder has600
logical points, otherwise one readable column. The root scroll holder disables
content drag scrolling; wheel/bar/caret/focus scrolling and explicit separate
Copy/Edit/Delete buttons remain. Codes group6 digits3+3 or8 digits4+4 and use the
whole CODE typography role. Key fallback tiles use opaque BLUE/ON_BLUE vectors.

One wall-clock sample serves each frame. Only visible cards compute HMACs; cached
ID ordering is invalidated by list/filter changes and mutation completion. UTC
second repaint deadlines include every configured OTP boundary. A click samples
the clock again, copies raw digits and passes only period/counter validity metadata
to the clipboard actor. That actor checks the wall clock again immediately before
native mutation and text publication, rejecting a queued expired code rather than
receiving a seed. Clock failure removes values and disables Copy. A wall versus
monotonic delta >5seconds shows local date/time guidance; there is no remote time
verification or OS clock mutation. Resume/session events use the existing native
lifecycle gate, lock and request fresh state.

`App::logic` uses monotonic idle time and lifecycle signals while hidden/minimized;
it never consumes the stale UI events/time retained by pinned eframe0.37's public
`Context::run_logic` contract. On Lock the app requests exactly one public
RequestPaintWhileHidden scrub pass, without unminimizing/showing an ordinary hidden
window. Actual capture restoration still follows its full nonsensitive-frame rule.
Locked idle UI does not request a periodic repaint. The clipboard actor/timer is
independent of Root paint and native lifecycle events.

All display and timeout writes use the existing sole-store-owner cancellable async
SaveIntent worker. Query saves debounce500ms and coalesce edits made during a write;
completion never overwrites newer live input. Failure keeps the previous vault and
offers Retry. Hidden logic also services pending display saves. Normal Close flushes
the latest valid display preferences behind a nonsensitive closing frame. A failed
flush remains on that nonsensitive error screen with Retry and deliberate Close
without saving. That discard action still performs the bounded native clipboard
cleanup ACK stage while Root pumps owner messages. Ordinary successful close also
waits up to300ms in that pumping stage, followed by the at-most300ms on-exit wait;
an approved programmatic close is not intercepted again. A second forced close can
cancel the wait. OS/idle/explicit
Lock immediately clears UI/query/widget state and cancels outstanding operations;
uncommitted display changes may be lost on Lock, force-close, storage failure or
process termination. Already admitted complete encrypted operations retain the
existing honest cancellation semantics. New preference/Trash mutations clamp their
metadata timestamp to the aggregate update time after local clock correction;
OTP generation continues to use actual wall time.

Payload schema2 adds encrypted bounded query and stable folder-filter identity with
an explicit lazy schema1 migration; envelope1 is unchanged. See storage-format.md.
Confirmed Delete sets only the account's Trash state in one real atomic transaction,
retaining IDs, recovery values/used states and folder associations. Cancel is inert.
The full Trash screen/restoration/permanent deletion remain Task18. Task17 should
reuse FolderFilter, `Preferences::codes` and `Vault::set_codes_preferences`; deleting
the selected folder must reset that filter to All in the same transaction.

`platform/clipboard.rs` owns a bounded eight-message actor, zeroizing queued text,
Win32 global-memory ownership and a nonplaintext lease (HWND, random16-byte marker,
sequence and salted SHA256 of exact UTF-16 bytes/length). Native isolated tests
observed sequence advancement at CloseClipboard, so a fresh exclusive reopen
verifies owner/marker/content before sampling the established sequence. If that
reopen is Busy, conservative provisional evidence is retained without rebasing its
sequence; cleanup later still requires the exact sequence and all ownership proof
in one exclusive open. A post-close external replacement, including identical text,
cannot be adopted as an owned copy. No unrelated Unicode clipboard text is read.
Framework CopyText output is intercepted too; secret/recovery editors retain their
native password clipboard guard even when visually revealed.

Expiry/Lock/lifecycle cleanup opens with NULL (so Root HWND destruction cannot make
opening fail) and conditionally clears inside the same exclusive interval. Busy
clears retry off UI; after2seconds the actor warns and continues with only the
nonplaintext lease. Normal shutdown requests cleanup and waits at most300ms on UI;
the actor has a2second best-effort stop budget. Windows calls already admitted may
finish after cancellation or stall; process termination/OS denial can defeat final
cleanup. History/cloud opt-out formats are hints to Windows, not protection against
third-party clipboard observers. Full constraints and primary references are in
security-design.md and docs/qa/task10/implementation-evidence.md.

## Portable backup engine (Task11)

`backup` provides synchronous selected-path export, authenticated preview,
opaque final plans and consuming transactional apply for the existing sole-store
worker. It reuses crate-private reviewed KDF/AEAD and bounded codec primitives,
not the local envelope, local DEK, protected directory writer or session lock.
`AUTHPBK1` uses fresh independent randomness, a separately entered passphrase and
explicit portable AEAD domain separation. Exact bytes, migration, graph conflict
policy and filesystem limits are normative in
[portable-backup-format.md](portable-backup-format.md).

The preview binds authenticated incoming records to an exact committed local
ciphertext revision. KeepLocal/KeepBoth merge operates on complete incoming
account/membership/recovery bundles; all folder copies and cross-type ID remaps
are reviewable. Explicit whole Replace reports removed local counts. Local
vault/preferences identities and startup remain local. Display and security
preference import each require an explicit option. Final plans validate before
the existing atomic local transaction. Source-file changes and local revision
changes invalidate old plans.

All Vault aggregate mutation APIs now clamp metadata time to their current
logical aggregate update time. Incoming record creation/update/trash/used dates
remain truthful, while a future-dated imported snapshot no longer makes current
wall-clock account edits or creation fail. OTP uses actual UTC as before. Direct
recovery-code updates must use the aggregate logical time and update their set
and vault together (Task16). Tasks12/13 own all GUI/worker/session wiring and
post-import cached display/ID/code invalidation; this task adds no UI route.
