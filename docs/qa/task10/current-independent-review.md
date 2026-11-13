# Independent review of current clipboard evidence

13 November 2026. Distinct read-only reviewer `review_clipboard_current`, using
`ui-consistency:verifying`. No production source, task status, native window,
clipboard content, commit or push was changed. Only this report was written.

## P2 — rejected published Copy remains an acceptance blocker

The author's quiet-observer run records app HWND ownership, lease-format
presence and a matching generated public-fixture code at sequence73/time1794574732.
Its `after_exit_cleanup` record at1794574800 has the same sequence, matching
fixture value and `unlocked=false`. The author separately observed the exact
Changed feedback immediately after keyboard Space on the focused Copy button.
The saved images do not capture that transient feedback. These are attributed
native observations, not a native repetition by this reviewer.

Source confirms a relevant error path: `native::copy_after_close` publishes
Unicode text, closes/reopens the clipboard, and returns Changed when
`owned_content` is false. The actor's error arm reports failure without adopting
a new lease. Thus that rejected attempt has no newly retained cleanup descriptor
for later expiry/Lock/shutdown. This identifies a source path needing diagnosis;
it does not establish which content-proof condition failed during the observed
native run, or prove that the visible data retained the exact original nonce.

Do not call this a demonstrated failure of **all successfully leased** Lock
cleanup. The separate sequence-aware pointer run records Copy success, sequence64,
and Unicode disappearance/sequence65 in the same second as Ctrl+L. The earlier
unsequenced anomaly and immediate-observer Busy attempts are correctly separated
in the author's report. The initial observer could itself contend for the
clipboard. The final250ms quiet period reduces that particular interference but
does not make the diagnostic invisible to OS scheduling.

Task10's keyboard-Copy and complete clipboard-lifecycle acceptance must stay
open. The same native adapter also serves recovery codes, so its error handling
deserves focused review before acceptance; this review did not reproduce this
failure with a recovery code or establish permanent-secret exposure.

## Safe next diagnostic and repair boundaries

Before selecting a behavioral fix, add debug-only, fixed-size reason metadata
at the actual failing native proof branch. Suggested enum/booleans:

- Copy attempt reached text publication and post-close open result.
- Owner matches; only then marker handle/bounds/lock/equality result.
- Only after owner **and exact marker** match: Unicode handle/bounds/lock result
  and digest equality.
- Written/post-close/proof sequence metadata and admission state.

Record only the failing branch and booleans/sequence numbers, never handles,
nonce bytes, expected/actual code, digest, seed, passphrase or arbitrary clipboard
text. Preserve short-circuit read authority: a failed owner/marker proof must not
authorize reading Unicode merely to improve diagnostics. Keep it excluded from
release and use one bounded actual keyboard reproduction with a quiet observer.

A provisional descriptor retained solely for guarded cleanup is a possible
design to evaluate after the failure is understood. It is **not approved here**
as a sufficient repair. Every eventual clear must retain the exact sequence,
HWND, random marker and bounded digest checks in one exclusive open interval.
Never rebase to an arbitrary later sequence, clear another writer, report success
after failed proof, or hold/read/free a published memory handle after its valid
lifetime. Add meaningful failure-path tests and independently re-review any fix.

Microsoft documents sequence changes and delayed rendering in
[GetClipboardSequenceNumber](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclipboardsequencenumber),
and system ownership of successfully published memory plus implicit format
conversion in
[SetClipboardData](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setclipboarddata).
Those facts do not establish that conversion, a clipboard manager, the input
helper or another particular component caused this run's rejection.

## Independent checks and evidence integrity

Independently executed current-source commands:

| Command | Result |
|---|---|
| `rtk cargo test --offline --locked --lib platform::clipboard::tests::actor_expiry_hidden_lifecycle_lock_shutdown_and_newer_identical_external_copy -- --exact --nocapture --test-threads=1` | 1passed,209filtered,1.31s |
| `rtk cargo test --offline --locked --lib isolated_native_clipboard_process -- --nocapture --test-threads=1` | 1passed,209filtered,0.06s |
| `rtk git diff --check` | passed after this report |

The second parent case runs its existing helper in a separate noninteractive
window station, with actual native same-value external-writer/marker/digest/
ownership cases. These passing baseline tests do **not** reproduce or repair
the current interactive Changed residual. No full suite is claimed.

Read the entire new `examples/interactive_clipboard.rs` and three JSONL artifacts
(initial11, sequence-aware7, quiet-observer3 records). They contain booleans,
timestamps and sequence metadata, no clipboard strings or digests. The observer
only compares bounded Unicode under an exclusive open for its own HWND with
lease-format/queued proof or its previously verified unchanged sequence; the
separate harmless later writer has a different owned HWND. This is disposable
fixture evidence, not proof of the actor's exact nonce/digest. The actual default
vault path is never selected. The code/sample account lives only in an isolated
encrypted test store, not production frontend content. Release/non-Windows
diagnostic entry is inert.

Opened both actual600×800 PNGs (`sequence-run.png`, `quiet-observer.png`) and
compared their visible card/toolbar regions as a set. Both mask the code before
disk writing and show a visible Copy focus border, opaque active icon tile,
readable service/account/countdown and separate Copy/Edit/Delete controls.
The top controls are intentionally scrolled; these frames cannot certify the
whole page, header, three widths, error state or locked-state rendering. No
additional visual deviation is identified within these two regions. Exact
reference pixels are unavailable; they remain PRD-inventory comparisons.

The author's pointer expiry/newer-copy/pointer-Lock observations are useful
bounded evidence and its failure/unknown distinctions are accurate. This review
approves integrating the **diagnostic and honest partial evidence**, not declaring
Task10 complete or certifying the unchanged clipboard implementation.

## Unevaluated and calibration

No independent native keyboard/clipboard repetition or new process-identity
verification was performed; no default/reference vault, initial user clipboard
or unrelated window was touched. Success-path exit without prior Lock,
keyboard Copy success, exact rejection cause/repair, backup-code native copying,
all-page layouts, physical/mixed DPI, actual OS lock/suspend/clock delivery,
network-disabled execution, Narrator/high contrast and precise reference fidelity
remain open.

Existing same-kind calibration used the now-removed detached worktree
`C:/Users/root/AppData/Local/Temp/authenticator-ui-cal-4cg1yjfn/checkout`.
A separate checker detected the TITLE400-versus600 plant; copied-binary launch
failed. That establishes source typography-bundle checking, not runtime pixel,
scroll, keyboard, clipboard or geometry sensitivity. No new calibration was
performed by this reviewer.

Cost estimate:2 rendered component states compared as a set, approximately15
project files opened including evidence/images,3 source searches,1 skill and
2 official API documents read. This is not a whole-app audit or external
security certification.
