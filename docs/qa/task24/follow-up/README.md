# Task 24 follow-up implementation and review handoff

14 November 2026 local. This is author evidence, ready for a distinct read-only reviewer. It does not change task statuses or constitute full acceptance. The parent owns integration, current hosted CI and the clean committed package. Base source is `2e2da2cba35e99d6d6d218471a12f48d67e7abc0`; the measured working tree is dirty.

## Material changes

The hosted run 43515853116 passed formatting and strict lint but failed `backup_import_normal_viewports_keyboard_scroll_reaches_each_primary_action`. Local reproduction confirmed a real focus-visibility defect: at 600 × 800, the focused Review button occupied y=777..813 and extended 13 pixels below the viewport after 30 settle frames. The original visibility oracle was preserved and its failure message now includes the label, viewport and rectangle.

The shared focus helper now performs minimal reveal for three actual frames after gained or first-observed focus. This covers late wrapped-page layout and programmatic `request_focus()` called after widget creation. Sizing passes do not consume extra frames. Wheel input, pointer press or lost focus cancels the temporary reveal budget, preserving ordinary scrolling while the control keeps keyboard focus. Only temporary frame/focus metadata is stored. `response.has_focus()` is read outside the Context data write lock.

Native testing of the first correction found a second real defect: the initial Cancel action in the large Replace dialog remained clipped after 400 ms. The final first-observed-focus correction fixes that case. The two component regressions independently exercise keyboard focus and programmatic focus, late geometry changes, whole-control clipping bounds, and a subsequent wheel movement greater than 100 points while focus remains. This is a behavior fix, not a longer test wait or a weakened oracle.

Packaging now uses `/Brepro`, a source epoch for ZIP/build metadata, stable relative notice numbering, deterministic JSON/ZIP attributes and narrow checkout-byte attributes. Upstream notice provenance and static CRT checks remain mandatory. `source_timestamp_utc` means the source timestamp, not the actual build wall time. The QA helper supplies a separately encrypted 257-account public backup for actual pagination and Replace checks; production data paths and guards were not relaxed.

## Validation

The original failing import-navigation test was red locally in 16.06 seconds and green on the final source in 16.81 seconds. The keyboard late-layout regression was red against the original helper (rect 424..460, clip bottom 392); the programmatic-focus regression was red against the first correction (rect 752..788, clip bottom 392). Both red probes completed in 0.03 seconds. The final `focus` filter passed ten tests in 23.60 seconds:

- `focused_action_rechecks_settled_geometry_then_allows_pointer_scroll`
- `requested_focus_after_widget_creation_is_revealed_and_wheel_remains_free`
- `folders_pointer_keyboard_widths_multiselect_discard_and_focus`
- `export_keyboard_submit_focus_survives_validation_correction_and_keep_existing`
- `recovery_cancel_focus_masks_widths_and_oversized_existing_set_are_safe`
- `export_synthetic_root_overwrite_cancel_focus_busy_success_and_back`
- `detail_trash_confirm_cancel_focus_fault_retry_and_account_associations`
- `detail_later_code_and_scrolled_list_focus_are_visible_after_save_undo_and_trash`
- `trash_context_widths_pagination_safe_dialog_cancel_focus_keyboard_and_metadata_only`
- `initial_subpage_focus_keeps_heading_and_first_control_visible` (45 page/width/zoom cases within this test)

Commands used the owned D: scratch wrapper `rtk proxy python -X utf8 target/follow24-cargo.py`, which sets fresh TEMP/TMP and `CARGO_BUILD_JOBS=2`, then invokes `rtk proxy cargo` with the supplied arguments. The targeted commands were `test --offline --locked backup_import_normal_viewports_keyboard_scroll_reaches_each_primary_action -- --nocapture` and `test --offline --locked focus -- --nocapture`.

Final frozen-source checks are retained in [exact commands and timings](final-check-results.json) and `final-check-0.log` through `final-check-4.log`: formatting 0.818 s; strict all-target Clippy 3.902 s; Settings heading/counts/first-action paint regression 12.938 s; triggering-frame lock privacy regression 23.862 s; and all 17 Python packaging regressions, 0.944 s test body / 1.103 s wrapper. [Every Python test name](package-test-names.json) is retained. No current full-suite hosted pass is claimed; the parent must run it after integration.

## Native scope and source attribution

