# Task16 implementation evidence

13 November 2026. Implementation handoff for independent UI/security/data-loss review. Task status and acceptance belong to the parent; this document does not close native or Task18 gates.

## Scope and implementation

- `src/domain/recovery_code.rs`: a bounded per-code full-content guard (selected account, parent set identity/state, selected code ID/value/state/used timestamp). Equal timestamps alone never authorize an update. Mark used, Mark available, exact previous-used-time Undo and per-code Trash touch the code, set, account and aggregate at the vault's clamped logical mutation time, inside the existing sole-store transaction.
- `src/ui/state.rs`: typed `RecoveryCode` intent uses the existing cancellation, admission, commit and publication worker. No second mutable store, DEK clone, schema change or synchronous Root write was added.
- `src/ui/recovery_detail.rs`, `src/ui/backup_codes.rs`, `src/ui/shell.rs`, `src/ui/mod.rs`: selected-account detail, twenty active-code rows per page, separate Reveal/Conceal and Copy, explicit used-state controls, one guarded Undo, confirmed per-code Trash, existing Task15 Edit/confirmed set Trash routes. A revealed arbitrary 4096-byte value wraps inside the shared bounded BODY-height holder. The session retains reveal IDs rather than a plaintext set copy. An Undo/confirmation guard contains at most one selected code snapshot.
- `src/platform/clipboard.rs`: production clipboard actor unchanged; a `cfg(all(test, windows))` fixture exposes the existing Fake to UI tests. Static recovery Copy uses the existing actor without a TOTP code window, never changes the used state and never produces an Undo. Raw values do not enter feedback or logging.
- `examples/recovery_detail.rs`: argument-free owned preview, public generated records in a fresh `TempDir`, F6/F7/F8 widths and F10 actual framebuffer sanitization. All current OTP/editor/detail reveal bounds are redacted before PNG bytes reach disk; leaving the detail via its footer retains that frame's already-painted bounds until collection.

Successful saved state, editing, route changes, pagination, Lock and adopted replacement import conceal values. Failed writes retain prior data and a still-valid prior Undo. Failed backup import preserves the unrelated detail session; the backup worker's outer success does not prove that its nested Apply succeeded. Undo is captured from the actual adopted record after success and restores its original `used_at`, including a pre-existing timestamp. Leaving/locking clears Undo. Programmatic focus explicitly scrolls its live control into view: header Back on open, the changed code's used-state control after Mark/Undo, first remaining Reveal after per-code Trash, header for an empty set. Cross-page Undo selects the affected active code's page before restoring focus.

Code and set deletion here mean reversible encrypted Trash. Associated values and used history remain; counts and total limits continue to include retained Trash as appropriate. Account Trash hides its associated set. Restore/permanent-delete UI belongs to Task18. Generated backend tests simulate restoration and reject orphan references; they do not claim Task18 UI exists.

## Refreshed screen checklist and decisions

Sized as the existing detail page's full phase. The process supplied authorization and the eight requested roles before this child started; no further approval or product choice was needed. Ending3: the existing parent execution/review loop owns integration. The checklist lives in that running task, refreshed here against actual source rather than the obsolete no-source planning snapshot.

The detail family has one same-kind reference, the current `backup_codes.rs` Detail branch. Its List/Select regions, Task15 editor and Task10 2FA cards provide mechanisms, not a fabricated majority of detail pages. Shared pieces win over private copies. The grouped source search used `c::card|c::button|c::heading|c::status|c::icon_tile|c::Dialog` over `backup_codes.rs`, `recovery_editor.rs` and `codes.rs`; the actual 2FA card orchestration was followed into `shell.rs`.

