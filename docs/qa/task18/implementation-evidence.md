# Task18 implementation evidence

13 November 2026. Author handoff for separate review; this is not acceptance. Parent owns task/index/coverage/PRD and integration. No real/default/reference vault, backup or interactive clipboard was used. Source and focused checks are ready for review. Full-suite and native gates remain open as recorded below.

Changed source: new `src/domain/trash.rs`, `trash_tests.rs`, new `src/ui/trash.rs`, `trash_tests.rs`, their `mod.rs` exports, safe errors in `src/error.rs`, sole-worker integration in `src/ui/state.rs`, Settings/lifecycle/Close integration in `src/ui/shell.rs`, actual Trash guidance in `src/ui/{folders,backup_codes,recovery_editor,recovery_detail}.rs`, and new owned `examples/trash.rs`. Parent documentation/status edits were preserved. No child staging/commit/push or completion-status edit.

## Shape and current decisions

Sized as a whole UI phase, ending3: existing task18 orchestration owns the plan and distinct review. Task18s eight roles were reread before UI code. Actual Route has only Codes, BackupCodes, Settings; Trash is a visible Settings subpage, not a fourth tab. Nearest mechanisms are folder manager and backup list (two list regions of three candidates; recovery detail is not a list peer). No majority for Trash-specific roles is asserted. Shared components take precedence; Root dispatcher/state are not peer pages. No root DESIGN.md/original reference pixels; docs/reference-ui.md inventory is the fallback.

Refreshed checklist: Root shared h1/scroll holder; Back/own-Trash count/Empty action near top; retain-until-permanent and own-versus-inherited policy;20-row metadata cards using account identity/full IDs and escaped folder names plus UTC deletion dates; separate visible Restore/Permanently delete; shared safe-Keep Dialog with preserved association counts, actual purge cascade counts, conflict consent and parent-review actions; shared error/progress/retry/current-review route; theme/focus/keyboard/ordinary340x780,600x800,960x900. Shape is requested, mechanisms use components.rs/theme.rs and existing list/Dialog patterns. Noto400/600 BODY/LABEL14/20,CAPTION13/20,TITLE24/32,SECTION18/24,base4,padding16/control36/Root640; no new palette/font/shared component. No swipe-only/profile/cloud/language controls. Native raster/input/Narrator/DPI remain unverified.

All five own-Trash kinds are included: account,folder,membership,set,code. Hidden active children are not separate deletions. Restoring a parent preserves independently deleted child states; leaf restores require active parents and offer explicit parent review. Matching account credentials/identity or exact folder names require separate-record consent; nothing overwrites/coalesces existing records. Restore association counts mean preserved records, including ones still in Trash, not a promise that all become visible.

Permanent account deletion removes its links/sets/codes; folder deletion removes only its links; set deletion removes its codes; leaf deletion removes only the leaf. Overlapping Empty scope deduplicates records. Removing the last stored code explicitly previews removal of its now-empty owning set, even when that set own-state is active; account/unrelated sets stay. Counts disclose active-own-state child records even if inherited hidden. No automatic expiry or secure filesystem/SSD/memory/old-backup erasure claim. Exported backups are unchanged.

Domain Review streams a type/count/length/optional-delimited SHA256 binding over every domain field including seeds/settings/values/used dates/own states/IDs/metadata/preferences/logical timestamps. This is an in-memory stale-review guard, not storage/authentication/diagnostic output; no passphrase/DEK/full-Vault clone. Digest is zeroizing and Debug redacted. Equal-time changes invalidate it. Latest Codes preferences are carried atomically with approved Trash mutation and reloaded only after adopted success; failure preserves prior data/dirty intent. An intervening preference commit can safely require review again.

Maximum valid graph measurement (10000codes x4096bytes) found611ms capture and631ms checked apply in debug. Consequently preparation moved off Root onto the existing sole-repository worker with typed result, idle/lifecycle/generation and nonblocking cancellation/publication checks. Root retains only IDs/counts/review digest and shows progress/Lock. Mutation rechecks guard on worker. Active hash/key buffers may remain until work returns; no forcible erase/cancel claim.

Explicit Close now discards unsubmitted account/recovery/folder/Trash/export/import presentations and delete/detail dialogs before the existing eligible latest-display-preference flush. This addresses guards that otherwise strand advance_close. Busy writes/native capture/pickers retain the existing cancellation/Lock path; no business mutation is submitted by Close. Successful replacement invalidates derived Trash state even with reused IDs; failed import keeps the prior valid review. Nested import failure is not treated as success. Codes/Backup/folder-form queries remain distinct.

Skills read/applied: ui-consistency finding-patterns/values/decisions/conventions/planning/implementing and called joining/words/checklist/design-reading/family-by-region/position/contrast/earlier-plan/plan-file references. Plan already authorized by user; no questions or additional agent. Existing task04 source calibration is prior source sensitivity only, no native/pixel/input proof. Parent will dispatch verifying separately.

## Earlier focused checks

All commands prefixed RTK; tests use known TEMP/TMP runtime roots under ignored target via target/task18-run.py, logging exact owned PID/executable/arguments/start/root only. UI constructor substitutes existing Fake clipboard before actions. Test data is generated public data. No user clipboard values are read/written.

