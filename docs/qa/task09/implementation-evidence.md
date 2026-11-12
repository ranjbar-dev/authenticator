# Task09 implementation handoff

Parent acceptance update: Task09 functional implementation accepted after distinct
review_09 and focused P2/P3 re-review. The author handoff/history below is retained;
see [independent-review.md](independent-review.md) for independent commands, live
scope, original findings, corrections and final cursor-access limitation. Final
full suite88 and focused regressions passed. Existing global manual gates remain open.

12 November 2026. Author evidence, ready for the parent's distinct read-only
functional/UI/security/data-loss review. No acceptance/status/coverage change or
commit/push is made by this child. Parent-owned task/checkpoint edits are separate.
No real/default/reference vault, credential, backup or clipboard was read.

## Source checklist and decisions

Read Task09, CODEX_PROMPT/checkpoint, PRD FR2/FR1.5 and sections5.2,7–12,
architecture/security/storage/UI foundation and Task08 implementation/independent
review. RTK.md governed shell commands. Reference pixels remain unavailable per
reference-ui.md; this follows the observed inventory without a pixel-match claim.
Read the user's UI lessons and UI-consistency finding-patterns, values, decisions,
planning and implementing, with applicable process/checklist/word/plan support.
The preapproved task/autonomy settles the plan and named shared controls. No
additional child was spawned. This is ending3: the parent's existing process
owns review and integration. Existing Task04/07 checker calibration has its stated
limited scope; no new calibration pass or independent approval is invented here.

The bounded production form family is the one ROOT shell, its import review and
shared components/theme; diagnostic FoundationApp/examples are excluded. The QR
review is the nearest form reference. Shared-piece reach was checked in source;
there is one production shell, not a fabricated majority over several pages.
Estimated cost: 30 project/doc files, 20 source searches, plus skill/support reads.

The refreshed Task09 checklist, in reading order:

1. Existing ROOT scroll holder/header; Account title, Back to accounts, and
   explicit discard/Keep editing. Shared heading/button/card roles and content
   width/padding/gaps apply (request roles, shared-piece mechanism at level4).
2. Service, optional website, optional identifier in reference order, via the
   same labeled fields as QR review. Focus begins at Service.
3. New/scanned secrets are concealed by default. Deliberate Reveal/Conceal uses
   a native password input with a custom visible/masked layouter. Editing starts
   with **existing key is kept**, an empty replacement buffer and explicit
   Change secret key / Keep existing secret actions. No seed is put into the
   editable replacement buffer merely by opening Edit.
4. Collapsed Advanced always retains a concise algorithm/digits/period summary,
   including nondefaults from QR. Expanded, labeled native combo boxes restrict
   selections to SHA1/256/512,6/8,15/30/60/120. Full BODY typography applies to
   selected/menu text; selected ON_BLUE/BLUE replaces unreadable pale-on-blue.
   Actual selection closes the popup and returns selector focus. Tab/Space work;
   Escape closes an open selector without discarding the form.
5. Creation's optional Backup codes on/off reveals a masked labeled multiline
   input, intentional reveal, unique/duplicate counts and explicit exact-duplicate
   removal acknowledgement. Other fields survive toggle changes. No implicit
   replacement of a stored recovery set is offered by Account Edit.
6. Shared inline field/status errors retain input; changing input clears stale
   global validation feedback. Duplicate warnings exclude the selected edit ID
   and share the domain predicate with vault/trash/batch. Creation stays new;
   Save changes updates only the selected stable ID.
7. Shared stacked full-width Save/Cancel actions remain reachable at340/600/960.
   Busy removes editors/repeat Save, presents progress/Cancel and lock, and names
   the operation accurately. Escape during save locks; it does not promise a
   rollback after commit. Manual Cancel/Back/Escape offer Keep editing/Discard;
   QR import keeps its Task08 explicit Cancel/Escape behavior.
8. Shared opaque icons, focus strokes, font bundles and foreground/background
   pairs apply. No private styles, excluded account/cloud/language/promotional
   controls, backend replacement or hardcoded production business data were added.