1. Account header: existing `identity`/ambiguity logic and shared `card`, `icon_tile`, level2 `heading`; full UUID remains for visually identical service/identifier pairs. Header Back and existing editor actions remain labeled and keyboard reachable. Requested structure; shared-piece mechanism.
2. Guidance: concise service-issued, potentially single-use explanation and explicit Copy-does-not-mark-used wording, using shared BODY/CAPTION labels. New copy is recorded in source for review. Requested meaning; shared typography.
3. Rows: shared card with BODY available/used label, static concealed placeholder and no changing secret announcement. Twenty rows per page. Requested roles; existing pagination mechanism.
4. Reveal and Copy: independent shared buttons in explicit two-column holders, separate code-number AX names. Full value only enters plain label/TextRun content after deliberate Reveal. Requested actions; shared button mechanism.
5. Used/Undo: current state determines Mark used/Mark available; a labeled one-step Undo is disabled until a successful used-state adoption. Guarded Undo and session scope are explained beside it. Requested behavior; shared buttons.
6. Deletion: shared Dialog defaults to Keep code; Escape/cancel returns to a live opener. Existing Task15 confirmation moves the set to Trash with account preserved. Permanent deletion remains Task18 and must be explicitly irreversible there. Requested meaning; existing dialog/editor mechanism.
7. Empty/error/copy feedback: static labels and existing Root status/clipboard receipt region. No raw code in a label for a control, toast, status, error or live region. Requested privacy; shared feedback mechanism.
8. Theme/focus/narrow/Lock: `theme.rs` selected by Root; BODY14/20 regular400, LABEL14/20 semibold600, CAPTION13/20 regular400, existing level1 TITLE24/32 and level2 SECTION18/24 semibold600. Theme text/surface and primary foreground/background pairings, 4-point unit, SMALL_GAP8, shared card padding16/control height36; no new private palette/fonts/opacity. Value holder uses existing `MULTILINE_HEIGHT`. Explicit scroll-on-focus handles collapsed busy layouts. Requested constraints; shared tokens.

Not copied: unavailable reference pixels are not treated as measured fidelity; no profile, sign-in, cloud, language, extension, encryption toggle or swipe-only interaction. No additional design system or app-wide styling was introduced.

Skills read: ui-consistency finding-patterns, values, decisions, planning and implementing, plus their called process/checklist/word/plan references; Computer Use and its guidance/API/confirmation documents. Source reading/search counts were not maintained exactly (approximately 20 project files and 30 grouped/narrow searches). No project convention counts are invented. Prior same-kind calibration is `docs/qa/task04/calibration.md`: a separate checker detected TITLE600→400 in a detached copy, but live launch/pixel sensitivity was unproven. Parent supplies the distinct checker for this task.

## Native attempt and boundaries

Built the production bin/lib and owned example with `rtk cargo build --all-features --bin Authenticator --lib --example recovery_detail` (success). Computer Use imported documented `@oai/sky` through `node_repl`, then used fresh returned inventories and exactly two normal `sky.launch_app` calls with the explicit owned example path. Both returned:

`GetCursorPos failed: Access is denied. (0x80070005)`

Before, between and after the attempts, the owned-window inventory was empty. Read-only process/fixture checks found no `recovery_detail` process and no `authenticator-task16-*` directory. No window meant no input or screenshots. Cause remains unknown; it is not evidence that Windows was locked. No elevation, permissions, security/privacy/global settings, network adapter change, substitute high-integrity launch, private protocol or unrelated preview was used. No cleanup was required for a preview that never started.

All test vaults/backups are generated public fixtures in owned temporary directories. UI Copy tests replace the actor backend with the existing Fake before any Copy action. They never read/write the interactive user clipboard. Existing isolated native helpers run only through their parent tests in a separately created noninteractive station; WinSta0 is never selected or read. No real/default/reference vault, backup or code was opened.

## Checks

`rtk cargo test --all-features --lib recovery_detail_tests -- --test-threads=1`: **7 passed**, 158 filtered out, 186.02s. The final focused source included explicit scroll-on-focus and the following checks:

- Real Context press/release pointer coordinates and wheel scrolling, plus Tab/Enter/Escape keyboard activation; no synthetic element activation. Normal 340×780, 600×800 and 960×900 layouts, code-specific AX labels, usable two-column action bounds, deliberate Reveal/Conceal full-tree checks, bounded long-value holder, exact 4096-byte Copy, pagination and navigation re-conceal, idle/explicit Lock.
- Existing List → View set → detail IDs, keyboard used/Undo/Edit/delete requests, pointer copy and confirmed per-code/set Trash. A code20 offscreen save and Undo restore focus to the same live code control with actual in-viewport bounds. Trash moves focus to remaining Reveal; opening detail from set-list row20 lands visibly on header Back.
- Copy actor acknowledgement and Changed failure feedback, exact requested static value, no used/Undo/ciphertext change from Copy alone, independent actor expiry without Root frames, Lock cleanup and preservation of a newer external identical fixture copy. All use Fake and the production actor queue, not Windows' interactive clipboard.
- Atomic save failure/retry, prior Undo surviving a failed subsequent mutation, actual adopted used timestamp and exact old-history Undo, encrypted restart and portable backup IDs/used-state preservation. Per-code Trash retains value/history/reference, set Trash leaves account active; the generated restored-parent backend fixture preserves association/history.
- Admitted-save commit gate held while Lock removes reveal and Undo before I/O release, no late unlocked adoption/status, honest encrypted completion subsequently visible only after explicit unlock.
- Failed replacement import preserves reveal/Undo/current route data; approved successful replacement reusing IDs returns with values concealed and Undo cleared. Parent Trash hides all associated codes and a backend-only restore fixture returns concealed.

