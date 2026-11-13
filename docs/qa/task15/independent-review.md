# Task 15 independent review

13 November 2026. Distinct read-only reviewer `review_15`; implementation author was `implement_15`. Reviewed the working source against main `dec798caeff6b859cea38330b8a73b05ba027a22`, the complete Task15 task, and the refreshed eight-position checklist in `implementation-evidence.md`. Source/tests were not edited. This review does not certify task status or later-task completion.

## Finding requiring correction

**P2 — cancelling Delete Set from the detail screen loses keyboard focus.** At `src/ui/shell.rs:1135–1142`, the detail action creates the editor and calls `editor.request_delete(opener)` with the detail screen's Delete button ID. The detail screen is then replaced by the editor. `src/ui/recovery_editor.rs:362–366` preserves that vanished opener, while `src/ui/components.rs:1003` / `1056` restore focus to it after Keep editing/Escape/backdrop. The editor's own Delete button has a different ID. The result is no focused element, rather than the actual opener or a deliberate current-screen replacement.

Independent reproduction used the real public `AuthenticatorApp::with_repository` with `CreationContext::_new_kittest`, an isolated owned encrypted public fixture and Context pointer press/release events: View set → Delete backup-code set… → Keep editing → six settling frames. Output: `detail opener=15056542516250788156`, `current editor delete=Some(17104408732404784349)`, `restored focus=None`, `label=None`. These numbers identify this disposable run only. Stored codes were unchanged. This affects the direct detail Delete entry, including large preserved sets; opening Edit first and deleting from inside the editor has a live opener. Correct by preserving/restoring the detail view on cancellation or deliberately returning focus to the actual editor control, with coverage of this direct entry. The existing new focus test covers Cancel inside the editor and therefore misses this route.

No additional material security or data-loss defect was found within the inspected source and deterministic checks. Current verdict: **source/Context changes requested for the P2 above; native acceptance remains open independently.**

## Scope and evidence

The eight checklist regions were inspected in order: ROOT/selected identity; shared concealed labeled multiline; local file action/guidance; count-only Preview; exact replacement/used/Trash explanation; Save/Cancel/Delete confirmations; fixed errors and retry; theme/focus/scroll/Lock. Source branches inspected include new/existing sets, large read-only prefill refusal, set-in-Trash conflict, busy picking/preview/save, invalid input/read failure, duplicate approval/reset, replacement/delete/discard/file confirmations, failed save/retry, Help and Lock. `shell.rs`, `state.rs`, module barrels, platform adapters and the example are integration/support files, not additional UI pages.

Source findings support these boundaries:

- Shared normalization enforces raw-byte, line and normalized-code bounds, initial BOM, LF/CRLF, physical blanks and case-sensitive duplicates. Preview approval is tied to the exact current input. File failure/cancel retains earlier edits/preview; valid file replacement requires deliberate review.
- Borrowed exact-value tree indexes remove the earlier quadratic matching path. The mutation snapshot checks complete current account credential/state and set/code contents, including equal-time changes. Exact values retain IDs/used history; omitted active codes become encrypted Trash; explicitly re-added Trash codes preserve history. Total limits include Trash. Transactions validate/encode/encrypt before atomic commit; no second mutable repository/key owner was introduced.
- The Windows reader rejects unsupported namespaces, ADS/traversal/ambiguous ordinary names and remote-drive types. It opens root-to-leaf no-follow directory handles with restrictive sharing, validates the no-follow leaf handle, and bounds the actual read to MAX+1 independently of metadata length. It never writes or persists the selected source. This is not a kernel/administrator/drive-namespace/filesystem certification. Original user input files are intentionally preserved.
- Lock drops owned editor/preview/snapshot/job state and clears widget memory before admitted I/O can finish. Post-admission encrypted writes may finish; the UI must not claim rollback or adopt late success. Revealed text intentionally reaches layout/AX; no complete dependency/allocator/OS memory-erasure claim is justified.
- Task14 placeholder-collision assertions remain intact. The seven-line test change uses actual Cancel→Discard between cases because the new editor owns the form; it does not weaken any UUID assertion. Selected identity shares `display_identity` with the list/summary.

