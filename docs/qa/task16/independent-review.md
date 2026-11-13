# Task16 independent UI, security and data-loss review

13 November 2026. Separate reviewer `review_16`; implementation author `implement_16`. Read-only review against base `00b554f6690fb8f5b9c9c7d6364fb11e19827848`, including the new untracked domain/detail/test/example files. The only repository write by this reviewer is this report. No source, tests, task status, PRD, coverage, staging or commits changed.

## Verdict and findings

**No material implementation defect found in the reviewed source and isolated checks. Limited source approval for Task16. This is not full acceptance or permission to mark the task done.**

No new prioritized security, data-loss or UI deviation is reported. Existing blockers remain separate:

- **Open native clipboard gate:** actual interactive Copy previously returned `Changed` after clipboard close. Its exact failing ownership subcondition and cause remain unknown. Task16 uses that same production actor; Fake success and private-station evidence do not establish successful interactive copy, expiry or ownership cleanup. No ownership checks were weakened here.
- **Open native UI/lifecycle gates:** actual Windows pointer/keyboard, rendered pixels, physical DPI, Narrator/other assistive technology, OS lock/suspend and device/reboot paths remain unverified. The author's two documented normal owned-preview launches failed with `GetCursorPos ... 0x80070005`; this reviewer did not repeat an already unavailable native attempt. No conclusion about the cause or OS lock state is drawn.
- **Task18 cross-gate:** restoration and irreversible permanent deletion/cascading logical references are not delivered or accepted by this review. Task16 retains associated account/set/code IDs, values and history and rejects orphans; its backend restore fixture is not proof of a Trash restoration UI or permanent-delete implementation. Set/code deletion here is reversible encrypted Trash, not irreversible deletion.

## Source and data review

`src/domain/recovery_code.rs:21` captures only active parents/code. `:65` compares the selected account state and credential identity/seed/settings plus the parent-set state and selected code's state, exact value and used timestamp, so equal logical timestamps alone cannot authorize a stale operation. The selected-code snapshot is bounded; no full-set cache is added per frame.

`src/domain/recovery_code.rs:91` validates the guard and restored timestamp before changing fields. Used, set, account and aggregate timestamps use the existing clamped mutation time. Trash retains the code's value and used history. `src/ui/state.rs` hands the intent to the existing sole-store worker and `src/storage/mod.rs:275` commits a validated encrypted candidate atomically; there is no additional mutable store, crypto/schema/dependency change or raw-value error/log path.

`src/ui/shell.rs:1163` resolves Copy against the current active account/set/code IDs and sends a zeroizing value through the existing static clipboard path without a TOTP window. Copy does not write the vault, change used state or create/replace Undo. The clipboard changes are confined to `cfg(all(test, windows))` Fake/observer support. Production actor and ownership logic are unchanged.

`src/ui/shell.rs:363` records Undo from the actual adopted saved record; `src/ui/recovery_detail.rs:40` restores exact old history and selects the affected code's page before focusing it. Success, editing, navigation, pagination, Lock and successful replacement import conceal; failure retains prior committed data and a still-valid prior Undo. Nested backup Apply failure does not clear the unrelated detail session. Lock clears detail/Undo/pending bookkeeping before held I/O completes and prevents late unlocked adoption.

## UI consistency and accessibility scope

Applied `ui-consistency:verifying` and its called values/accessibility/contrast guidance. Compared one changed detail page against all eight refreshed roles in `implementation-evidence.md`, and against the actual prior detail, List/Select mechanisms, Task15 editor/dialog and Task10 card/Root mechanisms. There is only one same-kind detail reference; no peer-majority claim is made. Shell/state/tests/module files are orchestration or verification, not additional pages.

All eight roles were inspected across their source branches: account/ambiguity/header Back/Edit; service-issued/single-use guidance; available/used concealed rows; separate Reveal/Copy; explicit Mark used/available and one-step Undo; per-code and set confirmation; empty/error/copy feedback; theme/focus/narrow/Lock behavior. Additional branches inspected include set/parent Trash, stale confirmation/Undo, pagination and last-page clamping, failed writes, busy saves, successful/failed replacement import and navigation. Empty/last-page-empty and stale-dialog branches were source reviewed, not separately exercised by a new probe.

No new source-level difference from the agreed roles/shared mechanisms was found. Existing header/footer Back controls are retained; the duplicate generic detail heading is removed in favor of the account heading. Two explicit action columns use shared buttons. Values use shared BODY styling and a bounded nonselectable holder; actions precede long content. All secret-bearing control/status/live-region names remain static or code-number based. Revealed label/TextRun content deliberately contains the value; concealment is not a guarantee of erasing framework/allocator/OS memory copies.

The applicable Root theme is the existing dark scheme: bundled Noto Sans 400/600, BODY14/20, LABEL14/20, CAPTION13/20, TITLE24/32, SECTION18/24, shared 4-point spacing, 16 padding and 36-point controls. No root `DESIGN.md` was found. No new palette, opacity or font system is introduced. Shared opaque blue icon tiles retain contrasting strokes.

Separate accessibility observations:

