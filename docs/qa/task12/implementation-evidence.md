# Task12 implementation evidence

Author handoff for a distinct UI/security/data-loss reviewer. This is not an
acceptance certificate. Parent owns task/checkpoint/coverage statuses, commits
and pushes; those files were not edited by this child. Task13 is not implemented.

## Implemented behavior

Settings → Export & Import → Export encrypted backup uses the actual local
backup engine. Included-data guidance covers accounts, recovery/used state,
folders, Trash and preferences, with no displayed credentials. A native Windows
Save dialog selects a local `.authpbk` path. Both separately entered backup
passphrase fields use the existing concealed labeled password controls. Exact
12–1024 UTF-8 bytes are accepted without trimming, and guidance explains the
irrecoverable passphrase and always-enabled encryption.

The native dialog does not create a test file or ask for overwrite consent.
Destination inspection and bounded hashing run on the picker thread.
`Destination::review` returns an opaque `DestinationReview` without authorizing
a write. ROOT names the selected existing file in a modal and initially focuses
Keep existing file. Only Replace file and export approves that prepared target.
The original engine checks its exact bytes/absence at the atomic write gate.
Selection, mismatch, Keep existing, Escape, pre-admission cancellation and denied
access cannot overwrite an existing file. Changed targets require selection and
review again. Access/space guidance is actionable without raw OS errors or file
contents; `Unavailable`, `Changed`, `Limit`, `Cancelled` and `Locked` retain
distinct safe messages. The engine intentionally combines permission/space I/O
failures into its opaque Unavailable variant; the UI does not invent a precise
OS diagnosis.

Export transfers the sole existing `VaultStore` into the existing asynchronous
worker. No second decrypted store, local key copy, UI-thread KDF/hash/write or
network/cloud service was added. The worker preserves lifecycle/monotonic idle,
generation checks, nonblocking cancellation, `try_lock` publication and the
worker post-publication lock check. Busy removes editor/submit controls and
prevents repeated export. Cancellation locks immediately. An already admitted
whole encrypted write may finish; UI wording does not falsely promise no file or
rollback. Stale completion cannot publish success or reopen the session.

Back, close, Lock, idle and completion clear owned passphrase buffers and native
widget history. Close with dirty display preferences clears the export form
immediately instead of waiting for that unrelated save. Path/review/dialog
metadata drops at Lock. Picker Drop never waits for the native dialog, file
inspection or publication. If its original deadline expires after fresh ROOT
activity extends the session, a quiet metadata-free completion releases the busy
slot. Native I/O/KDF already in flight may retain worker buffers until return,
as in the existing documented memory/OS limits.

Changed source paths:

- `src/ui/export.rs`, `src/ui/export_tests.rs`, `src/ui/shell.rs`, `src/ui/state.rs`,
  `src/ui/mod.rs`.
- `src/platform/backup_picker.rs`, `src/platform/mod.rs`.
- `src/backup/files.rs`, `src/backup/mod.rs` (opaque pre-consent destination review;
  no envelope/crypto/writer-format changes).
- `examples/export_backup.rs` (argument-free, fresh scoped TempDir and actual
  encrypted fixture; never opens default/reference data).
- `docs/architecture.md`, `docs/ui-foundation.md`,
  `docs/portable-backup-format.md`, this report.

## Checklist and design decisions

The existing Task12 eight-line screen checklist remains the task specification.
The work was sized as a new native ROOT form, using the ui-consistency
finding-patterns/values/decisions/planning/implementing workflow in the parent's
existing sequential process. User authorization already covered implementation;
no approval stop or additional agent was added. Parent supplies the distinct
reviewer after this author's FINAL. Existing same-kind calibration reports are
`docs/qa/task04/calibration.md` and `docs/qa/task07/calibration.md`; they do not
certify new live pixel sensitivity.

Snapshot13 November2026: bound is the production `AuthenticatorApp` and imported
native UI modules. The existing ROOT shell/form is the reference; account-editor
fields are imported children, not a separate peer. Diagnostic FoundationApp,
galleries and examples are excluded from conventions. There is one production
shell, so its way is a shared precedent, not a manufactured majority. Grouped
exact-name search `c::(labeled_secret_field|heading|button|status|progress|card)`
plus `theme::(GAP|SMALL_GAP)` over `src/ui/shell.rs` and
`src/ui/import_review.rs` confirms reuse in both source files; the imported
account editor belongs to that shell. No new colors/fonts/scale or private
widget styling was introduced. Theme values and complete role bundles remain
in `src/ui/theme.rs`; no root DESIGN.md was found.

- Shared scroll holder/header: existing ROOT `CentralPanel`/`ScrollArea`, level1
  `c::heading`, shared Help/Lock, and explicit Back; as requested/shared piece.
- Summary: `c::body` and `c::secondary` with existing BODY/CAPTION bundles;
  includes categories without counts or secret samples; as requested.
- Destination: shared heading/body/button; full selected path wraps through
  `c::body`, real Save adapter; new native Save role explicitly requested.
