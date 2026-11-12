# Task09 independent review

12 November 2026. Distinct read-only `review_09` returned two findings before
acceptance, then independently confirmed both corrections. Task09 is accepted
within its functional scope. It was neither the author nor the task04/07 calibration agent. Parent
inspected the diff, relevant source and representative sanitized components.
The findings below describe the first review; correction and recheck must be
recorded separately rather than presented as an original pass.

## Findings at first review

- **P2: responsive lock.** `OperationCancellation::commit` held the cancellation
  mutex during the admitted filesystem replace/MoveFileExW WRITE_THROUGH action.
  Synchronous `cancel`, called by `AppState::lock` and then `Shell::lock` before
  generation/query/review/widget cleanup, waited for that same mutex. A stalled
  admitted replace could therefore delay UI lock completion. The existing gated
  action test independently demonstrated cancellation waiting for release. No
  actual kernel/filesystem hang, ordinary visible-OTP failure or data loss was
  observed. Required correction: nonblocking cancellation/admission plus a
  deterministic actual-save test proving UI/state lock and privacy cleanup finish
  before the admitted commit gate is released. An admitted complete encrypted
  save remains permitted; cancellation must never produce a partial batch.
- **P3: stale duplicate feedback.** Actual failed Save for unacknowledged recovery
  duplicates followed by approval changed the button to acknowledged but left
  the red global message requesting acknowledgement. Subsequent Save worked.
  Required correction: clear contradictory feedback on the relevant correction,
  preserving acknowledgement invalidation when text changes and retained input.

These findings prevented Task09 acceptance. The P2 mechanism predated Task09 and
was reused by its generalized worker. The parent requested sequential correction
from implement_09 after the review final, then a focused independent re-review.

## Independent evidence that passed

- Components6; import review3; all24 create/edit/persistence1 (48.69s);
  validation/stale/replacement/association1; atomic recovery/create/edit faults1
  (9.50s); concealment/lock/popup1 (6.74s); normalization, contrast and existing
  cancellation-gate test1 each; formatting and diff checks.
- Actual SendInput pointer/key checks on only owned temporary profile
  `authenticator-task09-aK6lvX`: stable-ID edit, existing-seed default, rename,
  Advanced at340, selected-row contrast, SHA256/60 choice and restored focus,
  Escape popup priority, failed-write retained edits, ciphertext restore/retry,
  busy state, manual creation, conceal/reveal, optional code input and duplicate
  approval/save, Tab out of multiline, Keep editing/Discard, F5 repository reopen
  and Ctrl+L removing metadata/focusing the empty passphrase field.
- Live layouts340/600/960. Representative author images reviewed included
  Advanced600, duplicate600, corrected-validation600 and twenty-lines340/600.
  Manual and QR forms share the same field tree, theme and typography.
- Independent standard-library HMAC matched actual edited SHA256/6/60 at
  Unix1794511203, counter29908520, within a stable time bracket. Only the boolean
  result/metadata are recorded; no seed or OTP artifact was created.
- Source review covered save/normalizer/cleanup/security boundaries. Full-node
  tests distinguish deliberate Reveal text-run exposure from concealed and
  re-concealed protection. No additional security/data-loss blocker was found.

The reviewer closed its owned window and observed its profile/locator absent,
without source/docs/status changes, commits, clipboard access or real/default/
reference vault access. Estimated cost: two Account modes plus shared QR source,
approximately23 project/doc files and14 searches.

## Scope limits

Independent process relaunch was not repeated; the separate author's F9/new-process
proof and guarded example source were reviewed. F5 was explicitly not counted as
process restart. The reviewer's live multiline held two lines; author20-line
screenshots/source and the independent40-line geometry/AX test supplement it.
Physical/mixed DPI, disconnected-machine operation, actual OS lock/suspend,
Narrator and reference pixels remain open in their owning tasks. Calibrations
detected planted source typography defects, without general pixel-sensitivity
claims. This is agent review, not external security certification.

## Correction and focused re-review

Both findings resolved; focused read-only re-review found no remaining issue.
Cancellation is now an atomic request competing with CAS admission, without
acquiring the filesystem-action mutex. Separate worker serialization and RAII
preserve token reuse and release admission after success/error/panic. UI/state
cleanup occurs first; cancellation/polling use try_lock. The reviewer checked the
producer's post-publication recheck, missed UI slot acquisition, later cancellation
and stale polling: an unlocked completion cannot reopen a cancelled session.
Active worker key/plaintext/I/O lifetime limits remain explicit.

Acknowledgement now clears corrected backup-specific feedback and its scroll flag;
disabling optional backup input does likewise. Other errors, input and exact-text
acknowledgement invalidation remain intact.

Independent focused commands passed:

```text
rtk cargo test --locked lock_clears_ui_before_an_actual --lib -- --test-threads=1
  1 passed,10.12s
rtk cargo test --locked cancellation_ --lib -- --test-threads=1
  5 passed: nonblocking admission/publication, precommit, reuse/panic
rtk cargo test --locked duplicate_acknowledgement_and_toggle --lib
  1 passed
rtk cargo test --locked account_with_recovery_and_edits_are_atomic --lib -- --test-threads=1
  1 passed,11.19s
rtk cargo fmt --all -- --check
rtk git diff --check
  passed
```

The real-save gate regression proves UI/AX/query/history cleanup **before gate
release**, rejects late success/unlock, then confirms both accounts persist
together after explicit unlock. This is deterministic held-action evidence; no
actual filesystem hang was claimed. The author's final full suite passed88 ordinary
tests with the two existing helper entrypoints exercised by subprocess machinery.

The final corrected600 screenshot and complete follow-up evidence were inspected.
Independent final pointer repetition was unavailable: two documented preview-launch
attempts returned `GetCursorPos failed: Access is denied (0x80070005)`. No settings
change or bypass was attempted. Earlier independent340/600/960 interactions remain
as reported above; final P3 pointer evidence is author-provided, supplemented by
independent source/test verification. No review preview, profile or locator remained.
Focused follow-up cost: approximately10 files/artifacts and4 searches.

Parent accepted Task09 after this final re-review. No global05/07/08/21 offline,
hardware, Narrator or reference gate is closed by that acceptance. Commit reference
is recorded in the following checkpoint.
