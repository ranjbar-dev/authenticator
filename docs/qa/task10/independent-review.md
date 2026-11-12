# Task10 independent review

Parent record, 13 November 2026. Distinct read-only `review_10` was neither the
implementer nor a calibration agent. The original review required corrections;
focused independent re-review resolved both. Source is accepted within the reviewed
scope. **Task10 remains in progress: interactive Copy acceptance is open.**

## Findings and correction

- **P2, shared route query:** Codes and Backup Codes bound the same `AppState.query`,
  but only Codes changes updated its encrypted preference projection and ID cache.
  Codes `public` -> Backup Codes `absent` -> Codes could show `absent` while filtering
  `public`. Parent separated `backup_query` and its widget identity. Only Codes
  projects Codes preferences; both session buffers zeroize on Lock.
- **P3, corrected-query feedback:** invalid search assigned a global status that
  survived valid correction until another status/save replaced it. Parent removed
  that assignment; inline validation derives from current input and does not replace
  unrelated operation notices.

The new `route_searches_stay_separate_and_corrected_input_clears_only_query_feedback`
regression drives actual egui TextEdit paste/select-all events, checks query/result
IDs/visible labels across routes, oversized rejection, corrected validation,
unrelated feedback, both-buffer lock cleanup and encrypted owner reopen. These
deterministic events are not represented as native pointer evidence.

Focused reviewer re-review independently passed:

| Command suffix after `rtk cargo test --locked` | Result |
|---|---|
| `route_searches_stay_separate --lib` | 1 passed, 8.89s |
| `lock_drops_repository_access_query_and_pending_intent --lib` | 1 passed, 7.79s |
| `rtk git diff --check` (separate command) | Passed |

No new finding arose. Parent final checks after correction: full
`rtk cargo test --locked --all-targets` **104 passed,3 subprocess helper entrypoints
ignored directly,14 suites,62.84s**; formatting, strict all-target/all-feature Clippy,
diff and build of Authenticator/codes_list/account_editor/import_flow/qr_batch_fixture
passed. Helpers are exercised by their corresponding subprocess parent tests.

## Independent functional/security checks

The original reviewer independently passed:

| Filter after `rtk cargo test --locked` | Result |
|---|---|
| `isolated_native_clipboard_process --lib` | 1 passed, real actor/private station |
| `platform::clipboard --lib` | 5 passed; helper exercised by the parent above |
| `schema1_frozen --lib` | 1 passed |
| `all_six_sort_modes --lib` | 1 passed, encrypted owner restarts |
| `trash_and_preferences --lib` | 1 passed |
| `ui::codes --lib` | 2 passed |
| `capture_privacy_tests --lib -- --test-threads=1` | 7 passed,73.41s |
| `ui::components --lib` | 7 passed |

A fresh temporary Rust probe linked the actual `codes::copy_value`. Independent
Python standard-library HMAC matched **120/120** values across all24 settings and
five boundary/forward/backward times. Aggregate results only were emitted; the
probe was removed. An initial link lacked the cached Windows native-library search
path; adding it resolved that tooling failure.

Review covered bounded clipboard buffers/queues, native admission/counter/lifecycle
guards, sequence/owner/marker/bounded digest cleanup, atomic repository ownership,
stale-result rejection, lazy schema migration, preference coalescing and retained
Trash associations. No additional demonstrated source security/data-loss blocker
was found. The reviewer consulted Microsoft [clipboard ownership](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-operations)
and [sequence semantics](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclipboardsequencenumber).
This focused review is not cryptographic certification or the final Task20 audit.

## UI scope and limitations

The reviewer compared all eight actual sanitized native images in `images/`, covering
340/600/960 Codes/no-results/list/grid plus editor/delete/locked regions against
the complete task checklist, theme and foundation. No additional visual deviation
was established in those regions. Redacted code pixels remain unevaluated. Parent
separately verified all PNG chunk CRCs, dimensions, IEND and absence of trailing
bytes, and viewed the intermediate/narrow/desktop and modal/locked frames.

Two original documented normal `sky.launch_app(codes_list.exe)` attempts returned
`GetCursorPos failed: Access is denied (0x80070005)`, with fresh inventory between.
One bounded fresh attempt during re-review returned the same denial. No Codes
preview appeared. No retry loop, permission bypass, unrelated preview interaction
or inference about the cause occurred.

The author separately reached live Copy and received `Changed` during post-close
ownership confirmation; the exact failed HWND/marker/content subcondition/cause
is unestablished. Healthy listener/nonzero owner/queue booleans are recorded in
`copy-diagnostic.json`. **Final interactive pointer/keyboard Copy success and its
expiry/Lock/exit clearing have not passed.** Isolated native tests do not replace
that gate, and the reviewer launch denial does not explain the author rejection.
Checks must not be weakened to force a pass.

Hardware/mixed physical DPI, actual OS lock/hardware suspend delivery, disconnected-
machine operation, Narrator, exact reference fidelity and independently repeated
process relaunch remain unverified. No native input repeat of the corrected route
search is claimed. Task20/21 must revisit these gates; no requirement is waived.

Task04/07 calibrations detected source typography plants, with documented live/
pixel limitations. No new calibration or general pixel-sensitivity proof is claimed.
Reviewer cost estimates: three Root route families, eight images, about24 source/doc
files and five grouped searches/diffs; focused re-review two files, one search and
two tests, no new rendered comparison.

Both reviewer finals recorded no Codes preview/process or relaunch locator, no
source/docs/status edits, no clipboard inspection and no default/reference access.
The unrelated lifecycle preview was untouched. Commit reference follows in the
parent checkpoint because embedding a commit hash in itself would be circular.