- **Contrast:** source-resolved shared pairs pass the project's 4.5:1 text and 3:1 essential-graphic thresholds. Independently printed values: TEXT/SURFACE 14.32:1; TEXT/BACKGROUND 16.73:1; TEXT/HOVER 11.41:1; MUTED/SURFACE 8.12:1; MUTED/HOVER 6.47:1; ON_BLUE/BLUE 5.85:1; ERROR/ERROR_SURFACE 8.69:1; SUCCESS/SURFACE 10.46:1; BLUE/SURFACE 5.01:1. Focus against background/surface/field/hover is 9.49/8.12/8.80/6.47:1. Actual rasterization and disabled-state compositing were not evaluated visually.
- **Focus:** Context tests verify live, in-viewport header Back, the changed code's current used control, cross-page Undo, remaining Reveal after code Trash and current editor Delete after cancellation. Safe initial dialog focus and Escape/Keep/backdrop return are covered by the existing direct-delete regression. Native focus appearance remains open. Post-failure focus placement beyond the tested cancellation/success cases was source inspected but not separately asserted.
- **Keyboard:** Context Tab/Enter/Escape events exercise Reveal/Conceal, independent Copy, Mark/Undo, Edit and delete routes. This is framework input evidence, not actual Windows keyboard proof.
- **Labels:** action AX names identify the code number without its value; state text says Available/Used. The detail adds no editable input field. Native screen-reader output remains open.
- **Text alternatives:** actions have text/code-specific names and the header recovery tile is decoration; state does not depend on color alone.
- **Targets:** Context geometry checks cover the two-column controls at 340/600/960 widths, at least 100 logical points wide and the shared 36-point minimum height. Physical target size, raster overflow and overlapping native hit regions remain unverified.
- **Structure:** Root heading and account level-2 heading use shared semantic Heading widgets; controls use Button roles and feedback uses shared polite Status. Rows use the project's generic card/label structure. No claim of complete accessibility conformance is made; other WCAG criteria were not checked.

Original reference screenshots remain unavailable; `docs/reference-ui.md` is the inventory fallback. No native raster screenshot was produced or judged. Context geometry cannot establish final visual hierarchy, wrapping, painted focus/contrast, gestures or real pointer capture. The existing Task04 calibration (`docs/qa/task04/calibration.md`) detected TITLE600-to-400 in a detached copy at `C:/Users/root/AppData/Local/Temp/authenticator-ui-cal-4cg1yjfn/checkout`; it proves source sensitivity only. That copy was removed, and copied-executable/live-pixel sensitivity and interaction detection remain unproven. No new calibration was run by this checker.

## Independent checks

Cargo commands ran sequentially; no broad all-target build or full-suite repetition was needed. All runtime data was generated public `TempDir` data. UI Copy tests inject Fake before actions; no interactive user clipboard, real/default/reference vault, credential or backup was accessed. No native window was launched, no unrelated process was stopped, and no settings were changed.

| Exact command | Independent result and useful oracle |
| --- | --- |
| `rtk cargo test --locked --all-features --lib domain::recovery_code -- --test-threads=1` | 2 passed, 163 filtered, 0.00s. Clock clamp, exact original used history, parent/aggregate updates, invalid historical date rejection, equal-time value/account changes, hidden parents/codes and orphan validation. |
| `rtk cargo test --locked --all-features --lib recovery_detail_tests -- --test-threads=1` | 7 passed, 158 filtered, 184.95s. Context press/release coordinates + wheel and Tab/Enter/Escape; three normal viewport sizes; 4096-byte exact Copy/long-value bound; concealed AX/no framework CopyText; Copy preserves ciphertext/used/Undo; Fake Changed/expiry-without-frames/Lock/newer-identical handling; used/Undo/encrypted reopen and portable export-decode IDs/history; write conflicts/retry; admitted-save Lock; failed/successful reused-ID import; generated parent restore association; later-code/deep-list/cross-page focus. |
| `rtk cargo test --locked --all-features --lib recovery_direct_detail_delete_cancel -- --test-threads=1` | 1 passed, 164 filtered, 40.67s. Keep/Escape/backdrop cancel targets the live editor Delete button at normal 340/600/960 viewports, ordinary and oversized sets, without ciphertext mutation. |
| `rtk cargo test --locked --all-features --lib backup_display_placeholder_collision -- --test-threads=1` | 1 passed, 164 filtered, 6.63s. Missing identifier versus literal `No account identifier` keeps full UUID disambiguation and exact selected IDs through both handoffs. This older test uses 16000-height layout, not normal-height native visibility proof. |
| `rtk cargo test --locked --all-features --lib actor_expiry_hidden_lifecycle -- --test-threads=1` | 1 passed, 164 filtered, 1.30s. Existing Fake-backed actor expiry/lifecycle/Lock/shutdown and newer identical external fixture ownership. |
| `rtk cargo test --locked --all-features --lib held_copy_admission_and_already_admitted -- --test-threads=1` | 1 passed, 164 filtered, 0.03s. Existing actor admission/stale-success handling with Fake. |
| `rtk cargo test --locked --all-features --lib actual_text_and_essential_graphic_pairs -- --test-threads=1 --nocapture` | 1 passed, 164 filtered, 0.00s. Shared project contrast thresholds. Repeated once as `rtk proxy cargo test --locked --all-features --lib actual_text_and_essential_graphic_pairs -- --test-threads=1 --nocapture` solely to expose ratios hidden by RTK's summary; same pass. |
| `rtk git diff --check` | Passed. |

The author's full-suite/static/build results were read as prior evidence, not counted as this reviewer's execution. Portable inspection here confirms encrypted export/preview decoding and metadata preservation, not a native restore interaction. No Task17 or Task18 work was performed.

Review cost estimate: 1 target detail page, 8 checklist roles and the relevant List/Select/editor/2FA/shared references; approximately 24 project files opened and 17 searches. Skill instruction files are additional reading, not product pages. No external disposable probe artifacts were created.