## Ownership, normalization and data-loss boundaries

`AppState::begin_save(SaveIntent)` generalizes the existing sole-owner import
worker. `begin_import` wraps Import intent, preserving whole-selected-batch QR
atomicity and overlap/lifecycle/generation/deadline guards. The repository/key
is moved, never cloned. Create adds account plus optional recovery set within
one detached cancellable encrypted transaction. Edit explicitly targets its ID,
rejects absent/trashed/stale targets and compares the original ephemeral draft
as well as state; same-second metadata/seed/settings edits cannot evade a coarse
timestamp check. Preservation of the existing seed is enforced again inside the
transaction when replacement was not deliberately requested. Identity/creation,
icon, folders, existing recovery sets/IDs/used states and other accounts stay put.
The original edit snapshot is ephemeral, zeroizing conflict evidence, not a
second committed-record cache. Failed writes return the unlocked repository and
retain editors; lock/cancel/panic/stale completion fail closed.

`domain::recovery_input::RecoveryInput` is reusable by Task15: UTF-8 strings,
optional initial BOM, LF/CRLF, <=1MiB, trim surrounding whitespace BEFORE pure
BackupCode value validation, omit blanks, reject internal control characters,
<=10,000 nonblank lines and <=4096 bytes per normalized value. Exact duplicates
are counted, first occurrence order is retained, and approved removal requires
acknowledgement tied to the exact current raw text; editing it invalidates that
acknowledgement even through direct service calls. No raw value is in an error.
Preview uses the pure `BackupCode::normalized_value`, not random record creation,
and caches only counts/error on text changes. Raw/normalized/acknowledgement text
uses zeroizing owners. Newly created codes start unused. Task15 still owns file
input, standalone code-set editing and exact retained-code used-state policy.

The multiline editor bounds native input to1MiB+1 Unicode scalars (at most
4*(1MiB+1) UTF-8 bytes for layout); the domain still rejects >1MiB bytes. The extra
scalar is intentional: native clamping must never turn an oversized ASCII paste
into an accepted truncated input. This does not promise unlimited input retention.
No file format/schema/envelope change or migration was made.

Precommit partial/write/rename/cancellation/validation failure preserves committed
memory/ciphertext. A cancellation losing final-action admission (corrected after
Review09; see the follow-up below) may leave the
complete encrypted operation and a locked session. No postcommit deletion or
false rollback guarantee is added. The actual UI fault test first deliberately
replaced its OWN disk bytes; failed Save preserved those **faulted pre-save**
bytes, not the earlier original ciphertext. Restoration was explicit before retry.

## Secret display and accessibility limits

Concealed fields produce masked galleys and masked full AccessKit values/text
runs. Explicit Reveal intentionally exposes plaintext to visual layout and
AccessKit TextRun children in pinned egui0.37; a PasswordInput parent role or a
native UIA tree masking its Value is NOT proof that all revealed AX children are
secret-free. Comments/tests document that distinction. Native password semantics
prevent clipboard CopyText output in either mode; Cut can still delete the
selection. This is clipboard-output protection, not a promise that Cut is inert.
Conceal/lock/form exit drops widget histories and lock drops editor/snapshot/text
owners; framework/font/allocator/compiler/OS copies cannot be guaranteed erased.
No system clipboard was read or changed by automation. Actual Narrator remains
unverified.

## Commands and results

All commands below passed unless the diagnostic is explicitly noted.

```text
rtk cargo check --locked --all-targets
rtk cargo test --locked account_ --lib -- --test-threads=1
rtk cargo test --locked --all-targets
rtk cargo test --locked password_clipboard --lib
rtk cargo test --locked multiline_conceals --lib
rtk cargo test --locked ui::components --lib
rtk cargo test --locked ui::import_review --lib
rtk cargo test --locked capture_privacy_tests --lib -- --test-threads=1
rtk cargo test --locked normalization --lib
rtk cargo test --locked account_validation_cancel --lib
rtk cargo build --locked --example account_editor --example import_flow --example qr_batch_fixture
rtk cargo fmt --all
rtk cargo clippy --locked --all-targets --all-features -- -D warnings
```