`rtk cargo test --all-features --lib recovery_ -- --test-threads=1` before the new tests: 20 existing recovery-focused tests passed (136.67s). The final full-suite run below supersedes this early regression snapshot.

Early focused runs exposed and corrected shared-button width misuse, legacy footer focus, and test scrolling beneath the Root clipboard feedback panel. Earlier failed iterations are not acceptance evidence. An intermediate full `rtk cargo test --all-targets --all-features` passed 177 tests with 4 ignored parent-exercised helper entrypoints in 19 suites (132.72s). A subsequent cross-page Undo regression correctly failed against that source (`focused action`, 15.13s): code20 was absent from page2 after Undo. Adoption now selects its page. `rtk cargo test --all-features --lib detail_later_code -- --test-threads=1` then passed the updated regression (1 passed, 17.76s). The final all-target/static results are appended below.

The isolated example's long fixture was corrected from an over-limit literal to 4092 bytes before final delivery; the failed native launches never reached fixture creation. The same `public_fixture` function now validates its graph before use and has a pure, non-native example test, so a compiled example cannot silently hide an invalid fixture.

Final commands and results:

| Command | Observed result |
| --- | --- |
| `rtk cargo test --all-targets --all-features` | **178 aggregate passed**, 4 intentionally ignored standalone helper entrypoints, 164 filtered out from parent-launched helper runs, 19 suites, 129.87s. Includes the existing 4 parent-exercised private helper cases and the new pure example-fixture test. |
| `rtk cargo clippy --all-targets --all-features -- -D warnings` | Success, no issues. |
| `rtk cargo fmt --all -- --check` | Success. |
| `rtk git diff --check` | Success. |
| `rtk cargo build --all-features --bin Authenticator --lib --example recovery_detail` | Success, final build 4.91s. No standalone all-target build touched unrelated preview executables. |

After the final full-suite run, the stale Backup Codes help sentence saying these controls were unavailable was replaced with an accurate static description of the new actions. No behavior changed after that test run; the final strict Clippy, format, diff and bin/lib/owned-example build include that copy correction.

Changed paths: `src/domain/mod.rs`, `src/domain/recovery_code.rs`, `src/platform/clipboard.rs`, `src/ui/backup_codes.rs`, `src/ui/mod.rs`, `src/ui/recovery_detail.rs`, `src/ui/recovery_detail_tests.rs`, `src/ui/shell.rs`, `src/ui/state.rs`, `examples/recovery_detail.rs`, and this evidence file. No PRD, task/status/index/coverage, staging, commit or push changes were made by this child.

Ready for the parent's distinct read-only UI/security/data-loss review. No self-approval or native acceptance is asserted. Owned test directories use `TempDir` lifetime cleanup; native preview creation never began, and no unrelated process or file was terminated, deleted or moved.

## Open gates

Actual Windows pointer/keyboard, rasterized contrast/wrapping/focus, DPI/assistive-technology and native reboot/restore paths remain unverified. Context input/AX/shape tests cannot close them. The existing real interactive clipboard `Copy → Changed` ownership-proof issue from Task10 remains open; production checks were not weakened and no live success/expiry claim is made. Task18 still owns restoration/permanent deletion UI and logical cascade implementation. No next-task work was added.


## Hosted CI follow-up, 13 November 2026

Commit `7e1bf02890115bcf1c59f743ed023a1b85486b9e` run [43362221337](https://github.com/ranjbar-dev/authenticator/actions/runs/43362221337) failed: Format and Lint passed; Tests reported 160 passed, 1 failed, 4 ignored in629.77s; Build was skipped. Sole failure: `ui::shell::recovery_detail_tests::detail_later_code_and_scrolled_list_focus_are_visible_after_save_undo_and_trash`, `action unavailable: View set; button bounds Some([[33.0 -352.0] - [567.0 -316.0]])`. That error currently prints the first matching button, not necessarily the selected occurrence; do not infer the actual target geometry from it. Logs were retrieved from the run archive without exposing/persisting authentication. The sequential task17 source owner has the exact failure for diagnosis and correction; fix/re-review/new hosted evidence pending. Earlier local/reviewer passes remain limited evidence and do not override this failure.


Hosted follow-up: CI43370232096 SUCCESS atdba745a7295a016fb35556ad28338a466707d12c confirms original deepListregression ...ok05:20:23UTC and all172library/17exampletests. Source-only deterministic Contexttiming correction resolves observed CI16testfailure; no nativeinput/interactiveclipboard/18acceptance claim. See task17 hosted evidence.
