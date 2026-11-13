# Independent review of resumed Task05 evidence

13 November 2026. Distinct reviewer `review_resume05`, read-only production/source
review, using `ui-consistency:verifying`. This approves only the bounded new
Task05 evidence; it does **not** complete Task05 or approve the full QA harness
for arbitrary revealed screenshots.

## Findings

**P2 — account review screenshots can be incorrectly labeled sanitized.**
`examples/workflow_qa.rs` F10 exports every current frame to
`sanitized-frame-*.png` and writes a `sanitized` readiness marker. Its sole mask
input is `AuthenticatorApp::code_bounds_for_evidence()`.
The production debug bounds cover current OTPs, standalone recovery editor
(`shell.rs` around1708) and recovery detail (around1811). However,
`import_review::Editor::show` can deliberately render the Base32 secret after
Reveal, and account creation can reveal its optional backup-code multiline
(`shell.rs` around2711), without enrolling those field rectangles in that list.
Thus F10 on either revealed account-form state can persist readable disposable
secret material while claiming sanitation. This is a development evidence
capture defect, not an established production storage vulnerability. The two
new Task05 images show no such content and are safe in their observed states.
Before full21 evidence capture, mask these fields or refuse captures of those
states; do not rely on the filename/readiness marker as proof of sanitation.
No real vault or credential was used to reproduce a leak, and no leaking image
was created by this review. The finding follows the complete relevant field and
mask-enrollment source paths.

The harness also retains visible metadata by design. Only clearly disposable
account labels may be captured; it is not a general PII sanitizer. Its isolated
TempDir and explicit `VaultStore::open_directory` keep the default user vault
outside this harness's ordinary launch path.

## Independent evidence

- Opened both new600×800 actual framebuffer files: `resume-created-empty.png`
  and `resume-locked.png`. Their holder/header/card hierarchy, readable text and
  blue/dark foreground pairs, opaque active icons, actions and focus are
  consistent with the shared theme and PRD inventory. No clipping or exposed
  credential is visible in these two states. Exact reference pixels were not
  available; the new app's frames are not reference evidence.
- Fresh process query matched PID44548, executable
  `target/debug/examples/workflow_qa.exe`, creation13Nov12:40:27 and the exact
  `target/task21-owned-8lrp7u/ownership.txt` manifest. Fresh `sky.list_windows`
  returned one owned window526350. No unrelated/reference/default vault was
  opened or changed.
- Fresh `@oai/sky` window screenshot and native accessibility observation showed
  Unlock vault, static recovery guidance, three tabs, an empty labeled Vault
  passphrase field and no account/OTP/recovery content. The field had focus.
  After a fresh activation/observation, real `sky.press_key(Tab)` moved focus
  to Unlock vault. No unlock, data mutation or new screenshot file occurred.
  Native phase ended with the owned app still locked.
- Independently ran
  `rtk proxy python target/task21-run.py lib ui::shell::capture_privacy_tests::lock_clears_ui_before_an_actual_admitted_save_action_is_released --exact --nocapture --test-threads=1`:
  **1passed,0failed,208filtered,7.75s**. The inspected wrapper used the existing
  source-through20 test executable, fresh owned runtime and bounded process wait;
  this was not a fresh build or full suite. Runtime
  `target/task21-runtime-38f032ce-1b0b-40e3-9dbb-4176a574badb` retained with log and
  manifest. Fresh process query found testPID29412 absent. No deletion required.

The author correctly records its one zero-test filter attempt separately. Its
native Create/busy/Lock observations are attributed author evidence; this reviewer
did not independently repeat creation. Source confirms busy submit removal,
masked shared fields, safe local recovery copy, form/widget cleanup, generation
invalidation and cancellation/Lock paths. No new production defect was found in
this bounded review.

## Unevaluated and acceptance

Create, wrong-passphrase, corruption, busy/cancel, relaunch, populated hiding,
capture/import/recovery interactions, real Copy and all-page/narrow/zoom states
were not independently exercised in this resumed review. Offline-machine,
actual OS lock/suspend delivery, physical/mixed DPI, Narrator, high contrast and
exact reference fidelity remain unverified. Private-window notifications and
held-save framework tests do not replace those gates. Leave both open Task05
acceptance boxes unchecked.

Existing same-kind calibration is `docs/qa/task04/calibration.md`: separate
checker detected TITLE weight400 instead of600 in an outside-checkout worktree.
Copied binary launch failed; the proof covers source bundle checking, not live
pixel sensitivity, geometry or input. No new calibration was performed.

Cost:2 rendered saved states compared plus1 native locked state/keyboard step;
approximately13 project files opened,7 source searches and4 skill/reference
documents read. Counts are estimates; this is not a whole-app visual audit.