Full all-target suite: RTK reported80 passed,2 ignored,65 filtered across13
suites (53.66s). The filtered inner child-process runs/ignored executable entry
probes are existing storage-test machinery; they are not new skipped acceptance
checks. This pass preceded later minor selector/focus/error-clear refinements
and two full-node component tests. After those changes, focused component5,
review2, capture privacy2, normalizer1 and account validation1 tests passed.
The all24/create-edit/atomic group was rerun after direct parameter assertions
were added:3 passed (58.34s). Full checks are listed below again at final handoff
when final formatting/lint/regression results are available.

Meaningful coverage includes:

- All24 algorithm/digit/period combinations created by real form service/save
  worker, then actual store/instance-lock drop+reopen; stored enums and same-time
  code values checked. Every stable ID is edited to the next combination and
  reopened again; metadata, exact enums, code values, IDs, creation, memberships,
  recovery set/code IDs and initially unused states persist. Existing published
  RFC/domain tests establish engine correctness separately from persistence.
- Deliberate seed replacement, unchanged-seed default, Unicode shared duplicate
  policy/self exclusion, no implicit overwrite/new ID collision, missing/trashed
  target rejection and same-second stale edit rejection. A separate used recovery
  code/folder regression proves existing associations/IDs/used state survive edit.
- Atomic create+codes and edit through real encrypted transactions with injected
  partial-write/before-replace failures, rename-gate cancellation, a candidate
  recovery-set validation failure AFTER candidate account add, and successful
  lock/unlock. No partial account/recovery commit or temp remnant is accepted.
- BOM/CRLF/LF/trim/blank/exact duplicate/boundary/control cases and acknowledgement
  invalidation even when text changes outside the widget.
- Full AX concealed/revealed/reconcealed node values, Copy/Cut output guards,
  multiline actual galley rows/font/weight/leading/tracking, and existing lock/
  Help/capture/batch regression tests.

An initial normalization command attempted to relink the Windows test executable
while the preceding owned account test still ran and got LNK1104. Waited for that
test to finish, then sequential reruns passed; it was not a product failure.
A nested PowerShell cleanup string failed parsing before executing. Replaced it
with a literal temporary script, validated exact temp parent/name/ownership marker
and locator before native Remove-Item, and observed both profile/locator removed.
The script deleted itself. No other profile or process was affected.

## Actual pointer/keyboard journeys and screenshots

Used documented Computer Use `@oai/sky` from node_repl, unique returned owned
windows and fresh geometry/text observations. Normal unelevated `launch_app`
avoided the elevated-shell UIPI limitation. Physical press_key entered egui text;
pointer clicks were SendInput-based, not synthetic element.click. No OS network,
display, security setting or installed reference app interaction occurred.

Disposable preview1/2 created only owned encrypted temporary profiles. The final
ephemeral journey used `.tmpnkBRg6`, manually entered a public test seed and demo
service, checked Reveal/Conceal and field order, opened optional masked backup
input, entered exact duplicate lines, and saw Save reject until explicit removal
acknowledgement. F4 armed its own ciphertext conflict; Save retained metadata,
masked seed/codes and acknowledgement. Faulted pre-save bytes were unchanged.
F4 restored original ciphertext; retry saved account+set. F5 dropped/reopened the
actual repository. Edit default kept the key; Cancel/Keep editing/Discard, missing
service and invalid replacement secret were exercised. Returning to Keep existing
secret and saving renamed only demo and persisted SHA512/8/120. Both that current
code and F5-reopened code matched independent Node standard-library HMAC.

An actual process relaunch used final `examples/account_editor.rs`: unique owned
`authenticator-task09-ZIRRfx`, existing published seed/account fixture with folder
and used/unused recovery codes; pointer Edit and keyboard renamed the service and
selected SHA512/8/120, then explicit Save. F9 exclusively wrote a public locator
plus matching ownership marker and retained this test-only directory. Closed the
process and observed no owned window and unchanged ciphertext. A new normal
argument-free launch verified the temp parent/prefix/marker, reused exactly that
file without reset, started locked and exposed no account metadata. Actual masked
passphrase typing + pointer Unlock restored the edited account; HMAC matched.
F5 reopen and this new-process relaunch are distinct evidence, not synonyms.