Shared Dialog reach was inspected across editor confirmations, Task09 account discard, Task10 account Trash, Task12 overwrite, Task13 collection replacement, and Help/About. The sizing pass and title-specific non-dragging ScrollArea are shared changes. A disposable external probe compiled the current `theme.rs`/`components.rs` directly, with only their unrelated Route/constant dependencies stubbed. For short Save followed by taller Delete, Keep editing had focus on the **first visible frame** at 340×780, 600×800 and 960×900. Tab reached confirmation and center press/release submitted it at all three sizes. At an additional 340×300 stress size the taller confirm initially had zero clipped height; Tab scrolled it fully into view, and settled center press/release submitted it. The initial probe pressed while that scroll animation was moving and missed; after allowing settling it passed. This is Context geometry/hit testing, not OS pointer or raster evidence. The direct-detail opener defect above was reproduced separately with the actual app.

Independent commands (all successful; tests/build/link operations sequential):

| Command | Result |
| --- | --- |
| `rtk cargo test --locked --lib recovery_ -- --nocapture` | 19 passed, 136 filtered, 27.02 s |
| `rtk cargo test --locked --lib export_synthetic_root_overwrite_cancel_focus_busy_success_and_back -- --nocapture` | 1 passed, 154 filtered, 12.87 s |
| `rtk cargo test --locked --lib export_keyboard_submit_focus_survives_validation_correction_and_keep_existing -- --nocapture` | 1 passed, 154 filtered, 6.11 s |
| `rtk cargo test --locked --lib backup_import_normal_viewports_keyboard_scroll_reaches_each_primary_action -- --nocapture` | 1 passed, 154 filtered, 24.49 s |
| `rtk cargo test --locked --lib backup_root_empty_populated_no_match_selection_back_help_and_lock -- --nocapture` | 1 passed, 154 filtered, 17.38 s |
| `rtk cargo test --locked --lib backup_display_placeholder_collision_has_exact_ids_in_list_summary_and_both_handoffs -- --nocapture` | 1 passed, 154 filtered, 6.36 s |
| `rtk cargo test --locked --lib actual_text_and_essential_graphic_pairs_meet_project_thresholds -- --nocapture` | 1 passed, 154 filtered, 0.00 s |
| `rtk git diff --check` | passed |

Focused recovery tests exercise ordinary real file reads, BOM/endings/blanks/Unicode/duplicates, exact approval, size/code/line/global limits, actual leaf/parent symlink rejection, equal-time stale content, reversible ID/used preservation, creation equivalence, write-conflict/retry/encrypted reopening/portable history, masked AX, read-only oversized set, and held admitted-save Lock without late adoption. Their editor click helper generally uses 600×16000, so those successes alone do not establish normal-height reachability. The separate ordinary-size Dialog probe addresses its specific shared component only. The author's reported full suite, strict clippy, fmt and production/example build were read but not independently repeated.

The external probes lived only under `C:/Users/root/AppData/Local/Temp/authenticator-task15-review-dialog-20261113`. The standalone Dialog probe initially needed the already-installed Windows native library search path, and one initial run needed explicit egui texture-delta disposal; these were probe setup corrections, not product failures. Both final probes compiled and ran. Their exact owned directory, executables, source, symbols and encrypted fixture were verified and removed after exit. No unrelated lifecycle preview, real/default/reference vault, credentials, backup or clipboard contents were inspected or changed.

## Unevaluated criteria and remaining gates