Fresh isolated no-run build: target/task18-isolated,50.49s initial dependencies. Root main library test output remains locked by prior author PID30688; it is untouched, as is unrelated vault_lifecycle.exe.

- domain::trash_tests:7 passed0.64s before performance case; seed/settings/used/independent states, parent paths, duplicate consent, cascades/overlaps/final-code set, equal-time binding, dense10k links.
- domain::trash_tests::trash_maximum_code_graph_review_guard_and_leaf_scope_are_bounded:1 passed3.34s; actual valid40MiB graph,611ms capture/631ms checked apply, exact leaf scope and valid remaining graph. Observational timings, no flaky threshold.
- ui::trash::tests:1 passed0s; UTC epoch/leap2000/nonleap2100/year9999 and exact u64MAX fallback. Gregorian calendar reference consulted: https://howardhinnant.github.io/date_algorithms.html ; implementation uses bounded simple year/month loops, no new dependency or copied date-library implementation.
- Before worker preparation refactor: normal three-width/pagination/Keep-Escape-backdrop/keyboard/metadata test1 passed17.69s; parent-restore/duplicate-consent1 passed6.93s; fault/retry/latestprefs/encryptedrestart1 passed10.76s; Empty/final-code scope/admitted Lock1 passed9.08s. Affected tests will rerun after async changes.
- After worker preparation: read-only held preparation Lock/Close1 passed13.00s, no write/no late review; equal-time stale review/current-review retry1 passed8.30s.
- Development `cargo check --locked --all-targets --all-features` passed. Temporary unused test imports were subsequently removed; final strict lint passed below.

Fault-test oracle correction: initial helper checked AX bounds against whole viewport, then clicked final confirm at y595..631 below actual modal clip ending593; backdrop closed review and no worker queued. Temporary coordinate/clip markers proved bounds did not move between press/release, so focus/production Dialog was not implicated. Helper now checks actual painted rectangle inside emitted ClippedShape clip and wheels inside modal before coordinate press/release; explicit assertion proves worker queued. Real error/retry test then passed. No shared Dialog/focus production fix was made, and markers removed. These are synthetic Context/paint geometry, not native pixels/input.

## Final passing checks

Focused commands were `rtk proxy python -X utf8 target/task18-run.py <filter> [--exact] --nocapture --test-threads=1`, using exact `target/task18-isolated/debug/deps/authenticator-2eb1a8e5551a193d.exe`. The owned wrapper sets fresh `target/task18-runtime-<UUID>` TEMP/TMP, logs PID/path/arguments/start/root, and uses90s timeout. The final harness writes stdout directly to an owned file with `Popen.wait(timeout)`, avoiding descendant pipe EOF hiding partial output. Timing is observational, not a test threshold.

| Filter (UI rows prefixed `ui::shell::trash_tests::`) | Result and oracle |
| --- | --- |
| `domain::trash_tests` | 8 passed4.17s on final binary: exact seed/settings/used history, independently deleted children, parent/collision paths, cascades/overlaps/final-code owning-set scope, equal-time binding, dense10k relationships and max40MiB valid code graph. Repeat measured621ms preparation/644ms checked apply. |
| `ui::trash::tests` | 1 passed0s: UTC epoch/leap2000/nonleap2100/year9999 and exact u64MAX fallback. |
| `trash_context_widths_pagination_safe_dialog_cancel_focus_keyboard_and_metadata_only` | 1 passed22.05s final binary: three normal sizes, pagination, painted modal clip, coordinate press/release/wheel, keyboard/focus, Keep/Escape/backdrop and metadata-only output. |
| `trash_restore_parent_path_preserves_own_child_trash_and_duplicate_consent_is_explicit` | 1 passed10.03s final binary: parent handoff, independent child state and explicit collision consent. |
| `trash_permanent_scope_fault_retry_latest_preferences_and_encrypted_restart` | 1 passed10.52s final binary: confirm actually queues worker, true purge scope, injected encrypted commit failure preserves data/preferences, retry and encrypted reopen with latest query/sort/view. |
| `trash_empty_confirmation_last_code_scope_and_admitted_lock_clear_review_before_release` | 1 passed8.63s final binary: deduplicated Empty and last-code owning-set scope, safe confirm, held admitted write/Lock clears before release and prevents late adoption. |
| `trash_readonly_preparation_lock_and_close_discard_late_result_without_writing` | 1 passed13.00s after async preparation change: held review, immediate Lock/Close clearing, no write/no late review. |
| `trash_equal_time_stale_review_rejects_without_overwrite_and_current_review_can_retry` | 1 passed8.30s: equal logical time/content mutation rejected atomically; current-review retry succeeds. |
| `trash_close_matrix_discards_unsubmitted_views_flushes_latest_preferences_and_preserves_business` | 1 passed49.34s: six views (folder-create, Trash-review, recovery-editor, account-delete, code-delete, manual-account), dirty latest preferences, approved exit and encrypted reopen; exact business fields unchanged. |
| `trash_close_error_retry_and_explicit_discard_preserve_latest_intent_and_disk` | 1 passed17.76s: dirty Trash Close failure, retry and separately explicit Discard. |
| `trash_folder_close_error_retry_and_explicit_discard_preserve_latest_intent_and_disk` | 1 passed17.44s: folder Close failure/retry/Discard regression. |
| `trash_close_during_admitted_delete_clears_presentation_before_commit_and_never_late_adopts` | 1 passed10.05s: held admitted delete, Close privacy before release, no late adoption. |
| `trash_portable_all_kinds_restore_and_failed_successful_reused_id_import_invalidation` | 1 passed40.78s: separately encrypted source/backup/local public fixtures; all five exact kinds/IDs/used states survive backup/import; worker restores all kinds; failed changed-source Replace preserves review; successful same-ID Replace invalidates review and adopts actual source graph. |