Independent HMAC comparisons recorded metadata/boolean only (never actual or
expected OTP): SHA1/6/30 at Unix1794508599 counter59816953 matched; edited
SHA512/8/120 at1794508906 and F5 reopen1794508908 counter14954240 matched.
Final relaunch fixture SHA512/8/120 before Close1794509321 and after new-process
unlock1794509390 counter14954244 both matched. Seeds/source QR/current OTPs were
not written as evidence artifacts.

At340/600/960 logical widths, visually inspected readable labels, theme/contrast,
wrapped instructions, masked fields, open menus, focus, errors, Save/Cancel and
scrolling. Actual pointer selection closed menus; actual Tab/Space selected8 and
120; Escape closed a popup without a discard prompt. The corrected final build
also removed stale validation feedback after typing a correction. Ctrl+L removed
editor/account/settings nodes and focused the empty passphrase field. Closing the
owned windows removed ordinary TempDirs. The retained relaunch directory, public
locator and cleanup script were verified absent after guarded cleanup. Unrelated
`vault_lifecycle.exe` was left alone.

Sanitized component JPGs under `images/` contain only disposable metadata and
concealed sensitive fields, no OTP or revealed seed/recovery values:

- manual-600, masked-controls-600, edit-existing-600, edit-340, edit-960;
- advanced-340, advanced-600, algorithm-menu-960, period-menu-340;
- backup-duplicate-review-600, write-failure-retained-600,
  inline-validation-960, invalid-secret-960, corrected-validation-600;
- discard-340, locked-600 and relaunch-locked-600.

The earlier algorithm-menu screenshot includes then-stale global validation text;
final build clears that text when input changes, proven by corrected-validation.
Representative images were read back for visual QA. A failed optional Sharp
import prevented a proposed result redaction; no unredacted result was saved.
Existing Task08 live capture/image/selected-batch evidence remains applicable;
this change reuses its worker and shared form, with automated regressions rerun.

## Remaining limits and review handoff

Author evidence is not independent acceptance or external security certification.
Parent must inspect the diff and start the distinct reviewer with Task09's full
screen checklist, these actual artifacts and UI-consistency verifying. Exact
reference pixels, physical150/200/mixed DPI, networking-disabled machine journey,
real OS session-lock/hardware suspend and Narrator remain the existing explicit
manual/hardware/future gates. No unperformed gate is claimed here. Task15's full
recovery editor/file/used-state management is not declared complete by adding its
shared normalizer now. Task10 owns the complete code-list/copy interaction.

## Final shared multiline refinement and checks

Parent source feedback correctly identified that native desired_rows is a minimum,
so long recovery input previously expanded beyond its apparent five-row height.
The final shared helper now wraps TextEdit in an inner vertical ScrollArea bounded
by `theme::MULTILINE_HEIGHT` (six complete BODY rows +8 points). It disables content
drag scrolling so selecting/editing text does not steal pointer actions. Visible
response/focus/outer-scroll targets and full AX input bounds refer to that viewport,
with the layer's global transform applied to AX bounds. Native caret/Home/End and
wheel/bar scrolling remain inside the editor. A 40-line component regression checks
the bounded viewport/AX and Save position; actual galley typography tests still pass.
Lock explicitly closes any native combo popup memory before clearing the form,
preventing an old unrendered selector from surviving into a later unlock session.

In fresh owned `authenticator-task09-is4dtb`, actual manual form entry plus20
distinct disposable lines produced **20 unique /0 duplicates**. The input stayed
compact; Ctrl+Home/Ctrl+End scrolled its caret internally, pointer input inside the
visible field focused it, Tab reached Reveal then Save, and keyboard focus brought
Save/Cancel into view at340. Pointer Save completed the real encrypted account
operation. `twenty-lines-600.jpg` and `twenty-lines-340.jpg` show only masks/counts,
not recovery values or OTPs. The helper-only final AX-bound adjustment has the
explicit component regression; the preceding live pointer/keyboard check used
the same bounded scroll/focus control. The owned window closed and its directory
was observed absent. No process-relaunch/KDF retest was needed for that geometry
change. A preliminary owned viewport check directory `authenticator-task09-v2dPU8`
also closed and was observed absent before the final minimum-height refinement.

