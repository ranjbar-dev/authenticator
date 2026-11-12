# Task10 implementation evidence

Author handoff, 13 November2026. This is implementation evidence, not independent
acceptance. Parent owns review, task/coverage status and commits. No real/default/
reference vault was opened. All storage/native/UI fixtures are generated disposable
or published public vectors. Exact reference pixels remain unavailable, as recorded
in docs/reference-ui.md. Existing hardware/offline/Narrator gates are not waived.

## Refreshed screen checklist and implementation

The supplied Task10 checklist was reread before implementation. The page family is
the existing native Root shell (`ui/shell.rs`), shared components and theme, not a
webpage or a new design system. No root DESIGN.md or original safe reference pixels
were available. The PRD inventory controls roles; shared components control their
implementation. Sources read include Task09's complete author/independent evidence,
domain/OTP/storage/codec/lifecycle/capture/editor code and UI foundation docs.

- Header: reference-inventory title/Help, Lock, prominent Screenshot to Scan and
  Add Account; photo/manual flows retain the real Task08/09 services. Codes home
  omits redundant branding/unlocked captions to keep content near the top.
- Toolbar: bounded native search, all six exact reference sort labels, actual
  All/Unfiled/existing-folder choices and List/Grid with visible selected values.
  Shared stable-value selection supports duplicate folder display names safely.
- Cards: solid BLUE/ON_BLUE local key tiles, actual service/optional identifier,
  grouped6/8-digit code, true remaining seconds/progress and separate full buttons
  for Copy/Edit/Delete. Newly saved IDs highlight/scroll without changing sorting.
- States: real empty/no-results/loading/clock-failure/guidance states; no-results
  offers Show all accounts. The compact no-results text and equal-height wrapped
  navigation tabs keep useful controls in the narrow first screen.
- Responsive: shared LABEL14/20 remains unchanged; narrow tabs use equal56-point
  two-line boxes in one row at340, with full accessible names. More constrained
  logical widths still stack. Two-column grid starts at600 content points and
  otherwise falls back to one readable column. No private font/style replacement.
- Input/focus: native choices/buttons/labels and shared focus borders; Root content
  drag scrolling is disabled. Wheel/bar/focus/caret behavior remains native.
- Lock: query/local pending preference projection/ID cache/editor/widget histories/
  dialogs/popups clear before cancellation. Hidden/minimized Lock requests one
  nonsensitive UI/AX scrub without showing an ordinary hidden window. Existing
  capture full-frame-before-hide behavior is retained.

UI consistency finding-patterns/values/decisions/planning/implementing and relevant
support were read. The task's existing preapproved checklist supplied the plan;
user authorization permits autonomous implementation and repository evidence.
No additional agent was spawned. Read/search cost is approximately40 project files
and40 grouped searches; the family remains Root plus shared helpers, not an invented
multi-page convention. The separate reviewer must use verifying and check shared
narrow tabs in editor/locked routes as well as Codes. Existing04 calibration and07
native source plants are limited prior evidence, not a claim that every new defect
is detectable or that reference pixel fidelity has been established.

## Data and clock behavior

`src/ui/codes.rs` caches only derived stable account-ID order. Service/account
search and metadata sorts use `caseless0.2.2` Unicode16 default full case folding,
including sharp-S and sigma; no locale collation, accent stripping or normalization.
Latest/Earliest use creation time; stable ascending IDs break all ties. Missing
identifiers always sort last. Actual folder memberships and active account state
are authoritative. Visible cards alone compute HMACs; the repository/DEK is never
cloned for rendering.

One clock sample serves a frame, with UTC-second repaint deadlines. Click reads
the clock afresh and copies raw digits. The clipboard actor receives only counter/
period validity metadata and rejects expired queued copies at native admission and
immediately before text publication; it never receives a seed. Clock error removes
old values and disables Copy. Wall versus monotonic delta >5seconds gives actionable
local date/time guidance, without remote verification or changing the OS clock.
No changing code/countdown is a live accessibility region.