The last seven worker/Close/portable checks preceded only descriptive `active_children`→`active_associations` field rename included in final compile; mutation behavior was unchanged. All20 newly added library cases passed individually; this is not a full-suite aggregate result. UI counts now describe active-own-state *associations*, including a last code's owning set, rather than misleadingly calling every affected record a child. Restore counts remain explicitly associated/preserved, not reactivated.

`rtk proxy python -X utf8 target/task18-example.py --nocapture --test-threads=1` ran exact `target/task18-isolated/debug/examples/trash-04677d3e93efc5ed.exe`: 1 passed0s; all five own-Trash fixture kinds present and valid, no native window.

Final static/build commands:

- `rtk cargo fmt --all -- --check`: passed.
- `rtk cargo clippy --locked --all-targets --all-features -- -D warnings`: passed, no lint allowance added.
- `rtk cargo build --locked --all-features --bin Authenticator --lib --example trash`: passed22.51s.
- `rtk git diff --check`: passed.

## Incomplete full run and retained process

`rtk proxy python -X utf8 target/task18-all.py` launched `cargo test --locked --all-targets --all-features --target-dir target/task18-isolated` at **06:22:58.770687UTC**, intended600s wrapper bound. It compiled the final196-test library, then the library process remained present. The PIPE/communicate harness exposed no partial stdout on Windows timeout; `cargo-output.txt` is empty. No trustworthy final count, failure count or pending test names is available. Result: **INCOMPLETE**, not passed. Child lifetime exceeded the intended600s bound; wrapper descendant-date comparison did not establish child exit. Cargo proxy47560 and actualCargo43336 were subsequently absent.

Owned lib PID**32032**, exact `D:/Projects/github/authenticator/target/task18-isolated/debug/deps/authenticator-2eb1a8e5551a193d.exe`, started **06:23:17.863UTC**, was freshly verified before one explicit normal termination request after06:34:40UTC. The request returned success, but fresh inventory still found the process. Final inventory: CPU1366.921875s, working set967184384 bytes, unchanged from earlier samples; inspected threads reported Wait/Suspended plus Executive/Initialized. Cause is **UNKNOWN**. No resume, elevation, repeated termination or unchanged full retry. Subsequent direct focused execution of the same compiled file passed final domain/UI checks; this does not diagnose the full-run stall.

Preserved runtime: `D:/Projects/github/authenticator/target/task18-runtime-f93a98ba-fa48-4884-aa14-c4ffd36568e3`, public-test TempDirs plus empty output/process/descendant manifests. It remains while32032 is present. Old parent lib PID**30688**, exact `target/debug/deps/authenticator-2eb1a8e5551a193d.exe`, started **04:36:46.314UTC**, also remains (CPU1380.46875s, working set1024733184); it was not terminated again. Unrelated lifecycle preview untouched. Parent obtains a new hosted full gate after review; prior successful CI17 run43370232096 verifies prior source, not Task18.

## Native attempt, cleanup and open gates

Read the Computer Use SKILL/guidance/API/confirmations. Documented `@oai/sky` owned inventory, followed by one normal argument-free launch request for exact `D:/Projects/github/authenticator/target/debug/examples/trash.exe`, failed **`GetCursorPos failed: Access is denied. (0x80070005)`**. Fresh Sky inventory had no owned app/window; exact `trash.exe` process and `authenticator-task18-*` TempDir-prefix inventories were empty before and after. No owned native resource, screenshot, physical pointer/keyboard or framebuffer proof resulted. Stopped on unchanged denial; no elevated launch, bypass, settings change or substituted production profile. Cause unknown. Native raster/focus/input, physical DPI/Narrator, offline/hardware and prior manual gates remain open.

Cleanup: native PowerShell resolved each immediate owned `target/task18-runtime-*` path inside intended workspace target, matched its manifest root/executable, freshly required its PID absent, then used `Remove-Item -LiteralPath` recursively. **24 exited-test runtime directories removed**. The single full-run runtime above was deliberately retained. No broad Temp scan, unowned vault read or unrelated deletion. Ignored Cargo artifacts and small reproducible owned harness scripts remain under target. No product user data or native output files were created.

Separate review, new-source hosted full-suite success and native/manual gates remain required. Implementation is handed off for review without marking Task18 done.
