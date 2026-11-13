# Hosted check follow-up during current resume

13 November 2026, coordinating parent. Public GitHub Actions API inspection
found [run43407717785](https://github.com/ranjbar-dev/authenticator/actions/runs/43407717785)
failed on `6ea4c1a11ce5a901abc3d5a841008a11082c696d`. Format and lint passed;
Tests failed and app build was skipped. This supersedes any inference that the
earlier green through20 run covers that later checkpoint.

Authenticated log retrieval used the existing bounded `target/ci-summary.py`
credential helper without printing credentials or forwarding them to log storage.
The old summary returned an empty list because it filtered out the root combined
`0_check.txt`; inspecting that archive member with a bounded failure-line filter
identified the actual result:

- Library204 passed,1 failed,4 directly ignored helpers,886.15seconds.
- `ui::shell::trash_tests::trash_permanent_scope_fault_retry_latest_preferences_and_encrypted_restart`
  failed at `src/ui/trash_tests.rs:1119`:
  `assertion failed: app.preferences_dirty.is_some()`.

The source at that point tests a failed Trash write and preservation of pending
display preferences. It permits wall-clock UI frames before the injected conflict;
whether debounce persistence has already completed needs examination. A timing
cause is a hypothesis, not an established product defect or a passing result.
Do not remove the oracle or claim the requirement verified without a meaningful
replacement covering both pending and persisted preferences.

New [run43427542237](https://github.com/ranjbar-dev/authenticator/actions/runs/43427542237)
on `2a5c83e392fbc4f43a325421bd08ee613f8454c0` was in progress at inspection.
Current native clipboard child remains the only active implementation task;
any required CI correction must be sequential and independently reviewed.

Follow-up: run43427542237 completed **failure** at13:01:29UTC. Format/lint
passed, Tests failed, app build skipped. The same assertion1119 failed:
library205 passed,1 failed,4 direct helpers,887.99seconds. The added mask
regression passed. Both failed run records remain retained; neither is a current
full-suite pass. The single failure is reproducible on hosted Windows and needs
correction rather than another unchanged CI rerun.