Final checks after the helper/error refinements:

```text
rtk cargo test --locked ui::components --lib
  6 passed,64 filtered; actual typography/AX/Copy-Cut/bounded40-line checks
rtk cargo test --locked ui::import_review --lib
  3 passed,68 filtered; includes actual collapsed nondefault scanned-URI summary
rtk cargo test --locked account_with_recovery_and_edits_are_atomic --lib -- --test-threads=1
  1 passed,68 filtered (9.62s)
rtk cargo test --locked account_validation_cancel --lib
  1 passed,68 filtered; used recovery ID/state and folder preservation included
rtk cargo test --locked editable_review_masks_secret_and_lock --lib -- --test-threads=1
  1 passed,70 filtered (7.19s); includes closing retained popup memory on Lock
rtk cargo build --locked --bin Authenticator --example account_editor --example import_flow --example qr_batch_fixture
  passed; final executable/examples built for independent review
rtk cargo fmt --all -- --check
rtk cargo clippy --locked --all-targets --all-features -- -D warnings
rtk git diff --check
  passed (final formatting/lint rechecked at handoff)
```

There are19 sanitized component JPGs in total. Status/coverage/commit/push remain
with the parent. All owned previews are closed and the relaunch profile, locator
and guarded cleanup script were independently checked absent through filesystem
queries; the unrelated existing lifecycle preview was untouched.

## Review09 follow-up: P2/P3 corrections

The distinct first review found P2 responsive Lock and P3 contradictory duplicate
feedback. Its report is preserved separately in `independent-review.md`; the
original passes above are not retroactively relabeled as passes of these fixes.
The P2 defect was confirmed in source and the old deterministic gate test, not
by an observed real filesystem/kernel hang, ordinary visible-OTP failure or data
loss. This section records the sequential implementer's correction, pending
focused independent re-review.

P2: the old cancellation mutex was held while the admitted native final replace
ran, so synchronous cancel could wait for filesystem I/O before Lock cleared
state. `OperationCancellation` now uses atomic REQUESTED/ADMITTED flags. Cancel
does not acquire a mutex; a compare-exchange linearizes action admission against
cancel. A separate mutex only serializes background actions sharing a token.
Default/clone/sequential reuse remain supported until cancellation; an admission
drop guard clears the admitted bit after success/error/panic, preserving a cancel
request. Lifecycle/idle checks immediately precede admission. An admitted action
may complete or fail; native rename success remains the actual commit point.
No crypto, schema, envelope, partial-write or atomic file semantics changed.

Shell clears UI-owned review/form/widget state before state cancellation. State
increments generation and clears query/intent first. Worker cancel and UI poll
use try_lock on completion publication, so neither waits on an occupied result
slot. Completed results are locked when available; stale generation/token checks
reject them before UI adoption. Worker-only post-publication cancellation checks
close the race where UI try_lock missed a producer that had just checked its
token. A later UI cancel can acquire the released slot itself. This does **not** interrupt active Argon2/kernel
I/O or force-wipe a worker's vault, DEK, candidate or scratch while in use. Such
owners can remain until that operation returns, then lock/drop. Framework,
allocator, OS/pagefile copies remain outside erasure guarantees.

The revised `cancellation_and_rename_gate_have_one_linearization_order` retains
the held-action test and reverses its former blocking expectation: cancel returns
while the admitted action is still held; releasing it permits that action to
complete, while subsequent cancelled work rejects. It also covers token reuse
after an error and after an action panic. Existing pre-admission cancellation,
partial-write/native replace failures and whole-batch tests remain and passed.

