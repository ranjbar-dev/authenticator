# Task17 independent review

13 November 2026. Distinct read-only reviewer `review_17`; no implementation, status, staging, commit, push or calibration changes. **The initial P2 source finding is resolved by the focused re-review below. Task17 acceptance remains incomplete.**

## Finding

**P2 — Folder names can imitate generated identity captions.** `src/ui/folders.rs:14` and `:29` append ` · ID: <UUID>` only for exact duplicate/reserved names. A different, valid unique name may already contain that entire suffix. For example, two folders named `Public` cause the first caption to become `Folder: Public · ID: 11111111-1111-4111-8111-111111111111`. A third folder named `Public · ID: 11111111-1111-4111-8111-111111111111` receives exactly the same caption without a generated suffix. The name is within the existing raw256-byte validation limit and contains no forbidden control character.

The duplicate captions reach the manager heading (`folders.rs:189`), Codes selector (`shell.rs:2517`) and full selected-folder caption (`shell.rs:2560`). This prevents distinguishing these folders by the displayed caption or selector option name. **The selector remains keyed by stable `FolderFilter` IDs through `components::labeled_selection`; no wrong-ID mapping or data-loss defect is established.**

I independently inspected the two formatting branches and ran a static public-data construction of that exact formula: three distinct UUIDs produced two distinct captions, with captions0 and2 equal. This was a source/formula reproduction, not a native UI or actual Rust-function execution. Existing label tests cover reserved and exact-duplicate names but omit generated-suffix imitation.

Use an injective representation consistently in both `label` and `labels`: for example quote/escape the raw name and append generated identity outside the closing quote, or show the actual full UUID on every caption. Add an adversarial-name regression, including quote/backslash/suffix-like names, verifying unique captions, identical single/batch formatting and unchanged stable IDs/counts. Obtain focused independent re-review after the fix.

## Acceptance hold: portable-folder persistence

The required `ui::shell::folder_tests::folders_backup_repeat_import_retains_ids_trash_membership_states_and_conflict_policy` did not complete independently within75 seconds. The test remained at its running line; I requested normal termination of the exact spawned process, which exited1. This is **incomplete validation**, not a passing test or an assertion failure identifying the cause.

The author previously narrowed its pending work with removed temporary markers to portable decrypt/`crypto::derive` after scratch setup, using validated65536KiB/3iterations/1lane. I read the current unchanged `storage/crypto.rs`, `backup/envelope.rs`, preview/transaction boundary and diff: parameters and passphrase bounds precede the reviewed KDF call, folder transactions do not introduce a new crypto path, and these production modules have no final diff. I did not independently instrument the source or establish a root cause. Author observations of Wait/Suspended threads are prior evidence; they do not prove an OS-lock, algorithm, permission, filesystem or execution-host explanation. The author’s passing existing envelope control does not close this new case.

Do not ignore/remove this required test, weaken its identity/used-state/Trash oracles, claim the interrupted full suite passed, or accept fixture rearrangement as a fix. A complete normal run, potentially on the separate hosted Windows platform, is needed. Keep mandatory persistence and full-suite gates open until then. There is no recommendation here to disable protections or resume suspended processes.

## UI comparison and security scope

I read the eight task checklist roles first and compared the two changed Root views (folder manager/editor) and existing Codes toolbar region. The checklist was refreshed by the author against actual shared mechanisms; its original planning text is not evidence of measured conventions. `shell.rs` is the Root dispatcher/holder and `state.rs` a worker, not additional pages measured as deviations. Shared components/theme and the backup identity region were inspected where those roles required them. No additional source-level UI, security or data-loss finding was established in this scope.

Inspected branches include empty/no-results/invalid search, create/edit, six color choices and selected cue, duplicate consent, filtered/paged selections, hidden Trash membership guidance, dirty discard, folder Trash, saving/failure/retry, successful return and Lock. Source review confirms ID-backed many-to-many membership, same-pair reactivation, hidden trashed-account link retention, own-link/parent deletion filtering, equal-time content guards, retained-Trash limits and transactional latest Codes preferences. Account seeds/settings/recovery values and used history are not mutated by folder Save/Trash. No schema/KDF/dependency/import policy change is introduced. The new portable integration test remains unverified as above. Task18 still owns actual restore/permanent cascade and removal of its predecessor screens’ “upcoming Trash” guidance.

The task16 change is confined to deterministic1/60-second Context frame time and the requested nth-button diagnostic. It preserves coordinate press/release, wheel, normal600x800 viewport, set occurrence19, later-code/Undo/Trash and in-viewport focus assertions. Its independent pass is framework evidence; it is not native pointer proof or a new hosted-CI pass.

Accessibility observations are separate:

