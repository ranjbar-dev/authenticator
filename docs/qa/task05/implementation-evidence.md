# Task 05 implementation evidence

12 November 2026. Implementation ready for the orchestrator's distinct UI and
focused security review; this is not independent acceptance. Task status and
coverage/commits remain parent-owned. Dependencies01–04 were read from current
source and evidence, including task04's source-only calibration and editor fix.

## Boundaries and implementation

- Real local create with confirmation, unlock, explicit Lock, failure/retry,
  clear-fields, busy prevention, Escape cancellation and locked relaunch.
- Exact 12..1024 UTF-8 bytes, no trimming; guidance states this accurately and
  explains irreversible passphrase loss and independently encrypted backups.
- Sole repository ownership moves to a worker; no key/store clone or decrypted
  UI cache. Generation/cancellation rejects late results, including queued
  successes. Argon2 is bounded and cooperative, not interrupted unsafely.
- Create cancellation is synchronized with atomic rename. Before the gate:
  no commit; after rename: complete ciphertext can remain, always requiring
  unlock. No reset/overwrite of existing corrupt ciphertext.
- `Instant` idle expiry defaults300 seconds, checked before new activity.
  Settings saves1/5/15/30 minutes inside encrypted preferences. A diagnostic-only
  three-second option exercised actual expiry, then five minutes was restored.
- Hidden top-level native HWND/current-session WTS/power registrations, passive
  signals/repaint, fail-closed initialization/health and RAII close/unregister/join.
  Never invoked LockWorkStation, suspended the desktop, or changed OS settings.
- Lock removes query, pending workflow, dialogs/help/status and form strings;
  native TextEdit memory is removed. Secret editor undo history is discarded
  every pass. Shared focus now scrolls newly focused controls into view.
- Root forms use existing complete typography/theme bundles and named controls.
  The compact form omits a redundant decorative icon to keep the action near
  the fields. No new private color/type/spacing system.

## Checks and exact commands

All shell commands used the required `rtk` prefix.

| Command | Result |
| --- | --- |
| `rtk cargo check --all-targets` | passed after native handle-zero correction |
| `rtk cargo test ui::state::tests -- --test-threads=1` |5 passed,24.59s; real worker create/unlock/wrong pass/cancel, default/short monotonic boundaries, encrypted timeout persistence, relaunch, corrupt preservation, queued-result locking |
| `rtk cargo test platform::lifecycle::tests -- --test-threads=1` |1 passed; actual private native window receives lock/suspend/resume messages, ignores session-unlock, drops HWND |
| `rtk cargo test cancellation -- --test-threads=1` |3 passed; precommit destination preservation/temp cleanup, gate order and cancelled create/corrupt preservation |
| `rtk cargo test secret_history_tests -- --test-threads=1` |1 passed after correcting the headless test's unhandled texture delta; native undo/state removal verified |
| `rtk cargo test --all-targets -- --test-threads=1` |45 passed,2 ignored helpers,116.68s; parent storage tests invoke both helpers as real subprocess probes; RTK also reports30 filtered subprocess entries |
| `rtk cargo test lock_drops_repository_access_query_and_pending_intent -- --test-threads=1` |1 passed,7.12s after strengthening that test with a real disposable encrypted account/recovery set |
| `rtk cargo test ui::components -- --test-threads=1` |3 passed,0.03s after focus scrolling correction |
| `rtk cargo build --example vault_lifecycle --bin Authenticator` |passed |
| `rtk cargo clippy --all-targets -- -D warnings` |passed |
| `rtk cargo fmt` |applied; final format check recorded below |
| `rtk cargo fmt --check` |passed on final source |
| `rtk git diff --check` |passed |

Final create pre-cancellation check was followed by another targeted
`rtk cargo test cancellation -- --test-threads=1` (3 passed,0.08s), final
build/Clippy/format checks. One attempted example relink hit Windows LNK1104
because the disposable preview was still running; closing only that preview and
rebuilding passed. No process/data cleanup targeted a user application or vault.

Routine first-run build/test corrections: native HPOWERNOTIFY uses zero rather
than pointer null, Clippy preferred inspect_err, the headless egui API is run_ui
and its unapplied texture delta must be cleared. Physical keyboard checks found
Ctrl+L could arrive entirely within one frame; it now matches the event's
modifiers with consume_shortcut. A narrow-view keyboard check motivated shared
gained-focus scrolling. These were corrected before handoff.

## Physical interaction and native accessibility

Used the Computer Use skill's documented sky API from the Node tool, normal
helper launch of the built argument-free `vault_lifecycle.exe`. Closed only the
earlier disposable UI gallery. The test profile is explicitly
`temp_dir()/authenticator-task05-isolated-ui-20261112`, never user LocalAppData or
the installed reference vault. No business fixtures exist in production UI.
No network was required or called by these workflows; OS networking was not
disabled, because changing the user's network settings was unnecessary.

Observed with real pointer clicks and physical key events (no synthetic
element.click, no Unicode typeText dependency):

- Setup fields masked in screenshots and AX, mismatch inline error, correction
  and actual encrypted creation; primary absent during KDF, Cancel available.
- Pointer Lock removes the empty-vault/search interface and focuses empty
  passphrase. Wrong passphrase returns the fixed safe error and clears fields.
- Escape during a real unlock KDF shows cancellation and eventually remains
  locked; a subsequent correct retry unlocks. No duplicate submission is allowed.
- Selected15-minute preference saved, application closed/reopened locked,
  successful unlock and selected15-minute preference retained.
- Actual isolated3-second idle expiry returns to locked form; five-minute default
  restored through real Settings action. No five-minute wall-clock wait claimed.
