# Native clipboard availability correction

13 November 2026. Implementation child `resume10_clipboard`, following distinct
`current-independent-review.md`. Ready for independent security/data-loss review;
no task completion, commit, push or status change by this child.

## Established failure, without guessing its OS cause

Added debug-only fixed-size native proof snapshots at the actual confirmation
branch. An actual keyboard Tab→Copy→Space reproduced Changed with owner equality,
private-marker handle/bounds/lock/**exact bytes** all true, but Unicode
`GetClipboardData` returning NULL. Written sequence78 became81 after close;
sequence remained81 before/after the failing proof. The later observer matched
the published fixture code at81. No Unicode was read by the failed proof.
See `availability-fix/before-native-proof.txt` and `before-observations.jsonl`.

The first guarded availability correction then exposed the corresponding private
marker branch: its advertised format existed, owner/sequence matched, but the
marker handle was temporarily NULL. No Unicode read followed. Evidence is kept
in `intermediate-native-proof.txt` and `intermediate-observations.jsonl` rather
than treating that rejection as an external/platform impossibility.

These observations establish temporarily unavailable reads on this desktop.
They do not establish that the input helper, implicit conversion, a clipboard
manager or a particular Windows component caused them. No such cause is claimed.

Current Microsoft documentation was consulted:

- [GetClipboardData](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclipboarddata): returns NULL on failure; clipboard-controlled handles cannot be retained beyond the permitted interval; implicit format conversion exists.
- [SetClipboardData](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setclipboarddata): successful publication transfers object ownership to Windows.
- [GetClipboardSequenceNumber](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclipboardsequencenumber): contents/emptying and delayed rendering affect the sequence.

Those API facts support conservative lifetime/sequence handling, not an assertion
that a particular documented mechanism caused the observed failures.

## Implemented behavior and preserved authority

`src/platform/clipboard.rs` now separates publication from confirmation. After
publishing, at most four worker-side confirmation attempts are separated by50ms,
for at most150ms of scheduled waiting. Each attempt opens the clipboard anew and
requires the **unchanged initial post-close candidate sequence**, then the exact
owner HWND, bounded private marker and exact random marker equality before it
can request Unicode. Unicode handle/bounds/lock/digest must pass before success.
The UI thread does not sleep.

Only an advertised format with a temporarily unavailable handle/lock is retryable.
Missing formats, owner mismatch, marker mismatch, invalid bounds, changed candidate
sequence or digest mismatch cannot be converted into success. An unavailable
marker never authorizes a Unicode read. The clipboard is released between
attempts, admission is checked again, and a later writer—even with identical
text—is rejected. Sequence rebasing occurs only within an exclusive interval
after complete exact owner+nonce+digest proof, never across an intervening writer.

Confirmation-stage failure after publication and establishment of a nonzero
candidate sequence now carries its provisional descriptor back to the actor,
with an immediate cleanup deadline and **no Copied result**. It contains
the existing nonce/digest/sequence evidence, not plaintext. Failure before
publication retains any prior lease. Cleanup still requires exact sequence,
owner, marker and bounded digest within one exclusive open interval. Temporarily
unavailable reads retain the descriptor for the existing cleanup retry mechanism;
an actual later writer invalidates it. Existing bounded shutdown/OS-call and
endpoint limitations remain. No arbitrary clipboard value is read or cleared.

If Windows cannot supply a nonzero post-close sequence, publication still returns
Unavailable without a descriptor: there is no valid sequence authority to retain.
No such failure was observed here. This OS-access edge and bounded shutdown remain
limits; the correction does not guarantee cleanup without verifiable ownership.

Debug-only `CopyProofForEvidence` contains only attempt counts, booleans/options
and public sequence metadata. Its eight-slot bounded buffer preserves initial
failure and later successful proof snapshots. The API, snapshot storage and probe
writer are excluded from release. There are no handles, marker bytes, Unicode
values, OTP values, hashes, account identifiers or passphrases in snapshots.

`examples/interactive_clipboard.rs` consumes these snapshots outside clipboard
open intervals. Its existing250ms post-input quiet observer and disposable-only
vault remain. Production UI routes, crypto, vault data and styling are unchanged.

## Meaningful regressions and commands

The existing isolated-native parent exercises its helper on a separate create-only
noninteractive window station; no interactive user clipboard is touched by tests.
Test-only counters make the actual read boundary return NULL, while subsequent
reads call the real Windows API. Expanded helper scenarios cover:

- Unavailable Unicode followed by actual native digest proof.
- Unavailable advertised marker, then exact marker, then independently delayed
  Unicode, followed by actual complete proof.
- Exhausted marker/Unicode attempts return failure with a cleanup descriptor;
  unavailable marker never consumes the Unicode-read fault counter.
- An identical-text later writer between attempts is neither adopted nor cleared,
  for both marker and Unicode availability failures.
- A corrupt exact marker prevents Unicode access in confirmation **and cleanup**.
- Admission cancellation during delayed confirmation retains only cleanup evidence.

The new actor regression
`rejected_published_copy_is_not_success_and_retains_guarded_cleanup` verifies that
post-publication rejection reports Changed instead of success, keeps retryable
cleanup while busy, clears its own rejected value when available, and preserves an
identical later external copy. It uses the real actor with the existing Fake,
separately from the native proof tests.

| Current-source command | Result |
|---|---|
| `rtk cargo fmt --all` | passed |
| `rtk cargo clippy --offline --locked --all-targets --all-features -- -D warnings` | passed |
| `rtk cargo test --offline --locked --lib platform::clipboard -- --nocapture --test-threads=1` | **6passed,1direct helper ignored,204filtered,4.42s**; parent invokes that helper |
| `rtk cargo check --offline --locked --release --lib --example interactive_clipboard` | passed2.06s |
| `rtk cargo build --offline --locked --example interactive_clipboard` | passed4.57s |
| `rtk git diff --check` | passed |

The first new actor-test oracle incorrectly consumed `poll()` inside a wait helper
that calls its condition twice; it failed and was corrected to a bounded channel
receive plus explicit state assertions. Initial strict-Clippy diagnostics in new
test closures were corrected without allowing/ignoring them. No failed invocation
is relabeled a pass. No full-suite/hosted result is claimed in this report.

## Final actual native journey

The final freshly built probe used real pointer/keyboard input through Computer
Use, with one uniquely returned owned window and fresh observations between
actions. No arbitrary user clipboard, default/reference vault or OS setting was
accessed. The exact final files are `availability-fix/final-native-proof.txt` and
`final-observations.jsonl`:

1. Search → Sort → Folder → View → Manage folders → Copy through real Tab.
   Physical Space displayed **Copied**. Attempt0 had unavailable marker with
   sequence116; attempt1 passed the exact marker and all Unicode/digest checks
   at unchanged116. Observer public-fixture match=true at Unix1794576348.
2. Ctrl+L returned an empty locked form with no account/code nodes.
   Unicode was absent at1794576355, sequence117, before its30-second deadline.
3. Explicit unlock of the same encrypted disposable vault, then real pointer
   Copy displayed **Copied**. Attempt0 had unavailable Unicode after exact marker
   proof at125; attempt1 passed full proof at unchanged125. Observer matched the
   fixture code1794576404.
4. Normal Alt+F4 invoked real close/Lock/shutdown. `after_exit_cleanup`1794576411
   records unlocked=false, sequence126, Unicode absent. Fresh window inventory
   returned no probe; this is success-path exit without a preceding manual Lock.

The intermediate corrected run additionally observed a keyboard code expire at
exactly30s (6084→6114) and a rejected unavailable-marker attempt clear immediately
through its retained descriptor. The initial report's successful newer-copy
preservation is retained as prior evidence; final native newer-copy repetition
was not performed. Its same-value race protection is also exercised by the new
isolated-native/actor regressions. No additional screenshots were saved during
this corrective phase, so there is no new pixel-fidelity or full-page claim.

## Resources, integration and open limits

Retained only owned disposable roots under `target/`:

| Root | PID | Outcome |
|---|---|---|
| `task10-interactive-j2UmoQ` | 13692 | exact pre-fix Unicode-NULL proof; normal closed |
| `task10-interactive-gRgCNN` | 24196 | intermediate Unicode retry/marker failure proof; locked then normal closed |
| `task10-interactive-nL9OQk` | 29140 | final keyboard/Lock/pointer/exit pass; normal closed |

Ownership manifests contain executable/root/native HWND metadata; none is a
credential. Fresh process queries should precede future cleanup, never reuse a
PID blindly. This child did not recursively delete any roots or touch the parent's
other fixture. Final native exit removed the app-owned text; no subsequent user
copy was inspected.

Changed paths: `src/platform/clipboard.rs`, `examples/interactive_clipboard.rs`,
this report and six fixed-metadata evidence files under `availability-fix/`.
Earlier report/example/frame artifacts from this child remain preserved.

Parent must obtain distinct focused security/data-loss re-review of all guard,
retry, rejected-descriptor, actor-result and test paths before acceptance. The
current native scope closes the demonstrated transient-read/residual path only
after that review; it does not complete Task10 or the whole project. Full current
suite, native recovery-code Copy, actual OS lock/suspend/clock delivery, networking-
disabled machine, physical/mixed DPI, Narrator/high contrast, all-page QA and exact
reference fidelity remain their separate open gates. Parent should reconcile
architecture/security documentation with this confirmed behavior during integration.
