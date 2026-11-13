# Task13 native encrypted backup import implementation evidence

13 November 2026. Author implementation record, not independent approval. Parent
owns task/index/coverage/PRD statuses and commits. No Task14 work was performed.

## Behavior and changed paths

`src/ui/backup_import.rs` adds a session-only Import backup form, cancellable file
picker owner, concealed backup passphrase, authenticated counts and paginated
conflict/folder choices. The screen explains native AUTHPBK1 recognition by bytes,
unsupported local/reference formats and the absence of passphrase recovery.
Merge defaults preserve existing data and skip complete conflicting incoming
account/recovery/membership bundles. Keep both and per-folder default/reuse/copy
choices bind to immutable preview indices. Representative match metadata, exact
union match counts/reasons, names, IDs, folder colors and Trash states are shown.
Each collection view shows at most five records per page, including dense
incoming-duplicate cases. There are no raw seeds, current OTPs or recovery values
in import review. Used timestamps, recovery-code/set Trash metadata, account and
folder creation/update/deletion metadata are available without revealing values.

Final review uses actual `ImportPlan.summary` and `candidate_for_review`, comparing
current/incoming/resulting counts, removed Replace counts, skipped bundles,
remapped IDs, account/recovery states and exact folder copy/reuse resolutions.
Separate display/security selectors default to current values. Current and backup
display query/sort/view/filter and lock/clipboard timeouts are visible before the
plan; resulting values and fallback-to-All are visible before Apply. Startup and
local vault/key/passphrase/identity remain local regardless of choices. Replace
requires a second shared ROOT modal, initially focused on Keep current vault.
The modal names current removed/resulting counts and supports scroll/keyboard.
Back from final review retains the authenticated preview and choices; Back from
preview discards those records, returning to the file/passphrase step.

`src/ui/shell.rs` integrates Settings → Export & Import → Import backup, pending
display-preference preparation, typed completion, separate destructive consent
and success navigation. No snapshot is taken until the latest display projection
has saved successfully (including a currently pending/coalesced save). Failure
clears the passphrase and visibly offers retry with the display intent preserved.
Previously completed preference saves are independent transactions; later backup
cancellation is not described as undoing them. Only adopted approved success
invalidates current Codes/Backup queries, preference cache, ordered IDs, old
imported markers, export form/review, clock and clipboard presentation before
reloading from the actual store. Codes uses real current engine data afterward;
the retained/imported display filter can naturally hide accounts.

`src/ui/state.rs` adds typed Preview/Plan/Apply worker requests/results. All KDF,
file reads/hashes, conflict work, candidate clones/validation and atomic apply run
on the existing sole-store worker. No second mutable repository or DEK is created.
Generation, original monotonic idle deadline, lifecycle watch, nonblocking
cancellation/publication and post-publication checks follow existing save/export
rules. Lock immediately drops form, path, incoming metadata, candidate and widget
history; in-flight worker memory may remain until OS/KDF work returns. Cancelled
completion payloads drop before adoption. A write already admitted can finish
completely after Lock; it cannot publish late success or unlock the session.

`src/backup/{mod,files,merge}.rs` permits a borrowed immutable preview to rebuild
a plan after choice changes. Only the opaque source path/ciphertext digest are
cloned in this binding; the exact local revision, immutable snapshots and consuming
apply remain enforced. `src/backup/envelope.rs` explicitly classifies a non-native
eight-byte discriminator as Unsupported without guessed decoding/KDF. Truncated
native input remains invalid. Native/source limits and cryptography are unchanged.

`src/platform/backup_picker.rs` factors its existing native Save adapter to also
offer GetOpenFileNameW with FILEMUSTEXIST, native/all-file filters and the same
cancellation timer/path guard flags. Selection grants no import approval. Export
retains its ROOT overwrite approval and engine behavior. `src/ui/mod.rs` registers
the new form module. `examples/import_backup.rs` is argument-free: generated
public fixture backup, independent source/destination keys/passphrases in a fresh
TempDir, production app injected with that destination, no default/reference data.
Its F10 callback follows the existing real-framebuffer redaction pattern, but no
framebuffer evidence was obtained here.

`src/ui/backup_import_tests.rs` contains the production form/worker synthetic
regressions. `docs/architecture.md` and `docs/portable-backup-format.md` document
the implemented lifecycle and borrowed plan semantics.

## Checklist and design basis

Sized as one new native ROOT form, following Task13's already authorized screen
checklist and `src/ui/export.rs`, with shared values/components as authoritative.
The only existing portable-backup form was Export; its single-member family is a
precedent, not a measured majority convention. PRD inventory fallback is recorded
in `docs/reference-ui.md`; no safe original screenshots/pixels were available.
No exact-reference fidelity claim is made.

The session checklist, read before implementation, was: shared ROOT header/back;
native format/file guidance; shared masked field; counts before choices; labeled
safe Merge/Replace/conflicts; exact candidate review; separate Replace dialog
focused on safe action; progress/retry; secret/metadata cleanup on exit/Lock;
success data/projection refresh and logical focus. Shared `c::heading/body/
secondary/button/labeled_secret_field/labeled_selection/card/status/progress/
Dialog` and `theme` roles fill those positions. No page-specific color/font/icon
styling was added. Whole Noto Sans roles400/600 and existing BLUE/ON_BLUE icons
remain in use. Root drag scrolling remains disabled; controls use shared focus
scrolling. No excluded sign-in/cloud/language/encryption controls were introduced.

