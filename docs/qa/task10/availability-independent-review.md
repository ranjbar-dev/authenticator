# Independent re-review of clipboard availability correction

13 November 2026, distinct reviewer `review_clipboard_current`. Read-only source,
test and evidence review; only this report was written. No native UI/clipboard,
production source, status, commit or push was changed.

## Verdict

Scoped correction approved: no material security/data-loss finding in the reviewed
availability retry, failed-confirmation cleanup or deterministic Trash test setup.
The previously demonstrated advertised-format/NULL-handle rejection path is now
addressed without removing sequence, HWND, random marker or digest safeguards.
This does not complete Task10/21 or certify all platform behavior.

`publish` constructs a candidate after a nonzero initial post-close sequence.
Confirmation uses that same candidate across all four opens; an intervening
writer invalidates it, including an identical-text writer. A successful sequence
update occurs only inside the exclusive interval after full exact proof.
Owner mismatch stops before marker access; marker mismatch/unavailability stops
before Unicode access. Only an advertised but unavailable handle/lock is retried;
missing formats, invalid bounds and mismatches cannot become success. Each retry
releases the clipboard, rechecks admission and waits on the worker, with150ms
maximum scheduled delay rather than a wall-clock guarantee for OS calls.

After a confirmation-stage rejection, `CopyFailure` retains only its descriptor
and the actor gives it an immediate cleanup deadline while reporting failure.
An earlier lease remains if a new attempt fails before producing a replacement
descriptor. Cleanup still checks fixed sequence and owner, exact marker and
bounded digest in one exclusive open interval. Busy cleanup keeps the descriptor;
a later writer is preserved. Cancellation/expiry suppress stale success and
retain only guarded cleanup. Existing bounded shutdown and OS-memory limitations
remain applicable.

One documentation qualification was raised to the parent: a post-close sequence
of zero still fails in `publish` before a usable candidate exists. Therefore the
descriptor guarantee applies to confirmation failures after a nonzero candidate,
not every conceivable failure after publication. The parent agreed to qualify
the wording; no zero-sequence native failure was demonstrated and no unsafe
sequence adoption is recommended to conceal loss of OS authority.

## Independent checks

| Current-source command | Result |
|---|---|
| `rtk cargo test --offline --locked --lib platform::clipboard -- --nocapture --test-threads=1` | **6passed,1direct helper ignored,204filtered,4.44s**; the isolated parent exercises that helper |
| `rtk cargo test --offline --locked --lib ui::shell::trash_tests::trash_permanent_scope_fault_retry_latest_preferences_and_encrypted_restart -- --exact --nocapture --test-threads=1` | **1passed,210filtered,7.59s** |
| `rtk cargo check --offline --locked --release --lib --example interactive_clipboard` | passed0.29s, cached release configuration checked |
| `rtk git diff --check` | passed after this report |

Inspected the actual added tests and their oracles, rather than relying on count:
the private-station helper forces NULL reads at the native boundary then requires
real later marker/digest proof; exhausts attempts and requires a cleanup descriptor;
checks unavailable/corrupt markers cannot authorize Unicode; preserves identical
later writers between retries; and covers admission cancellation. The real-actor
Fake regression requires a failure result without Copied, busy cleanup retention,
eventual own-value removal and preservation of a later identical external copy.
The native parent uses a separate create-only noninteractive window station and
does not exercise the user's interactive clipboard. No full-suite pass is claimed.

The parent-owned Trash change moves pending-intent injection until after opening
Trash, waits600ms beyond debounce, and asserts both pending state and unchanged
committed query/sort/view before the original failure/retry/reopen oracles.
Thus it establishes the tested precondition instead of weakening the failed-write
assertion or changing production autosave. The report correctly distinguishes its
different local slow-navigation failure from the hosted assertion1119. New hosted
full-suite evidence remains required.

## Evidence and remaining limits

Read the corrective report, all six fixed-metadata proof/observation artifacts,
modified probe and complete relevant production/actor/test paths. Final proof
records show unavailable marker at sequence116 followed by exact full proof on
attempt1 at unchanged116; the other Copy shows unavailable Unicode after exact
marker proof at125 followed by full proof at unchanged125. Final observations
record no Unicode at117 after Lock and126 after normal exit. These corroborate
the author's attributed actual keyboard/Lock and pointer/exit journeys; this
reviewer did not repeat them. No nonce, digest, code or plaintext payload is in
the proof artifacts. Debug proof storage/API and probe are excluded from release.

No new screenshots were supplied or saved in this corrective phase. The earlier
two masked600×800 component frames remain the visual scope previously inspected;
there is no new whole-page, three-width or pixel-fidelity verdict. Source-only
calibration remains `docs/qa/task04/calibration.md`: the separate checker found
TITLE400 versus600 in the now-removed outside-checkout worktree; copied-binary
launch failed. It did not prove native keyboard, clipboard or pixel sensitivity.

Native recovery-code Copy, actual OS lock/suspend/clock delivery, disconnected
machine, physical/mixed DPI, Narrator/high contrast, all-page QA and exact reference
pixels remain open. Temporary lack of OS proof can still prevent cleanup, and
an admitted OS call cannot be forcibly revoked. This scoped review is not an
external security certification or an assertion of complete memory/clipboard
erasure under every endpoint condition.

Cost estimate for this re-review:0 new rendered pages, approximately11 project
files/artifacts opened and1 source search. Including the prior focused review:
2 rendered component states, approximately26 file/artifact opens and4 searches
in total; counts are estimates. No additional native process or clipboard access.