- Busy Cancel and successful Search receive visible initial focus. Native AX
  snapshots show associated passphrase/confirmation labels with bullet values,
  and locked/wrong-pass/idle snapshots omit credential contents.
- On the corrected final UI, physical Ctrl+L moved Search focus to the locked
  passphrase form (`ctrl-l-locked.jpg`). Physical click into the narrow field,
  then Tab, fully scrolled the focused Unlock action and Clear fields into view
  (`narrow-keyboard-focus.jpg`). Initial same-frame shortcut/focus defects were
  observed and fixed, not counted as passing evidence.

Visual samples cover340/600/960-point windows at100% app zoom, the340-point
window at150%, and340/600/960 at200%, plus the initial420-point setup/error/busy
states. The final preview size commands divide by app zoom, preserving the
requested window size; captures at600/960 with200% are actual600/960-wide
windows rather than enlarged1200/1920-wide substitutes. Top and lower/action
captures show readable labels, wrapping, contrast and reachable controls.
This is seven resize/zoom combinations, not a claimed nine-cell matrix. App zoom
does not change Windows display settings or establish physical DPI behavior.

The sky helper briefly rejected accessibility-index clicks with “foreground
window did not report a process id”; refresh/activation and screenshot-coordinate
physical clicks succeeded. One immediate post-resize scroll was rejected as
out-of-bounds while Windows still reported the old width; a fresh observation
confirmed the new size before continuing. No stale-coordinate action was retried
without reobserving. The preview was closed after final capture and rebuilt;
the isolated encrypted profile remains for the independent reviewer.

Screenshots are original window captures containing only disposable/empty UI,
masked form values and fixed messages. `create-420.jpg`,
`confirmation-error.jpg`, `create-busy.jpg`, `created-empty.jpg`,
`unlock-420.jpg`, `wrong-passphrase.jpg`, `cancelling.jpg`,
`relaunch-locked.jpg`, `unlock-busy-focus.jpg`, `idle-setting-saved.jpg`,
`idle-setting-relaunched.jpg`, `idle-locked.jpg`, and
`default-idle-restored.jpg` document those states. `*-ax.txt` contain sanitized
native trees. The `unlock-WIDTH-ZOOM*.jpg` captures document resize/app zoom;
additional lower/action captures show vertical access at compact large text.

## Explicit limitations and review handoff

Actual OS session locking and physical hardware suspend/resume were **UNVERIFIED**.
The private-HWND message-path tests establish adapter routing/cleanup, not
hardware delivery. Physical DPI/multiple monitors, Narrator/high contrast and
exact reference pixels remain **UNVERIFIED**. No reference vault was opened.

Current production account/capture/import/revealed recovery views do not exist
yet. Lock denies the real populated disposable repository and clears current
pending intent/query, but the future complete capture/import/revealed-code
journeys must exercise generation/owned-buffer cleanup in tasks07–10/16/21.
Corrupt-file preservation is an actual worker/storage test; no separate corrupt
file was substituted into the successful UI test vault. Portable backup is not
implemented by setup and remains task11 onward.

Egui history is dropped rather than securely zeroized internally; no complete
allocator/compiler/OS memory-erasure claim. Cancellation cannot stop a running
Argon2 call, so bounded worker buffers drop when it returns; UI/AX remain locked.
The native rename is the brief synchronized commit section, so cancellation
arriving after that gate can wait on the OS call and leave complete ciphertext.
Source-only shared-role calibration from task04 applies to unchanged typography;
the implementer did not self-calibrate or self-review task05. Parent must dispatch
a separate checker with the task05 screen checklist and verifying skill, and a
focused lifecycle/security review, before acceptance/status changes.

## Parent integration and independent review

Distinct review_04 inspected lifecycle/state/cancellation/nativeunsafe/forms/18saved screenshots and live isolatedwrongpass/unlock/CtrlL. Independentlypassed5state,1native,3cancellation,3editor tests,fmt andalltargetClippy. No materialfindings. IndependentliveEscape arrivedafterKDFcomplete and isnot a cancellationpass; authorlive and real-KDFautomatedtest evidenceabove remain distinct. Parent replacedUTF8-byte-leadingproductguidance withplainlanguage and accurate short/long errors; bytepolicy unchanged incode/docs. Reviewer approved thiscopy-onlysourcecheck; finalfmt/lockedalltargetClippy/diff checks pass. PhysicalOSlock/suspend andlaterworkflow gates stayopen in task05; no fulltaskcompletion/hardwarecertification claim. CommitSHA follows in nextcheckpoint.

Task08 integration update: distinct review_08 subsequently verified real capture,
editable QR/manual review, atomic save/current codes, picker cancellation and
locked UI/AX cleanup, with storage/worker stale/cancel regressions passing.
Capture/private-WTS cancellation is evidenced in task07. Actual OS notification
delivery remains unverified; later clipboard/backup-code workflows need their
own integration checks. Parent corrected the task05 disconnected-machine
acceptance checkbox to open, consistent with this report's explicit statement
that OS networking was not disabled. Observed local behavior and absence of an
application network dependency do not prove that requested manual gate.

Task09 corrected a pre-existing responsive-lock issue in the shared cancellation
gate: an admitted filesystem action no longer blocks UI cancellation. Independent
deterministic real-save testing proves generation/query/review/widget/AX cleanup
before the held action is released, then locked late completion and complete
encrypted data after explicit unlock. Active worker keys/buffers can remain until
uninterruptible I/O/crypto returns. This is not actual OS-lock/suspend evidence.