- **Actual pixels/contrast:** no native Task15 frame was rendered by this reviewer. Source foreground/background pair selection follows the sole dark theme and passed the existing calculated contrast targets; actual raster contrast, clipping, text quality and visual hierarchy remain unverified.
- **Focus:** first-visible safe Dialog focus and ordinary Cancel return pass Context checks; direct detail Delete cancellation fails as reported. Native focus presentation/OS restoration remain unverified.
- **Keyboard:** deterministic Dialog Tab/scroll and prior import/export keyboard routes pass; actual native Task15 editing, all form actions, native picker and keyboard paths remain unverified.
- **Labels:** source label association and masked PasswordInput/AX behavior were inspected/tested; Narrator interpretation is unverified.
- **Target size:** shared 36-point controls and settled Dialog Context hit rectangles were inspected; native hit targets/DPI are unverified.
- **Structure:** ROOT/section heading roles, shared holder and form ownership were inspected. Actual native reading order, nested scrolling and complete viewport overflow/overlap remain unverified.
- **Native workflow:** reviewer made no further launch attempt because the author's two documented normal launches already failed before an owned window existed with GetCursorPos access denied. Cause remains unknown, not proof of OS lock. Native pointer/keyboard/scroll/picker cancellation/Lock, actual 340/600/960 images, and complete 14→15→encrypted Save/native restart remain **OPEN**. Earlier hardware/DPI/OS lock/suspend, actual disconnected-machine, clipboard ownership-cause and native backup restore gates are not closed here.
- **Reference fidelity:** original safe reference images are unavailable; inventory fallback was used. Task04's parent calibration at the removed detached copy `C:/Users/root/AppData/Local/Temp/authenticator-ui-cal-4cg1yjfn/checkout` detected TITLE SemiBold600→Regular400 at source/font selection. It did not test clipping, pixels, native input or this editor's focus path.
- **Later scope:** Task16 detail Reveal/Copy/Mark used/Undo/per-code deletion and Task18 restore remain pending. Their absence is not reported as a Task15 implementation defect. No mandatory native gate is waived by this source review.

Whole-family comparison: one new recovery form with the branches above compared against account review, export and backup-code handoff, plus the shared confirmation family. One material difference was found (direct detail Delete cancellation focus); native portions remain parked for unavailable evidence. Cost estimate: 4 form/page families plus shared dialogs, approximately 27 project/source/doc files opened and 17 grouped discovery searches; 4 skill/reference documentation reads. Counts are estimates, not a measured convention or completion proof.

## Focused re-review of parent correction

13 November 2026, same distinct reviewer after parent edits and source freeze. **The P2 above is resolved at source/Context scope.** No remaining material finding was identified in the focused correction. The original finding and evidence above describe the pre-fix source; this later verdict supersedes its changes-requested status.

Read `parent-fixes.md` and the actual current `backup_codes.rs`, `shell.rs`, `recovery_editor.rs` and new regression. The detail Delete action now carries only the selected account ID. `shell.rs:1139` calls `request_delete_on_open`; the deferred flag is consumed at `recovery_editor.rs:581–583` only when the real editor Delete button has rendered, and the dialog stores that button's ID. Keep editing, Escape and backdrop deliberately remain in the editor and restore focus to its live Delete control. Deletion initiated inside the editor continues to use its own live opener. This correction changes neither the shared Dialog nor the transaction/file/cryptography/lock admission paths.

Independent sequential checks:

- `rtk cargo test --locked --lib recovery_direct_detail_delete_cancel_returns_to_current_control_at_normal_viewports -- --nocapture` — **1 passed, 155 filtered, 36.66 s**. Inspected its oracle: actual list→detail→editor ownership; normal and >1 MiB preserved read-only sets; 340×780 / 600×800 / 960×900; Keep editing / Escape / backdrop; safe modal focus; replacement focused ID differs from vanished detail ID; focused editor Delete has visible viewport bounds; full snapshot and committed ciphertext unchanged; actual Cancel→Discard between entries.
- `rtk cargo test --locked --lib recovery_dialog_defaults_to_keep_editing_escape_returns_to_actual_opener -- --nocapture` — **1 passed, 155 filtered, 9.37 s**. The original inside-editor Cancel safe-focus/Escape/return-to-Cancel regression remains passing.
- `rtk git diff --check` — passed.

No broad full-suite/native rerun or new external probe was needed for this bounded fix. The parent's reported final full suite/build/clippy/fmt results are recorded in `parent-fixes.md`, not independently re-certified here. The prior external probe directory remains removed.

The earlier **340×300** probe was an additional synthetic component stress check, not a supported application-size acceptance claim. Production `src/main.rs:13` configures minimum inner size **320×460**. That probe does not establish production app-zoom, physical DPI or native behavior. Mandatory ordinary viewport/native gates remain exactly as stated above.

Final source/Context verdict: **reviewed and accepted for Task15's implemented editor scope, with native/manual acceptance still OPEN**. This is not full Task15 native completion, Task16/Task18 acceptance or closure of any earlier manual gate. Focused added cost: 5 project files, 2 grouped searches; combined review estimates approximately 29 distinct project/source/doc files and 19 searches, with the same 4 form/page families and shared dialogs.