- **Contrast:** Root mounts the existing dark theme. Text/surface, primary action, error and focus pairs resolve through shared components; existing project4.5:1 text and3:1 essential-graphic tests pass. Ratios printed independently include body/background16.73, card14.32, field15.53, hover11.41, secondary/surface8.12, secondary/hover6.47, primary5.85, error8.69, success10.46, accent5.01; focus/background9.49, surface8.12, field8.80, hover6.47. Actual rasterization/disabled compositing is unverified.
- **New color accents:** computed opaque swatch/SURFACE ratios are Blue5.01, Purple7.26, Pink8.76, Red7.72, Orange9.67, Green9.88. Against selected BLUE they are1.00/1.45/1.75/1.54/1.93/1.97. The stripe is redundant decoration: each choice has a color name, written Selected cue and radio checked state; selection meaning does not rely on stripe contrast. No text is drawn over the swatch.
- **Focus:** shared focus borders and explicit programmatic scrolling are retained. Framework tests verify safe initial Keep focus and visible manager/Manage return focus, failure retention and Lock clearing. Not every Keep/ESC/backdrop and post-error focus permutation was independently exercised; native painted focus is unverified.
- **Keyboard:** Context Tab/Space/Escape and coordinate press/release/wheel exercise real egui controls. No direct element-click shortcut is used in that test. Physical Windows keyboard/pointer and native drag capture remain unverified.
- **Labels:** shared fields associate labels, helper/error descriptions and invalid state. Membership checkboxes have service/identifier/full UUID names and checked state; color controls have radio names/state. The caption collision above remains a separate finding. No recovery values are exposed by the tested folder AX output.
- **Text alternatives:** visible words accompany color/selection and shared icon-only header controls have names. No new meaningful unlabeled illustration was found.
- **Targets/structure:** shared minimum36-point controls, ordinary340x780/600x800/960x900 framework horizontal bounds and semantic Root/card headings are used. Actual pixel wrapping, overlap, physical size, screen-reader announcements and complete heading navigation remain unverified. No accessibility conformance claim is made; other criteria were not checked.

The applicable shared values are Noto Sans400/600; TITLE24/32, SECTION18/24, BODY/LABEL14/20 and CAPTION13/20, as-written case/tracking0; base4 spacing, padding16, control36 and Root max640. No root DESIGN.md or original reference screenshots were found. Reference fidelity uses `docs/reference-ui.md` inventory only. I did not retry the two unchanged author Sky denials; no native launch, screenshot or computer automation was performed by this reviewer. This is not a global diagnosis of UI availability.

Prior calibration: `docs/qa/task04/calibration.md` records a detached copy at `C:/Users/root/AppData/Local/Temp/authenticator-ui-cal-4cg1yjfn/checkout`, now removed, with TITLE600 changed to400 and detected by the distinct calibration agent. It proves source-level font-bundle sensitivity only. No plant established native pixels, contrast, overflow, focus or interaction sensitivity. Task07’s native overlay calibration is a different kind. Whole-Root cross-page visual comparison remains later task21 work.

## Independent commands and cleanup

All commands used RTK. Test commands below executed the current isolated binary `target/task17-isolated/debug/deps/authenticator-2eb1a8e5551a193d.exe` through an `rtk proxy python -X utf8 -c` subprocess wrapper that drained output, recorded each exact PID/start/command and set TEMP/TMP to a known reviewer-owned directory under ignored `target/`. Passing checks had a60-second per-invocation ceiling; the stalled required case had75 seconds. No product source instrumentation or ignore was added.

| Command after the executable | Independent result/oracle |
| --- | --- |
| `domain::folder_tests --nocapture --test-threads=1` | 5 passed,170 filtered,0.51s. Name byte/control/trim/consent/color policy; same-link/hidden-member preservation; own-link/parent Trash and backend-only restoration; equal-time stale guards; retained10k capacity. |
| `ui::shell::folder_tests --skip folders_backup_repeat_import_retains_ids_trash_membership_states_and_conflict_policy --nocapture --test-threads=1` | 3 passed,172 filtered,34.73s. Explicit focused exclusion is not a full-suite pass. Ordinary widths, coordinate controls/Tab/Space/Escape, paging/selection/discard; exact write-failure/retry; latest/older-queued preferences and encrypted reopen; held admitted-save Lock before release/no late adoption. |
| `ui::folders::tests --nocapture` | 1 passed,174 filtered,0.00s. Existing reserved/exact-duplicate labels; does not cover the new caption finding. |
| `ui::shell::recovery_detail_tests::detail_later_code_and_scrolled_list_focus_are_visible_after_save_undo_and_trash --exact --nocapture --test-threads=1` | 1 passed,174 filtered,17.78s. Original CI16 regression with retained visibility/input oracles. |
| `actual_text_and_essential_graphic_pairs --nocapture` | 1 passed,174 filtered,0.00s; actual ratios above. |
| `ui::shell::folder_tests::folders_backup_repeat_import_retains_ids_trash_membership_states_and_conflict_policy --exact --nocapture --test-threads=1` | INCOMPLETE at75s; normal termination exited1, no passing/assertion result. |
| `rtk proxy cargo test --locked --all-features --target-dir target/task17-isolated --lib --no-run` | Success,2.04s, current Cargo test output up to date; no source rebuild reported. Confirms the inspected source/binary build association. |
| `rtk cargo fmt --all -- --check`; `rtk git diff --check` | Passed. |