Applied ui-consistency finding-patterns/values/decisions/planning/implementing;
the existing autonomous brief supplied advance plan authorization. Session plan:
implement this form against the checklist; report ready to parent; parent dispatches
distinct read-only UI/security/data-loss review with verifying. No author approval
is claimed and no additional child was spawned. Existing same-kind Task04/07
calibration detected planted source font differences; its documented live/pixel
limits persist. No new general visual sensitivity claim. Estimated reading cost
35 source/document files and35 grouped searches; exact skill counts were not kept.

## Checks and meaningful coverage

All shell commands used RTK. Generated/public fixtures only; no real/default or
reference vault/backup/credentials/clipboard reads, raw payload logging or captures.

| Command/check | Result |
|---|---|
| `rtk proxy cargo test --locked --lib backup_import_ -- --nocapture` (first stable UI run) |8 passed,0 failed,41.06s |
| `rtk cargo test --locked --all-targets --all-features` |141 passed,4 directly ignored helper entrypoints exercised by parent tests,16 suites,91.23s |
| `rtk cargo clippy --locked --all-targets --all-features -- -D warnings` |Passed; repeated after final test additions |
| `rtk cargo fmt --all -- --check` |Passed |
| `rtk cargo build --locked --bin Authenticator --lib --example import_backup --all-features` |Passed |
| Final focused `rtk cargo test --locked --lib backup_import_` |9 passed,0 failed,39.58s |
| `rtk git diff --check` |Passed |

The native production Task12 form/worker writes the encrypted source backup.
Task13 form/worker authenticates, plans and explicitly confirms Replace into a
fresh independently keyed local vault. The test verifies24 OTP parameter
combinations, source/restored codes and an independent published RFC6238 HMAC-SHA1
value. Normalized full codec equality checks the entire record graph, preserving
all per-record IDs, metadata, timestamps, OTP settings, folder memberships,
recovery values/used states and Trash. Local vault/preference identity and startup
are retained. Explicit imported display/security timeouts match the reviewed exact
values; default import preserves local300s/30s security and pending local display
intent, including Replace.

Other regressions cover incorrect password, changed ciphertext, future version,
local AUTHLOC1, unproven reference format and oversized files without success counts
or mutation; default Merge and KeepBoth complete bundles, folder copies/reuse,
Back/rebuild without another passphrase; source/local revision changes between
preview/plan/apply; native write fault preserving prior bytes and a later retry;
pending-preference save failure/retry; duplicate worker refusal; early Lock,
stale lifecycle watch and expired idle deadline preventing admission; held actual
rename admission with privacy/locked AX/frame established before I/O release,
complete post-admission write and no late UI success. Picker expiration suppresses
chosen-path metadata and dropping a picker never waits on its held result mutex.

Synthetic ROOT pointer press/release events check Apply/Replace/safe Cancel/Back.
Synthetic keyboard checks safe modal focus/scroll, Help/Escape preservation and
normal340×780,600×800,960×900 viewport primary-action reachability. Scroll animations
are allowed to settle before bounds are asserted. Full-height24000 geometry/AX
passes inspect long metadata and password masking; they are not normal-window
reachability or visual proof. Submit-ID stability through validation-error insertion
is checked separately. These synthetic events are not native OS pointer/key input.

The shared secret editor has no editing-time character cap (`usize::MAX`), as in
existing export/unlock/account forms. Import validates the exact12–1024 UTF-8-byte
passphrase before any backup KDF/parser request, but an oversized pasted edit can
temporarily occupy UI memory until correction or exit. No new global secret-editor
behavior was introduced in Task13; editing-buffer hardening remains a concrete
Task20 security-review consideration. Password layout uses masks and undo history
is cleared per edit pass; this is not an immediate or complete-erasure guarantee.

During author checks a clipped destructive modal button was initially clicked by
off-clip synthetic AX bounds; the test now tabs from the safe initial action,
settles shared scrolling and clicks the visible confirmation. Early assertions
also sampled scroll animation before it settled. The stable focused run above
passed after these test corrections. A simultaneous targeted test/relink attempt
briefly produced LNK1104 on this agent's own running test executable; checks were
then run sequentially and linking passed. The unrelated running lifecycle preview
was never stopped or touched. No standalone all-target build was attempted.

## Open native/visual gates and cleanup

Computer-use skill/guidance/API/confirmations were read. Through documented
`@oai/sky` in node_repl, fresh inventory showed no owned import-preview window.
Two normal argument-free launches of the explicit owned import example both
returned **GetCursorPos failed: Access is denied (0x80070005)**. Fresh inventories
after each again showed zero owned windows. Native attempts stopped at the stated
bound. No production/default/reference app was launched, permission settings
changed, alternate input bypass used, or screenshots fabricated.

Real Windows pointer/key/picker interaction, Task12→13 native interactive restore,
actual framebuffer screenshots at340/600/960, visual/contrast/overflow/DPI review,
Narrator, hardware lock/suspend, offline network-disabled and second-device/user
recovery remain unverified. Existing Task12 native gate and Task05/07/08/10 hardware
gates are not closed by this implementation. Task21/23/final acceptance must retain
the applicable gates; independent source review is still required here.

Post-attempt exact-path process inventory and owned `authenticator-task13-*`
TempDir inventory returned no owned process or fixture directory. Rust test
TempDirs clean up through ownership; no retained backup locator/screenshots were
created. No recursive deletion was needed. Changes remain uncommitted for parent
review/integration; parent-owned task/status edits were not authored here.