The [additional frame manifest](frame-manifest.json) contains 59 author-inspected frames: 57 masked PNGs plus two empty/locked production JPEGs. Every PNG passed signature, chunk CRC, inflate, IEND, metadata/ready and SHA256 checks before copying. Masks were applied in memory before disk. Production screenshots contain only empty or locked blank fields. No source QR, live OTP, recovery value, filled phrase or raw speech transcript is retained.

These frames supplement the earlier [195-frame manifest](../frame-manifest.json). They do not change the earlier distinct review's 68 current plus 17 historical images actually viewed, or imply that its other 110 historical frames were viewed. Distinct review of these 59 new frames is pending.

| Evidence root | Exact attribution and observed work |
|---|---|
| `wTgU2q` | Base 2e2da2c minimal-focus source, before the final shared-focus changes. Blank manual nine-cell grid; empty Save error; advanced controls and actual Algorithm menu; optional recovery input focus; discard; mixed capture and actual overlay Ctrl+L. Exact helper binary SHA was not measured before rebuilding, so this is source-attributed historical evidence. |
| `LbDnux` | Intermediate three-frame correction, before first-observed focus. Settings heading; real import busy/wrong-phrase error; stable Review visibility at 340/600/960; real wheel while focus remains; 257-record candidate and actual page 2; Apply visibility. Frames 4/5 are intermediate selector/Help focus, not primary-action passes. Frames 13/14 show the Cancel defect, not a passing dialog. Exact helper binary SHA was not measured before rebuilding. |
| `q9qZkB` | Final shared-focus/helper source. Debug helper SHA256 `cfa1d7a7553034404049782af412f082148eb8e946d75a608fb7b32143f4680b`, 41,890,816 bytes. Initial Cancel nine-cell grid, keyboard Confirm, reopened Cancel, actual Replace success, Unicode no-results, current account-editor nine-cell grid and deep Save. Controlled idle and Narrator observations use this fixture. |
| Production | Extracted final EXE SHA256 `9656ed317d6b2e36a96cf17ac5c0d2729680cb6abb850941c295f7418272978e`. Real setup, QR scan, editable concealed review, explicit Save, exact-record HMAC, Copy, encrypted export, independent-key restore, restart and lock. |

The nine-cell grids use logical widths 340/600/960 and **application zoom** 100/150/200 percent on a 96-DPI desktop. Zoom reduces the available logical height at large text sizes. These are separate from Windows physical scale. A viewport image does not prove all content below it, and WGC can include offscreen or occluded pixels. The [state-cell table](state-coverage.md) identifies the actual states and remaining unrendered combinations.

## Mixed monitor capture and lock

With primary physical DPI 144, left DPI 96 and application zoom 1, the independently reported QR bounds were [-1824, 411, -1392, 843]. An actual drag without a ScreenshotId, overlay-relative (96, 411) to (528, 843), produced F12 selected bounds left=-1825, top=411, width=433, height=432. The QR was visible in the foreground on the left monitor. It decoded on this measured attempt, opened concealed review and preserved the ciphertext until explicit Save. Exactly one Public Fixture record in that fresh root matched an independent SHA1 HMAC at 22:18:22.807 UTC. The attempted `type_text` rename was not delivered; no unique edited-name success is claimed for that historical case. The later production case did successfully edit a unique name with `set_value`.

No capture-backend change was warranted by this result. The earlier two NoQR attempts remain failed observations with unresolved input/visibility causes; this successful measured crop does not retrospectively diagnose them. [Selected bounds](wTgU2q/capture-bounds.json), [mixed result](wTgU2q/mixed-result.json) and [actual overlay Ctrl+L result](wTgU2q/overlay-lock-result.json) are retained. Real `Control_L+l` canceled the overlay, returned Locked, removed private AX content and preserved ciphertext. This proves app-user Lock, not WTS/suspend delivery.

The controlled idle test used the disposable fixture's one-minute setting after its encrypted preference write settled. At 55.134 seconds the overlay was still active; by 79.697 seconds it was gone and the main app was Locked, with private AX absent and ciphertext unchanged. The honest measured bound is **55.134–79.697 seconds**, not a fabricated 65-second endpoint. [Exact result](q9qZkB/timed-idle-result.json). The earlier 385.451-second endpoint is retained as coarse history, not a measured one-minute pass. An abandoned background observer reset the REPL and produced no timing result. The final awaited observation is separate. [Default 300 seconds was restored and saved](q9qZkB/idle-default-restored.json).

