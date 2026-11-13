# Independent review of current native resume and evidence masks

13 November 2026. Distinct reviewer `review_current_native`, read-only source
and evidence review using `ui-consistency:verifying`. No production files,
task statuses, commits, user settings or native windows were changed.

## Verdict and findings

No material finding in the bounded source repair or two newly supplied locked
frames. The earlier QA-only P2 in
`../task05/resume-independent-review.md` is resolved in the reviewed source:

- `src/ui/import_review.rs` records the actual shared secret TextEdit response
  rectangle on every painted Create/Import/Edit field pass, whether concealed
  or deliberately revealed. It resets its rectangle before rendering, avoiding
  reuse when an edit keeps the existing secret instead of showing an input.
- `src/ui/shell.rs` enrolls that rectangle and the optional creation-time
  recovery multiline response rectangle in the same per-frame mask list as
  existing current OTPs, standalone recovery editing and recovery detail.
- Lock keeps already-painted geometry until the next `show` pass. That pass
  clears the list before painting the locked screen. Removing the form after
  emitting its shapes therefore does not remove the masks for that frame.
- The new geometry field and enrollment/API remain under
  `cfg(debug_assertions)`. They introduce no credential serialization, network,
  clipboard or vault mutation. The non-debug field behavior still derives
  `changed` from the same shared TextEdit response.

The argument-free `examples/workflow_qa.rs` opens an explicitly owned TempDir,
not the default vault. F10 obtains masks after `app.show` and applies them to
the screenshot pixels before its only PNG write. The pinned egui screenshot
API invokes its callback after the integration reads rendered pixels; mask
coordinates are converted using current pixels-per-point and padded. The
harness intentionally retains metadata and is suitable only for disposable
records. Its filenames/readiness marker are not a general PII-sanitization
guarantee. This review did not persist any revealed credential frame.

## Independent checks

Executed against current source:

```powershell
rtk cargo test --offline --locked --lib ui::shell::capture_privacy_tests::account_evidence_masks_revealed_seed_and_creation_backup_through_lock -- --exact --nocapture --test-threads=1
```

Result: **1 passed, 209 filtered, 10.58s**. This was a real selected test,
not the author's documented earlier zero-selection attempt. The regression
requires revealed seed/recovery text actually to be painted and its clipped
shape geometry contained in evidence masks at340/600/960 and1/1.5/2 app zoom.
It also checks form-removal/Lock geometry retention and next-locked-pass reset.
It writes no image. It does not certify physical Windows DPI or native
screenshot scheduling under every resize/input race. No full-suite run is
claimed by this focused review.

Opened and compared both actual600x800 PNGs as a set:

- `../task05/current-wrong-phrase.png`
- `../task05/current-keyboard-lock.png`

The holder/header/navigation/card order, whole typography hierarchy, opaque
blue actions with dark text, shared error treatment, labeled empty passphrase
field and visible focus agree with Task05's checklist and the shared shell
inventory. No visible credential values, overlap or clipping in these two
states. The additional error holder accounts for the taller wrong-phrase card;
the primary action remains visible. These are the new application's frames,
not reference-app screenshots. No precise reference-pixel comparison is possible
from the supplied inventory.

## Attribution and remaining acceptance

`../task05/current-resume.md` accurately bounds its author-observed native
Create/busy/Lock/wrong-passphrase/retry/Ctrl+L evidence and records the helper
text-entry/Tab limitations. This reviewer did not repeat those native actions
or verify their process identity; no native phase was needed for this focused
source/image review.

`resume-implementation.md` is attributed evidence from former implementer
`resume_21`. Its actual pointer screenshot import, SHA1/6/30 HMAC matches,
Copy success feedback, native export and fresh independently keyed Merge
restore are useful bounded author evidence. They are not independent
repetitions by this reviewer. The exported/restored dataset has one account;
the report correctly does not claim the complete folder/recovery/used/Trash
graph, Replace/conflict workflow, clipboard expiry/newer-copy/Lock/exit checks,
or a networking-disabled machine. Those gates remain open.

This review approves integrating the mask repair and the bounded current
lifecycle evidence. It does **not** complete Task05 or Task21. Full native
backup-code/folder/Trash journeys, every-page size/open/error/focus states,
keyboard-only reachability and actual clipboard lifecycle still need evidence.
Networking-disabled clean-machine execution, actual OS lock/suspend delivery,
physical/mixed DPI, Narrator/high contrast and exact reference pixels remain
unverified. No filesystem-wide sentinel, crash, traffic or packaged-app audit
was performed here. Retain the task acceptance boxes accordingly.

Existing same-kind calibration was a detached worktree at
`C:/Users/root/AppData/Local/Temp/authenticator-ui-cal-4cg1yjfn/checkout`, now
removed. A separate checker detected the planted TITLE400-versus600 difference;
copied executable launch failed. That proves source typography-bundle checking,
not pixel, scroll, interaction or runtime contrast sensitivity. No plant or
new calibration was performed by this implementation reviewer.

Next useful work is the remaining real clipboard/recovery/organization/Trash
and complete native recovery journeys, followed by independent discovery and
packaging where hardware-only gates cannot be exercised. No unchanged hardware
retry or source-test count should be used to close those gates.

Cost estimate:2 rendered states compared together, approximately26 project
files opened (including checklist/evidence files and2 images),2 source searches
plus checklist inventory,1 pinned dependency API read and1 skill read. No whole
application visual audit or accessibility-conformance verdict is claimed.
