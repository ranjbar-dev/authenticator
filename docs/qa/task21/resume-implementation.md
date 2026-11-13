# Task21 resumed implementation evidence

13 November 2026. Implementation child `resume_21`; independent review remains
required. Task21 acceptance is incomplete. No default/reference vault was opened.

## Current source fix and checks

Fixed QA-only P2 from `docs/qa/task05/resume-independent-review.md`: manual/QR
secret fields and optional creation-time recovery text now contribute actual
painted field geometry to debug evidence masks. Masks survive Lock until the
next paint pass, because already-emitted shapes can outlive their form.
Production values, validation, encryption and interaction styles are unchanged.

- `rtk cargo fmt --all`: passed.
- `rtk cargo test --locked --lib --no-run --target-dir target/task20-isolated`:
  passed, current source test executable rebuilt.
- `rtk proxy python target/task21-run.py lib ui::shell::capture_privacy_tests::account_evidence_masks_revealed_seed_and_creation_backup_through_lock --exact --nocapture --test-threads=1`:
  1 passed,0 failed,209 filtered,5.84s. Actual revealed text shapes must be
  contained in masks for Create/Import at340/600/960 and1/1.5/2 app zoom. Form
  removal/Lock retain masks until the next locked frame. No image was written.
- `rtk cargo clippy --locked --all-targets --all-features --target-dir target/task20-isolated -- -D warnings`:
  passed.
- `rtk cargo build --locked --example workflow_qa --example qr_fixture --target-dir target/task20-isolated`:
  passed, own examples built.
- `rtk git diff --check`: passed before native phase.

## Native milestones (in progress)

Verified the old owned process44548/executable/start/ownership manifest and
closed its locked window through real Alt+F4. Fresh inventory confirmed its
absence. Launched the newly built isolated `workflow_qa.exe` through the normal
Computer Use helper; selected exactly one returned window591900. The helper
uses its own TempDir and actual native HWND. No unrelated process was modified.

Native `sky.set_value` works on egui fields; `type_text` previously did not.
This is documented UI Automation field entry, not a physical-key proof. Actions
include actual pointer clicks, Return, scrolling and region drag. Each action
uses fresh returned state. No clipboard text-entry workaround was used.

1. Clean Create vault with masked confirmation succeeded. Busy/Cancel state
   became empty Codes with Search focused.
2. Launched separate public RFC-seed QR fixture, generated in memory only.
   Screenshot to Scan hid the app and displayed the native region overlay.
   First drag used an incorrect helper coordinate frame and returned actionable
   No QR; no mutation occurred. Existing task07/08 describe this helper's
   horizontal normalization. A corrected real drag of the fixture region
   produced masked Review account with Public Fixture/disposable andSHA1/6/30.
   No source/overlay screenshot or QR payload file was saved.
3. Changed service to `QA Screenshot Account`, retained concealed seed, scrolled
   to the explicit Save action, and clicked Save selected as new accounts.
   App reported encrypted import success and showed one real current-code card.
   One offscreen-click attempt was rejected by the helper before action; actual
   scrolling and the later visible pointer click completed the save.
4. Independent Node `crypto.createHmac('sha1', published RFC seed)` matched the
   actual displayed six-digit code at Unix1794565410. Observation start/end
   were the same second and30-second counter. Only match=true, settings and
   timestamp are recorded; no OTP value is logged.
5. F8 resized the native window to960×900 content. Real pointer Copy produced
   `Copied. The app-owned clipboard copy will expire automatically.` The
   production actor verifies HWND+nonce+digest before reporting success. No
   production clipboard source was changed; old Changed cause remains unknown.
   This step does not itself prove expiry/Lock/exit or newer-copy preservation.

Remaining native milestones and evidence files will be appended as observed.

6. Native Settings → Export & Import → Export encrypted backup: selected a new
   owned `.authpbk` destination through the native Save dialog, entered a
   separately masked backup passphrase and confirmation, explicitly Exported,
   and observed busy then `Encrypted backup saved`. File:
   `target/task21-backup-36187a15-1d1f-4d34-9d66-7ff0d06d2427.authpbk`.
   Source owned PID11368/root `target/task21-owned-F9S4b6` was verified from its
   manifest and executable/start metadata. Backup contains one account only;
   no full entity-graph native round trip is claimed by this step.
7. Launched a second fresh isolated QA instance (returned window134612366),
   created a clean vault with a different local passphrase/random keys, then
   selected that exact export through the native Open dialog. A deliberately
   incorrect backup phrase failed safely, cleared its field, and offered retry.
   Correct phrase authenticated a1-account/0-other-record preview with safe
   Merge, retained local display/security/startup and explicit exact review.
   Visible Apply succeeded with a current-code card for the renamed account.
   A second independent Node HMAC matched at Unix1794565892, same observed
   second/counter, match=true. No original local passphrase/key was needed.
   Wrong-phrase UI behavior is observed; exact ciphertext unchanged requires
   the separately recorded source/fault tests, not an invented native hash check.

## Limits

No WindowsSandbox.exe was present and read-only Get-VM listed no available VM.
Shared network adapters, OS lock/suspend, display settings and security settings
were not changed. Networking-disabled clean-machine, physical/mixed DPI,
Narrator/high contrast and exact reference-pixel acceptance remain unverified.
Native exports/restores and later-feature journeys are still in progress.
