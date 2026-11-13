# Task15 parent direct-delete focus correction

13 November 2026. Distinct review15 found one P2: Delete Set directly from detail
opened the editor-owned confirmation but stored the detail button's ID as the
opener. That button disappeared, so cancelling could leave `focused=None`.
Stored data was unchanged. This corrects the existing focus/action checklist
position; it introduces no new style or component.

The detail action now requests deletion on editor opening without carrying the
vanished ID. When the editor renders its actual Delete control, it binds and
opens the confirmation there. Keep editing, Escape and backdrop cancellation
deliberately return to that visible editor control. Delete initiated within the
editor retains its existing actual-opener behavior. The obsolete detail-opener
payload was removed from the internal action; account/set ID bindings, mutation
guards, confirmations, encryption and cancellation admission are unchanged.

`recovery_direct_detail_delete_cancel_returns_to_current_control_at_normal_viewports`
passed1 test in33.47s. It uses Context keyboard activation plus backdrop pointer
press/release through actual list/detail/editor ownership boundaries, at340x780,
600x800 and960x900, for both normal and oversized read-only sets. It verifies safe
modal focus, each of the three cancellation routes, actual visible replacement
control focus/bounds, real Cancel/Discard on exit, unchanged full record guard
and identical committed ciphertext. Events/AX remain synthetic, not native proof.

Corrected `rtk cargo test --locked --all-targets --all-features` passed168 tests,
4 helper entrypoints exercised by their parents,18 suites110.19s. Strict
all-target/all-feature Clippy, fmt/diff and production/lib/owned-editor-example
all-feature build passed on this final source. Focused independent re-review is
approved: new normal-viewport regression1 passed36.66s, original inside-editor
Cancel focus1 passed9.37s, diff check passed; no remaining material source finding.
Native input/picker/pixels/GUI restart and prior manual gates remain
open. No new native attempt, capture or calibration was performed. Parent fix
reading cost is approximately5 project files and2 grouped searches; earlier
author/reviewer counts remain in their reports.