The single-owner asynchronous SaveIntent worker now also handles display/idle/
clipboard preferences and confirmed Trash. No production preference transaction
runs on the UI thread. Live query changes debounce500ms and coalesce while a real
write is held; stale completion cannot overwrite newer input. Normal close flushes
the latest valid display change behind a nonsensitive frame. Failure stays open
with Retry or deliberate Close without saving. All deliberate discard/ordinary
success paths attempt conditional clipboard cleanup while Root pumps its owner
messages for up to300ms; approved programmatic Close is not reintercepted. The
on-exit fallback waits at most300ms, never indefinitely. OS/idle/Lock or a second
forced close can cancel promptly; uncommitted edits are not promised durable after
force-close/Lock/storage failure/process termination.

Schema2 explicitly migrates schema1 in detached memory, defaulting query empty and
folder All while preserving all IDs/data/preferences/key wrapping. Unlock never
writes; next successful ordinary atomic transaction emits2 inside envelope1. The
frozen `tests/fixtures/schema1-public.bin` was independently packed from the v1
layout and includes an RFC seed/public metadata/IDs/links/used recovery code. It is
not a current-codec self-roundtrip or an excluded reference-format fixture. Actual
encrypted legacy unlock/failed migration write/success/reopen tests pass. New
preference/Trash metadata mutations clamp timestamps after local clock correction;
OTP still uses actual wall time. See docs/storage-format.md.

Delete requires a root confirmation with Keep account focused first, then performs
real soft deletion in the same cancellable encrypted worker. Recovery values/used
state and memberships remain; cancel/failure/stale-target/cancelled transaction do
not delete records. Full Trash UI/restore/permanent deletion remain Task18.

## Clipboard ownership and native evidence

`src/platform/clipboard.rs` implements a bounded8-message actor independent of Root
repaint/minimization. Zeroizing text is <=1MiB; the retained lease is only random
marker, HWND, sequence, UTF-16 length and salted SHA256, never raw expected text.
GlobalAlloc uses MOVEABLE|ZEROINIT and RAII; SetClipboardData success alone transfers
ownership. Failure buffers attempt wiping/freeing. History/cloud zero-DWORD opt-out
formats precede text. These are Windows hints, not third-party-history prevention.

Cleanup checks exact sequence/HWND before marker, then marker before bounded own
Unicode data/digest. Verification and EmptyClipboard share one exclusive open.
No unrelated private clipboard text is read or backed up/restored. Actual native
tests observed sequence advancement at CloseClipboard; no unproven cause is claimed.
Fresh exclusive reopen validates ownership/content before sampling established
sequence. Busy retains conservative provisional proof without rebasing; later
cleanup still needs exact sequence and all ownership evidence. An injected
post-close external replacement, even identical text, cannot be adopted.

Busy clear warns after2seconds and keeps retrying with nonplaintext proof. A
destroyed Root HWND does not prevent OpenClipboard(NULL) check/clear; ownership
changes preserve external data. Generation/lifecycle guards suppress queued/late
success after Lock. Framework CopyText output is intercepted; native password
semantics still block seed/recovery CopyText even on deliberate Reveal. Reveal may
expose visual/AccessKit TextRuns, as Task09 documents; no private TextRun IDs are
overridden. An OS call already admitted may finish after cancel or stall, retaining
current/queued buffers until it returns. Abrupt termination, OS/station denial,
history/observers and forced shutdown can defeat final cleanup. No hard guarantee
of memory erasure or OS/history clearing is made.

The actual Win32 helper runs in a separate child process with a new random
CREATE_ONLY noninteractive window station and private desktop. It verifies the
created station name and never switches the input desktop or touches WinSta0's
clipboard. Native checks cover copy/clear, final sequence, same-text newer copy,
marker/digest mismatch, busy holder, owner destruction, actual pre-text validity
guard and injected post-close external replacements. It also runs the REAL actor
against its owned HWND while PeekMessage/DispatchMessage pump owner notifications,
then verifies cleanup ACK and absent Unicode format before destroying the window.
This is meaningful native clipboard evidence, not actual user-pointer evidence.

Independent fake-adapter tests cover timer expiry without Root frames, lifecycle/
Lock/shutdown, identical external newer sequence, held pre-/post-admission cancel,
stale success suppression, Busy beyond2seconds followed by successful retry,
bounded full queue and counter-boundary/backward-clock/error rejection while Copy
is held. Direct native and fake cases complement one another.