- Passphrases: two `c::labeled_secret_field` controls; labels and helper
  descriptions, native PasswordInput semantics, pre-layout mask and per-pass
  undo clearing; shared piece over private copy.
- Recovery/encryption copy: shared secondary text; no switch/plaintext/cloud;
  requested scope overrides excluded reference controls.
- Overwrite: existing `c::Dialog`, named path, Keep existing initial focus,
  deliberate Replace file and export; shared piece, exact prepared target.
- Busy/cancel: existing `c::progress` and buttons; no submit/field widgets while
  running; Cancel and lock uses the existing secure cancellation boundary.
- Feedback: shared status/body, one-shot error scrolling, safe success path,
  Back/focus restoration and secret clearing; shared piece.

Exact reference screenshots are unavailable (`docs/reference-ui.md`), so the
PRD inventory/shared theme is the authorized fallback. Estimated reading cost:
about30 project files, about20 grouped searches, and the five named skill files
plus conventions, checklist/words/joining-a-process and planning support. These
are estimates, not a new UI convention count.

## Tests and checks

All fixtures are generated disposable vaults or public RFC seeds. No real/default/
reference vault, credentials, backup or clipboard data was read. No raw secret,
passphrase, QR, code or user record was written to evidence/logs/screenshots.

Meaningful new tests use the actual form validation, real store worker and engine:

- Mismatch, short/oversized phrases and exact Unicode byte bounds; no write.
- Both full-node AccessKit password values concealed and labeled at340/600/960
  logical widths; Lock removes the form/path and password nodes.
- Complete exported authenticated incoming codec equals the source vault,
  including identity/metadata, folder/membership, Trash, recovery used state and
  encrypted preferences. Ciphertext lacks public sentinels, seed and backup
  passphrase. Task11 restores into an independently keyed empty local store.
- Local vault ciphertext remains unchanged by export; repeated activation/worker
  overlap cannot produce another file.
- Existing-file inspection itself grants no write; mismatch/changed destination
  preserve existing bytes. Missing parents fail safely with actionable guidance.
- Actual Windows exclusive sharing denial returns Unavailable and preserves
  the existing file. This is a real denied-access condition, not an asserted
  physical disk-full test.
- Synthetic egui pointer press/release dispatch exercises ROOT named overwrite,
  initial safe focus, Escape/Keep existing, explicit replacement, hidden busy
  fields/submit, success and Back. These are synthetic framework events, **not
  real OS pointer or native picker evidence**. First modal Area measurement
  requires a subsequent visible pass; tests inspect that actual Context state.
- Real worker cancellation before admission preserves file absence; held
  post-admission export proves UI Lock/AX clearing before release, then a whole
  file can finish without late UI/session adoption.
- Private own-listener lifecycle notification, hidden monotonic idle and close
  with pending preferences clear/fail closed without OS lock/suspend automation.
- Expired picker publishes a quiet metadata-free completion; picker cancellation
  never waits on a held publication slot.
- Synthetic Tab/Enter/Shift+Tab/Ctrl+A/Paste events prove Submit retains the same
  actual egui/AX ID and keyboard focus when mismatch feedback appears and is
  corrected. Default Enter on Keep existing restores Submit focus and preserves
  the file. Parent identified the conditional-holder risk; the error holder now
  stays mounted even when empty. This is an event/focus regression, not a claim
  of native keyboard input. The test sends both Windows ctrl and egui command
  modifier bits for Select All, as the framework expects.

Commands/results during implementation:

- `rtk cargo check --locked --lib`: passed. An initial unused opener warning was
  resolved by restoring the opener on Back.
- `rtk cargo test --locked --lib export_`: initial5 passed/25.92s; expanded8
  passed/26.66s (seven Task12 tests plus one existing engine test).
- `rtk cargo test --locked --lib export_synthetic -- --nocapture`: passed1/12.03s
  after making the test wait for egui's visible post-measurement modal pass.
- `rtk cargo test --locked --lib export_denied -- --nocapture`: passed1/10.23s.
- `rtk cargo test --locked --lib ui::export::tests`: passed2/0.02s after the
  picker deadline completion correction.
- `rtk cargo test --locked --all-targets --all-features`: intermediate **125
  passed**, four directly ignored subprocess helpers exercised by parents,
  15 suites/76.32s. This preceded the final picker deadline correction/tests.
- Subsequent full picker-corrected run: **127 passed**, four helpers directly
  ignored/exercised by parents,15 suites/76.47s. This preceded the parent-requested
  stable error-holder/keyboard-focus regression.
- `rtk cargo test --locked --lib export_keyboard -- --nocapture`: passed1/6.54s
  after supplying the native-platform Select All modifier bits in the synthetic
  input. No secret input values were printed.
- Strict all-target/all-feature Clippy passed after replacing deprecated Id::new
  in new tests with Id::unique. Fmt/diff checks and explicit production/lib/owned
  export example build passed. Definitive final checks follow below.