The mandatory new regression,
`lock_clears_ui_before_an_actual_admitted_save_action_is_released`, starts a real
two-account encrypted save via the actual AppState worker/store/temp-file path.
A per-token cfg(test)-only gate holds it after admission, before the native final
replace. Before releasing the gate, a separate scoped UI thread completes the
real Shell Lock and renders AX: generation advanced, query empty, review absent,
native widget state absent, no private metadata/seed/editor AX nodes, worker still
busy, session inactive and no success feedback. The original ciphertext is still
present while held. On release the entire encrypted operation is allowed to
complete; late poll yields no success/unlock/import IDs, and explicit unlock
reveals both accounts. A release-on-drop guard lives **inside** the scoped closure
so timeout/assertion unwinding releases the gate before scoped joins; regression
failure cannot strand the held worker in CI. This test uses only its owned
TempDir. The gate and AppState hook do not exist in production builds or affect
default/unknown profiles.

P3: approved duplicate removal now clears its code-specific global feedback and
pending error-scroll flag. Disabling optional backup input also clears corrected
backup-specific feedback. Unrelated metadata/persistence errors and entered
values remain; changing the raw code text still invalidates acknowledgement.
The new service regression recreates failed Save → acknowledgement → cleared
warning, retained input, subsequent save eligibility, acknowledgement reset after
text edits, and preservation of metadata errors through acknowledgement/toggle.

Actual SendInput pointer/key recheck used only fresh owned
`authenticator-task09-441AjJ`: entered valid disposable manual metadata/seed and
two identical recovery lines, pointer Save showed the acknowledgement error,
pointer Remove exact duplicates changed the button and **immediately removed**
the contradictory red status, with masked codes/metadata retained. Pointer Save
then succeeded. `images/duplicate-ack-corrected-600.jpg` contains only concealed
values/public fixture metadata/counts, no OTP/raw seed/recovery values. The owned
window closed, TempDir was observed absent and no relaunch locator remained.
The unrelated lifecycle preview/default/reference data were untouched. There are
now20 sanitized author component images. No HMAC/process-restart repetition was
needed for these cancellation/feedback changes; their earlier proof remains
distinct.

Exact checks after the corrections:

```text
rtk cargo test --locked lock_clears_ui_before_an_actual --lib -- --test-threads=1
  1 passed,72 filtered (10.66s)
rtk cargo test --locked cancellation_and_rename_gate --lib -- --test-threads=1
  1 passed,72 filtered
rtk cargo test --locked duplicate_acknowledgement_and_toggle --lib
  1 passed,72 filtered
rtk cargo test --locked --all-targets
  preliminary admission/feedback pass:87 passed,2 ignored,72 filtered across13 suites (61.75s)
  final pass including publication cleanup:88 passed,2 ignored,73 filtered across13 suites (56.38s)
rtk cargo test --locked cancellation_does_not_wait --lib
  1 passed,73 filtered; cancel returns while the completion slot is occupied
rtk cargo test --locked lock_clears_ui_before_an_actual --lib -- --test-threads=1
  final post-publication version:1 passed,72 filtered (11.28s)
rtk cargo fmt --all
rtk cargo clippy --locked --all-targets --all-features -- -D warnings
rtk cargo build --locked --example account_editor
  passed; this final example was used for the actual P3 pointer check
rtk cargo build --locked --bin Authenticator --example account_editor --example import_flow --example qr_batch_fixture
  final post-publication build passed at handoff (11.68s)
rtk cargo fmt --all -- --check
rtk git diff --check
  passed at final handoff; strict Clippy was also rechecked and clean
```

The two ignored probes/filtered inner runs are the existing storage child-process
machinery, not skipped new acceptance cases. Final fmt/diff/lint checks are
repeated at handoff. No status/coverage/parent review report/commit/push was changed
by this implementer. Independent re-review and existing physical DPI/offline/
hardware/Narrator/reference-pixel gates remain with the parent and owning tasks.
The final full rerun was justified by the additional producer-publication race
correction, not by copy-only changes. The occupied-slot test drops its held mutex
before assertions/scoped joins so a regression fails without hanging.