Primary sources checked: [clipboard operations](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-operations),
[SetClipboardData](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setclipboarddata),
[sequence numbers](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclipboardsequencenumber),
[format/history hints](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats),
[window stations](https://learn.microsoft.com/en-us/windows/win32/winstation/window-stations),
[station switching](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setprocesswindowstation),
[thread desktop](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setthreaddesktop),
[Unicode fold crate/source](https://docs.rs/caseless/0.2.2/caseless/).
Pinned eframe/egui0.37 local source was inspected for App.logic/run_logic,
RequestPaintWhileHidden (one frame), native screenshot callbacks and text copy.

## Checks and diagnostics

Stable pre-final-layout full suite: `rtk cargo test --locked --all-targets` passed
102 tests,3 ignored helper entrypoints,88 filtered inner tests across14 suites
(64.41s). The new ignored native helper is actually invoked by its parent test in
an isolated process; the other2 are existing storage subprocess entrypoints.
No new acceptance case is silently skipped by these entrypoint annotations.

Focused commands include:

```text
rtk cargo check --locked --all-targets
rtk cargo test --locked ui::codes --lib
rtk cargo test --locked schema1_frozen --lib
rtk cargo test --locked trash_and_preferences --lib
rtk cargo test --locked all_six_sort_modes --lib
rtk cargo test --locked platform::clipboard --lib
rtk cargo test --locked isolated_native_clipboard_process --lib
rtk cargo test --locked queued_code_window --lib
rtk cargo test --locked codes_frame_clock --lib
rtk cargo test --locked hidden_logic_ignores --lib
rtk cargo test --locked live_query_coalesces --lib
rtk cargo test --locked ordinary_close_write_failure --lib
rtk cargo test --locked ui::components --lib
rtk cargo test --locked capture_privacy_tests --lib -- --test-threads=1
rtk cargo fmt --all -- --check
rtk cargo clippy --locked --all-targets --all-features -- -D warnings
rtk cargo build --locked --bin Authenticator --example codes_list --example account_editor --example import_flow --example qr_batch_fixture
rtk git diff --check
```

Independent standard-library Python HMAC compared120 real fresh-copy helper values
(all24 algorithm/digit/period combinations × boundary-1/boundary/boundary+1/forward
resume/backward jump times).120/120 matched; only aggregate booleans were printed,
and no code-value output/fixture was persisted. Temporary probe source/script was
removed. Existing RFC6238 vectors remain independently attributable unit evidence.

Diagnostics were fixed, not relabeled as passes: initial native before-close
sequence failed its owned-clear test; actual post-close validation fixed it. The
old capture privacy fixture manually set a query before new encrypted restore;
it now persists the sentinel and retains full before-hide assertions. The old
domain test's schema1 expectation was updated to2. A first narrow-tab test used
default8-point panel padding instead of production16; corrected to actual Root
padding and checked full bounds/labels/whole LABEL role. Native final close found
programmatic-close reinterception; an approved-exit flag plus re-entry regression
fixes it. The old owned nonsensitive preview could not exit that loop and was
terminated only after exact unique executable-path verification; no unrelated
process/profile was touched. One attempted relink before that old preview exited
got LNK1104; rebuilt after closing the owned executable.

## Actual UI evidence and remaining gates

The argument-free `examples/codes_list.rs` opens only a generated owned TempDir,
with public records/associations and guarded marker/locator process relaunch. F4
faults/restores only its own ciphertext, F5 reopens the same owner, F6/F7/F8 request
340/600/960 logical widths, F9 explicitly retains the guarded fixture. F10 uses the
real native framebuffer screenshot callback and redacts exact rendered OTP label
bounds BEFORE encoding/writing PNG. It is not a reconstruction; no raw OTP screenshot
is written. It excludes other windows and works around Sky's monitor-cropped capture
limit. Native framebuffer scale2 was observed; this is not a claim to have changed
or certified physical monitor DPI. Source capture bounds contain only geometry,
are debug-only and clear at Lock; release product has no screenshot persistence.

## Final actual interaction/artifact table

Normal documented Sky launch used only returned unique owned example windows.
Physical keys entered the public fixture passphrase (Unicode type_text is not used
for egui). Actual process exit was observed before ordinary relaunch, which began
locked. Unlock restored `public` query, Earliest First, Grid and1 matching public
account after the confirmed Trash operation. Earlier actual restart restored
Earliest First/List; encrypted owner-reopen tests cover all six sorts/both views/
all filter kinds. These are actual encrypted repositories, not frontend rows.

| Evidence | Observed result |
|---|---|
| [no-results340](images/no-results-340.png),680×1560 pixels | Actual typed `absent`; corrected one-row wrapped full-label tabs, equal Add/Scan actions and Show all accounts in the narrow first screen. |
| [list340](images/list-340.png),680×1560 | Actual code cards; keyboard Search→Sort→Folder→View→Copy; focus scroll brings explicit actions into view. Codes redacted before disk. |
| [editor340](images/editor-340.png),680×1560 | Physical pointer Edit opened the selected real account with existing key preserved. Escape/discard dialog and pointer Discard left2 matches unchanged. |
| [delete confirmation340](images/delete-confirmation-340.png),680×1560 | Physical Delete opened root confirmation with Keep account focused. Pointer Keep retained2 matches; a second pointer Delete→Move to Trash completed encrypted save and reduced matches2→1. |
| [grid fallback600](images/grid-fallback-600.png),1200×1600 | Fresh generated encrypted6-record fixture; real native keyboard popup List→Grid, then Tab to Copy. Full readable one-column fallback and Sort/Folder/View. |
| [grid960](images/grid-960.png),1920×1800 | Actual selected Grid with5 active records after Trash; two readable columns and explicit independent buttons. |
| [restored grid340](images/grid-restored-340.png),680×1560 | Actual process restart restored query `public`, Earliest First/Grid and1 remaining public match. This is a card/focus frame, not a captured clipboard-warning frame. |
| [locked340](images/locked-340.png),680×1560 | Actual Ctrl+L; full tab names stay readable and codes/account/query nodes disappear. [AX booleans](lock-accessibility.json) are all true. |
| [live HMAC booleans](live-totp-check.json) | Node/OpenSSL independent HMAC at UTC1794519165/counter59817305 matched both visible6-digit/SHA1/30 public codes; no code values are stored in this file. |
| [Copy diagnostics](copy-diagnostic.json) | Unlocked, listener is_ok/healthy, owner_nonzero and actual queue admission all true immediately after the live Copy attempt. No handles/clipboard text are recorded. |

The eight PNGs are complete native framebuffer captures, with exact rendered code
label rectangles redacted before encoding. They were read back for visual QA. One
initial600 file was copied while asynchronous encoding was still underway and was
zero bytes; it was replaced from a fresh actual600 fixture, with PNG IEND and
1200×1600 dimensions checked BEFORE copying. All final PNGs have complete IEND.
The later capture helper reported `CreateForMonitor`0x80070057 for its monitor
snapshot; accessibility/physical keyboard still worked, and the app's documented
native framebuffer callback captured the actual600 surface. No settings, security
or display changes were made. The first attempts' out-of-monitor crop/geometry
errors and the earlier cursor-access history do not become fabricated pointer
passes. Actual pointer checks include sort popup/selection, unlock, Copy, Edit,
discard, Delete/Keep/confirmed Trash and List/Grid. Keyboard checks include the
toolbar, popup options, Copy/Edit and Ctrl+L. Wheel scrolling did not steal actions.

### Open core live Copy gate — independent reviewer must reproduce

The final live interactive-station pointer AND keyboard Copy attempts return the
explicit safe failure: **“The clipboard changed before copying finished. Try Copy
again; newer clipboard data is preserved.”** Session/listener/owner-nonzero/queue
booleans are true. The native adapter reached its post-close fresh ownership check
and could not establish the complete HWND/marker/content proof. The exact failed
subcondition and external cause are **not established**; no clipboard monitor,
OS lock or permission cause is asserted. No ownership check was weakened, and no
unrelated clipboard data was read. Native isolated HWND/sequence/marker/digest/
actor/message-pump tests pass, but they do NOT satisfy this final user-station
Copy success/expiry/Lock/exit acceptance gate. An earlier build's observed Copied
confirmation is historical and is not substituted for the final safe rejection.

Reproduce with argument-free `target/debug/examples/codes_list.exe`: fresh generated
fixture (or guarded retained fixture if deliberately requested), unlock only the
public fixture passphrase `disposable-codes-preview` when needed, Tab from search
through Sort/Folder/View to Copy, then physical Copy/Return. F11 writes only the
five booleans above into that owned fixture. F10 writes redacted native frames.
Never use the default Authenticator/reference profile or inspect unrelated clipboard
contents. An unavailable/failed lifecycle listener now gives explicit protection-
unavailable feedback rather than silently skipping Copy; this path has a regression.
The next distinct reviewer should inspect/reproduce this core gap with a fresh
station/window and assess the ownership logic. Task10 is **not self-accepted**.

## Final checks and cleanup

Final current-source `rtk cargo test --locked --all-targets`: **103 passed,3 ignored
helper entrypoints,89 filtered inner tests,14 suites,66.12s**. The three ignored
entrypoints are exercised only by their corresponding subprocess parents. Final
focused `platform::clipboard`5 passed +1 helper entrypoint and `codes_frame_clock`1
passed after the explicit Changed/lifecycle feedback changes. `ui::components`7
passed including equal narrow bounds/full labels/LABEL14/20. Serial capture privacy
group7 passed76.10s, including hidden one-pass AX scrub, held actual save Lock,
normal-close coalescing/error/retry/discard and approved-exit re-entry. Final
format check, strict all-target/all-feature Clippy, all five binary/example builds
and `git diff --check` pass. The final binary was rebuilt after the last source
changes; documentation-only table updates require no test repetition.

The retained-profile normal process exits were observed; the retained
`authenticator-task10-m0U51t` profile/marker/locator were removed by a native
PowerShell script checking exact absolute Temp parent, exact name, no reparse point,
expected marker, exact locator and absence of the unique owned executable. The
fresh `authenticator-task10-tyA82w` capture completed its native framebuffer, but
the helper's final close attempt failed to activate the captured window. Its exact
unique example executable was stopped, then its exact Temp directory was removed
after exact parent/name/no-reparse/allowed-filenames checks. The intended additional
SHA256 check did not run: Get-FileHash was unavailable and PowerShell treated those
errors as nonterminating. Both null results therefore compared equal; the script's
hash-success message is NOT valid evidence and no hash provenance check is claimed.
The exact directory/executable boundary checks did run, and only the known fresh
fixture was removed. This is an honest automation cleanup limit,
not a successful close-input claim. Final filesystem/window checks record no remaining Task10
profile/locator/preview. Earlier owned attempts also auto-removed. The unrelated
vault_lifecycle preview and all default/reference data were untouched. Temporary
probe/cleanup scripts were removed; no commits, pushes, status/coverage changes or
PRD scope changes were made by this author.

Other existing open gates remain: physical/mixed DPI and all monitors, real OS
lock/hardware suspend delivery, fully disconnected-machine journey, Narrator and
unavailable reference pixels. Source/isolated tests do not replace those gates.
Task21 owns the complete later-feature/hardware audit. Shared narrow tabs also
affect editor/locked routes and should receive the distinct reviewer's attention.

## Parent corrections after independent review

Distinct review_10 found P2 shared Codes/Backup Codes search-buffer desynchronization
and P3 invalid-search feedback surviving a valid correction until another operation
replaced it. Parent separated the session buffers and native widget identities;
only Codes updates the encrypted Codes query/filter projection. Both buffers clear
on Lock. Validation now uses current inline feedback instead of a global status,
preserving unrelated notices. The new route-search regression drives actual egui
TextEdit paste/select-all events, verifies visible text and authoritative result IDs
across tab changes, rejects an oversized query without changing encrypted preferences,
corrects it without stale feedback, checks both buffers on Lock and reopens the
encrypted owner. These deterministic events do not replace native input evidence.

Focused `rtk cargo test --locked route_searches --lib`:1 passed,90 filtered,10.42s.
Strict all-target/all-feature Clippy, formatting and diff checks passed after the
corrections. Full regression and focused independent re-review results follow in
the parent review report; the core interactive Copy gate remains open.