## Native UI attempt and open acceptance gates

Read the bundled computer-use skill plus guidance/API/confirmations; initialized
the documented `@oai/sky` through `mcp__node_repl__js`. Normal own executable:
`D:\Projects\github\authenticator\target\debug\examples\export_backup.exe`.
First `sky.launch_app` returned `GetCursorPos failed: Access is denied.
(0x80070005)`. Fresh `sky.list_windows` filtered by the exact owned preview title
returned no window. One further documented normal launch returned the same
error; final fresh inventory again returned no owned window. No stale input,
permissions/settings change, private protocol, elevated launch, unrelated
preview interaction, OS input fallback or further retry was used. The OS cause
is unproven.

Consequently there are **no native Task12 screenshots or real pointer/keyboard
claims** in this handoff. Synthetic Context/AX checks are not actual framebuffer
or visual-quality proof. Independent real Save-dialog selection/cancel/Lock,
340/600/960 screenshots, actual key/focus/overwrite/busy/error/success journey,
long-path readability and new-user completion remain open. The reviewer can
launch the owned example normally if the helper becomes available. It starts
unlocked in Settings with a fresh full encrypted public fixture and exposes only
its owned destination in a collapsible preview panel. F6/F7/F8 resize340/600/960;
F10 captures its actual framebuffer into its TempDir, redacting any code bounds
before writing. No example debug hotkey exists in production. Public disposable
local passphrase is in the example source; use any separate disposable backup
passphrase for the real form. Select only the displayed owned TempDir path.

Task13's actual import GUI cross-gate remains open. Task11 engine restoration in
the new tests is meaningful evidence without claiming the not-yet-built13 screen.
Physical disk-full, external Windows user/device, removable media, hard power
loss, actual OS lock/suspend, hardware DPI/Narrator, disconnected-machine and
reference-pixel gates remain unperformed. Prior Task10 Copy and05/07/08 hardware/
offline gates are unchanged.

Cleanup: final own-title inventory and `Get-Process -Name export_backup` found
no owned process/window, and TempDir prefix inventory found no Task12 fixture.
No locator, retained profile, backup, screenshot or helper script remains. The
preexisting unrelated lifecycle preview was not touched.

## Definitive final checks

After the stable feedback-holder/focus correction and final picker tests:

- `rtk cargo test --locked --all-targets --all-features`: **128 passed**, four
  directly ignored subprocess helper entry points exercised by their parents,
  15 suites/76.60s.
- `rtk cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
  A final lint initially caught the new picker test module before production
  items; moving that module to the end resolved it without suppressions.
- `rtk cargo fmt --all --check`: passed.
- `rtk git diff --check`: passed.
- `rtk cargo build --locked --bin Authenticator --lib --example export_backup
  --all-features`: passed. The production app was not launched. All-target
  standalone build was not repeated against the existing unrelated running
  lifecycle example artifact; the parent's known LNK1104 ownership gate remains.

Ready for the parent's distinct read-only UI/security review, with the native
pointer/picker/screenshot and Task13 GUI cross-gates above explicitly open.

## Parent correction after independent review

Distinct review_12 proved that entering export during the500ms display-preference
debounce froze the pending query/view/sort/filter while the form blocked normal
persistence. A real worker export then contained the previous stored preference.
Parent added a guarded preparation stage: save the pending valid projection
through the existing asynchronous mutation worker, wait for success, then start
the encrypted export from that current store. The original destination review
and explicit overwrite consent remain bound throughout. Newer coalesced changes
are saved before proceeding rather than silently using an earlier snapshot.

Failure stops before backup writing, retains pending display intent for retry
and clears backup passphrases with actionable feedback. Preparation hides fields/
submit and offers Cancel and lock. Unexpected repository locking also drops the
queued form/path/widgets. Three regressions cover current query/sort/view/filter
in the actual exported full codec, failed local save with unchanged ciphertext/
no backup then successful retry, and previously rendered form/path removal after
unexpected locking. These are worker/engine plus synthetic Context tests, not
native pointer or visual evidence.

Without pending display changes, export still leaves local ciphertext unchanged.
Saving an earlier requested display change is a separate authorized transaction:
if a later export fails/cancels, a completed preference save is not falsely rolled
back. Both stages use the existing cancellation/atomic commit boundary. Export
does not mutate account/recovery content. During preference preparation the form
owns the concealed phrases until encryption starts; Lock/exit/preparation failure
clears them without waiting for the worker.

Focused checks passed1 each: latest preferences15.39s, failure/retry15.56s and
unexpected repository lock6.17s. Final corrected full all-target/all-feature suite
**131 passed**, four directly ignored helpers exercised by parents,15 suites,
76.52s. Strict Clippy, formatting, diff and production/lib/export-example build
passed. Original source was rebuilt after the reviewer's external probe used the
shared Cargo target, so these counts refer to the corrected repository source.
Focused independent re-review follows in the parent report. All native UI,
rendered-quality and Task13 GUI cross-gates remain open.