## Narrator observation limit

The populated live Codes page remained foreground with Screenshot to Scan focused before/after a 31.049-second no-input observation. Counter 59820405 advanced to 59820406 and the displayed public code changed. These are valid rollover and non-code-focus observations. However, official Speech Recap exposed only window/title-bar chrome: 253 tree characters, zero document characters and zero known static-label occurrences. A foreground positive-control retry on a safe no-results page exposed the same limit. Therefore unchanged chrome cannot establish silence: **automatic code announcement behavior remains unverified**. The earlier historical Help/Close speech claims are not a positive control for this observation and require independent substantiation if they used the same inaccessible history.

Only boolean/count evidence is stored in [rollover](q9qZkB/narrator-rollover-result.json) and [positive control](q9qZkB/narrator-foreground-positive-control.json). Official Insert+Escape did not reliably close the later session. The visible Narrator Home Exit Narrator button did; [final process/window absence](q9qZkB/narrator-exit.json) was checked, with no termination bypass.

## Final package runtime journey

The initial private Python 3.13 tempfile extraction had an ACL that denied the normal UI launch (ShellExecute error 5). It was preserved unchanged. A separate fresh directory with ordinary inherited permissions contained only public program files. The EXE hash was rechecked there, then launch succeeded. No global policy or existing ACL was changed, and no cause is inferred for older unrelated launch failures.

The actual process PID 14476 reported the exact public EXE path/hash. `GetPackageFullName` returned 15700 (no package identity). Separately, the ordinary `AppData/Local/Authenticator` path was absent, while a previously empty Codex LocalCache Authenticator directory received the fresh `session.lock`. This is an observed data-root fact, not an inference from package identity, another Windows user or a clean offline machine. Production code still uses the normal known-folder lookup, with no QA data-root override.

The [runtime result](production-smoke.json) records the complete sequence. A random local phrase created the first vault. The public QR decoded on the negative monitor; its service name was changed to Task24 Production Public Fixture, review stayed concealed, and explicit keyboard Save created exactly one record. Independent HMAC matched at counter 59820459; real keyboard Copy matched at counter 59820461. Export wrote a 406-byte authenticated ciphertext with a different random backup phrase. After normal exit, only the proven fresh vault was copied/verified into the private evidence root and removed from the runtime root. A second local vault with a different random phrase started empty. Actual picker selection, authenticated preview and explicit Apply restored exactly one record; independent HMAC matched at counter 59820473. Normal restart opened Locked with private AX absent; the second local phrase unlocked it and HMAC matched again at counter 59820476. Real Ctrl+L locked with unchanged ciphertext and cleared private AX.

All owned production/QR windows closed normally. Authenticator.exe and Narrator.exe process checks were absent. Only proven fresh vault/session files were removed after ciphertext copies and hashes were verified; the preexisting redirected directory itself remains, empty. The ordinary path is still absent. Ciphertexts remain ignored in `target/task24-production-k6xgp5nt`; none is copied into Git. Random phrases existed only in the REPL and were not persisted.

## Release reproducibility and remaining acceptance

See [full rebuild proof](reproducibility.md). Two real builds from empty host and target release output directories each compiled 183 crates, then linked and packaged identical EXE and ZIP bytes. The source commit is still the dirty base identity; parent clean-commit packaging must regenerate metadata and verify the EXE hash before reusing this runtime proof. The prior 9165… and b7ac… artifacts predate final focus changes and are historical only.

Remaining acceptance includes distinct source/security/visual review; current hosted full tests/build/package; clean committed source/artifact reconciliation; effectively disconnected first use without Rust and recovery in another Windows user/isolated installation; actual WTS/suspend/logon delivery; and exact installed private Windows reference comparison. Speech announcements remain unverified due the explicit extraction limit. The software state combinations not rendered in [state coverage](state-coverage.md) remain unverified, not external blockers or automatic passes. The parent must reconcile those against the full task checklists and existing separately reviewed evidence.

At handoff all three monitors are 100 percent, Contrast is None, Narrator is off, the disposable auto-lock preference is 300 seconds and no native exclusivity remains. The older Task 21 workflow helper and Task 23 Explorer were left untouched. No task/index status, commit or push was performed by this implementation child.
