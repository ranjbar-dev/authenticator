# Pending-preference Trash test correction

13 November 2026, parent integration fix after clipboard implementer FINAL.
Production Trash/preferences behavior is unchanged.

Hosted runs43407717785 and43427542237 failed the same assertion that display
preferences remained pending after a failed permanent-delete write. The test
changed preferences while on Codes, then rendered Settings repeatedly before
opening Trash. The real500ms autosave can legitimately save that intent during
those frames. A pending-only assertion therefore lacked a stable precondition.

The corrected test opens Trash before injecting the pending display projection.
It deliberately waits600ms and renders again, asserts that intent is still
pending and committed query/sort/view remain original, then retains the existing
real worker/fault/retry/encrypted-restart and association-preservation oracles.
It does not remove the failed-write dirty-state assertion, alter debounce logic
or claim that a previously saved preference must become dirty again.

Commands and observed results:

- Original setup plus a600ms slow-frame delay: one selected test failed after
  4.56s execution at `Trash Settings action`, with autosave active
  during navigation. This is an additional timing-dependent failure, **not** a
  local reproduction of hosted assertion1119. The two hosted logs remain the
  evidence for that exact assertion.
- First corrected compile failedE0369 because `CodesPreferences` has no
  `PartialEq`; changed the test to compare the three affected fields individually.
  No domain trait or production behavior was changed to satisfy the test.
- `rtk cargo test --offline --locked --lib ui::shell::trash_tests::trash_permanent_scope_fault_retry_latest_preferences_and_encrypted_restart -- --exact --nocapture --test-threads=1`:
  corrected source **1passed,210filtered,7.99s**.

Independent re-review and new hosted full-suite evidence remain required.