No broad full-suite repetition was attempted independently. Author Clippy/build/full-run outcomes remain prior evidence, not this reviewer’s execution. No real/default/reference vault, credential, backup or interactive clipboard was accessed; UI fixtures inject Fake. No native app, unrelated preview, author leftover process, Windows security setting or hardware/network setting was changed.

My stalled PID33280 and completed check PIDs18712/40424/43444/40232/31720 were absent in a fresh CIM inventory before cleanup. Verified absolute reviewer roots `D:/Projects/github/authenticator/target/task17-review-runtime-62d7f306-0d11-4e1c-bcc2-d3e34eb8401b` and `.../task17-review-checks-7ebee3dd-6038-4639-8661-ae41bf173e1e` stayed below the intended workspace `target` and were removed with native PowerShell `Remove-Item -LiteralPath`. This cleaned the two known generated TempDirs left by my timeout without scanning other Temp data. Author PID30688/earlier unknown TempDirs were not touched; their separate cleanup limits remain in author evidence.

A proposed workflow job `timeout-minutes:30` is a reasonable bounded-execution follow-up to the measured14m02s prior hosted run. It must report timeout as failure/incomplete and keep every required test. It is a budget limit, not a test fix or acceptance evidence; adjust only from later measured successful duration if needed. No workflow edit was made here.

Recommendation: fix/re-review the P2 caption ambiguity before source integration. A coherent explicitly-incomplete checkpoint may then seek hosted execution evidence, while required portable/full-suite/native/manual gates stay open. Do not mark Task17 done. Review cost estimate:2 changed views plus1 toolbar region, roughly25 project files opened and14 searches; skill documents are additional reading. No pages were judged from native raster output.


## Focused re-review: caption correction

13 November 2026, same independent reviewer, after the parent fix. **P2 resolved; no new source finding established in this focused scope.** This supersedes the initial source-correction recommendation above. A coherent explicitly incomplete source checkpoint may proceed; Task17 is not accepted as done.

Both `label` and `labels` now use Rust string Debug formatting for the raw folder name, with any generated full UUID outside the closing quote. Quotes/backslashes within names are escaped. A unique-name caption ends at the closing quote, while an ambiguous-name caption ends at the actual UUID; raw name text cannot imitate the suffix boundary. Distinct ambiguous identities retain distinct UUID endings. Stored raw names, IDs, counts and stable FolderFilter selection are unchanged.

The permanent `folder_navigation_name_cannot_imitate_generated_identity` regression constructs the reported suffix-forgery case using the first folder actual UUID, plus quote/backslash/suffix-like names and a reserved name. Its oracle checks distinct captions, exact ID-set/count preservation, agreement between single/batch formatters, preserved raw forged name and a valid aggregate. The parent reports a genuine failure on the original formatter (7 IDs,6 captions); I read that evidence as prior execution, then independently ran the corrected actual test. No no-test-selected invocation was counted.

Independent commands after reading the fix/test:

- `rtk proxy cargo test --locked --all-features --target-dir target/task17-isolated --lib --no-run`: current source compiled successfully,4.84s, producing the same isolated test executable. It now contains176 tests.
- `ui::folders::tests --nocapture --test-threads=1` through that executable:2 passed,174 filtered,0.00s. Both the original reserved/duplicate regression and the new adversarial-name regression executed.
- `ui::shell::folder_tests::folders_pointer_keyboard_widths_multiselect_discard_and_focus --exact --nocapture --test-threads=1`:1 passed,175 filtered,6.92s. Retained ordinary340x780/600x800/960x900 Context geometry, coordinate press/release/wheel, keyboard checkbox, selection/save/discard and visible return-focus assertions. This remains synthetic framework evidence.
- `rtk git diff --check`: passed.

These direct tests used the same bounded Python subprocess method as the initial review, with known TEMP/TMP `D:/Projects/github/authenticator/target/task17-rereview-aa708b81-abdc-4f8c-8d32-dedd5c5013b5`, PID/command/start tracking and a60-second ceiling. Both exited0. Fresh CIM showed PIDs40668/22452 absent; the resolved exact owned runtime was verified and removed with native PowerShell `Remove-Item -LiteralPath`. No other process or directory was touched.

I also inspected the workflow diff: only job `timeout-minutes:30` was added. All existing formatting/lint/test/build steps remain. This is the bounded-execution follow-up assessed above and provides no test-pass or stall-fix claim.

No backup-engine, worker, preference, cancellation or native change was part of this fix. The unchanged required backup stall was not retried, and no native launch was attempted. Required portable-folder roundtrip, complete suite, hosted evidence and actual native/manual gates remain open as previously recorded. Additional focused cost:1 source file/test region, workflow and parent evidence section; approximately3 additional project files opened and2 searches. Initial comparison/raster/calibration limitations remain unchanged.
