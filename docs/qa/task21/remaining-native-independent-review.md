# Independent review of remaining native workflows

13 November 2026. Distinct read-only reviewer `review_native21`, using
`ui-consistency:verifying` and its accessibility guidance. No production source,
task status, vault, clipboard or native window was changed. This report is the
only review artifact written.

## Findings

**P3 — folder-resolution caption overstates the collision type.**
`src/ui/backup_import.rs` labels `local_global_id_collision` as “cross-type ID
collision.” In `src/backup/merge.rs`, the value checks `local_record_ids`, which
includes the same folder ID as well as other record types. The existing-folder
Replace frame `restored-010.png` therefore reports a cross-type collision that
the boolean does not establish. Use “ID already in current vault” or equivalent.
Resolution mapping and the reviewed candidate remain correct; this is factual
review wording, not an identified data-loss defect.

**P3 — existing account identity is repeated in Trash account review.**
`source-010.png` shows the same Account ID twice. `trash::identity` calls
`backup_codes::identity(..., true)`, then prints the generic Item ID. Suppress
only the second Account ID when the account identity was actually rendered;
retain fallback identity and distinct set/code/membership IDs. This pre-existing
redundancy is separate from the eleven navigation-copy changes.

No material security/data-loss or visual defect was found in the bounded source
changes or supplied evidence. Correct the two minor captions before integrating
this polish. Task21's full acceptance remains open.

## Compared evidence and source

Opened **all18 actual PNGs** under `remaining-native/`, comparing the set against
the Task12–19 screen checklists, shared components/theme and PRD inventory:
recovery entry/duplicate preview; revealed-but-redacted and used detail; empty
folder manager and membership selection; Trash/restore/permanent-delete scope;
wrong-phrase import; authenticated counts at340/960; restored Codes; Keep-current
and Replace review/confirmation; final Settings; corrected native help.

No readable OTP, seed, recovery value or passphrase appears in those retained
frames. Test names, public UUIDs and the owned backup path are intentionally
visible. The code rectangle and recovery input/revealed holders are masked.
`workflow_qa` applies current-frame masks before its PNG write; source enrolls
recovery-editor/detail bounds. The generated input, backup and owned vault path
are ignored by Git. This checks these artifacts, not arbitrary future screenshots
or all possible sensitive metadata.

The eleven arrow-to-English substitutions retain the same shared text helpers,
navigation meaning and behavior. The pre-fix empty-folder frame shows the missing
glyph; `guidance-fixed-600.png` shows readable “Trash in Settings” from the fresh
compiled helper. The other frames run before that copy-only rebuild and are not
evidence that the new words were rendered in every location.

Accessibility subjects were assessed separately:

- **Contrast:** resolved shared dark surfaces and opaque text/action/icon/focus
  pairs in `theme.rs`/`components.rs`; the existing arithmetic test passed its
  project4.5:1 text/3:1 meaningful-graphic thresholds. Disabled paging/Undo controls
  are visibly disabled and are not mistaken for muted active controls.
- **Focus:** supplied340 recovery Copy and other action frames have visible
  outlines. Some modal frames deliberately show scrolled body positions; they do
  not establish initial full-modal fit or every runtime focus transition.
- **Keyboard reach/order:** author reports actual Tab/Shift+Tab/Space and safe
  Replace cancellation. Independently repeated the existing three-width
  Settings/help/focus/locked-privacy Context regression; this is not native input.
- **Labels:** inspected reuse of shared labeled controls and source roles;
  snapshots show meaningful field/action labels. No independent complete native
  accessibility-tree or screen-reader check was performed.
- **Text alternatives:** sampled active icons are shared vector text-backed or
  named controls; no missing alternative was identified in the examined regions.
- **Targets:** inspected shared36-point button/interact sizing and wrapped
  controls, without claiming every runtime hit area was measured.
- **Structure:** sampled headers and sections reuse shared heading roles and
  root dialogs. Full native reading/announcement order remains unverified.

The restored screenshots consistently show0 available/1 used and the original
two Trash entries; visible deletion dates match the source Trash frame. The
Keep-current review shows0 used retained; Replace review shows1 used restored.
The author accurately distinguishes explicitly imported display settings from
retained local security/startup. Pointer workflows, cipher-hash cancellation
checks, file picker operations and independent HMAC match remain **attributed
author evidence**, not native actions or decryption repeated by this reviewer.
This review does not establish byte-identical complete vaults.

## Independent checks

Each Cargo command ran in a fresh PowerShell child with `TEMP` and `TMP` set only
for that process to a newly created owned directory matching
`D:/Projects/github/authenticator/target/review21-build-temp-<GUID>`. No global
environment/configuration was changed. Parent released the verified Cargo-cache
recovery before these commands.

| Command | Result |
| --- | --- |
| `rtk cargo test --offline --locked --lib platform::clipboard -- --nocapture --test-threads=1` | 6passed,1direct helper ignored,204filtered,4.42s; parent case exercises the isolated native helper |
| `rtk cargo test --offline --locked --lib ui::shell::settings_tests::settings_context_widths_help_keyboard_startup_and_locked_privacy -- --exact --nocapture --test-threads=1` | 1passed,210filtered,9.38s |
| `rtk cargo test --offline --locked --lib ui::theme::tests::actual_text_and_essential_graphic_pairs_meet_project_thresholds -- --exact --nocapture --test-threads=1` | 1passed,210filtered,0.00s |
| `rtk git check-ignore` with the exact paths below | all three paths ignored |

```powershell
rtk git check-ignore -- target/task21-full-626a58c7-c659-4d13-b6fd-a6a590f0d6d7.authpbk target/task21-input-7cfcd397-e477-4d26-b578-6d6c0dfda303.txt target/task21-owned-qm4ORE/vault/vault.bin
```

The parent test-only `consume_unavailable_read` CAS helper preserves the former
counter's sequentially consistent ordering and checked decrement, loops on a
spurious/concurrent compare-exchange failure, and does not underflow at zero.
Both call sites and the helper are test-only; production clipboard authority is
unchanged. The native fault cases above exercise transient marker/Unicode reads,
rejected-copy cleanup, cancellation and later-writer preservation. Current hosted
compiler/full-suite evidence is still required; local tests do not relabel the
earlier hosted lint failure as a pass.

## Unevaluated and calibration

No native session was needed or performed by this bounded reviewer. No source
export form screenshot, complete page/state/100–150–200% matrix, large-vault
pagination/performance, every conflict option, true offline-machine run, physical
mixed DPI, actual OS lock/suspend/logon, Narrator, high-contrast or packaged-profile
execution is certified here. Exact reference pixels were unavailable; comparison
uses PRD inventory and the shared Rust family. Other accessibility criteria were
not checked, so this is not a WCAG conformance verdict.

Reused calibration: the prior detached worktree at
`C:/Users/root/AppData/Local/Temp/authenticator-ui-cal-4cg1yjfn/checkout` (reported
removed) contained TITLE weight400 instead of600, detected by a separate checker.
Copied executable launch failed. That proof covers source typography bundles,
not live pixel sensitivity, geometry, scrolling or input. No new plant was made
or checked by this reviewer.

Cost estimate for this review:18 rendered frames compared as a set, approximately
27 project text files opened and8 focused source/checklist queries, plus3 skill
documents. No whole-project checker-cost total or complete visual sign-off is
inferred from these scoped counts.

## Focused correction recheck

The same independent reviewer checked the parent's two presentation fixes after
the implementation/review boundary. Both P3 findings above are resolved in source:

- The collision caption now says “ID already in current vault,” matching the
  all-record ID set used by `local_global_id_collision`; merge behavior is unchanged.
- Trash caches the exact account lookup, renders the shared identity with its ID,
  and omits the generic ID only for an Account whose identity was rendered.
  Missing-account fallback and every other record kind still render their
  distinct generic IDs. No record or disambiguating identifier is discarded.

Read `parent-review-fixes.md` and both complete diffs. The reported parent
three-width/pagination/dialog/focus/keyboard/metadata regression passed1 case,
210filtered,13.85s; that result is attributed, not independently repeated here.
The source change raised no new concern requiring another invocation of the same
case. Independent `rtk git diff --check` passed after this report update.

Scoped corrections approved, with no remaining finding requiring changes in
this review. No new native screenshot or full-suite result is claimed. The
earlier18 frames remain evidence of the preceding UI state, not rendered proof
of the two final caption changes. Broader Task21/environment limitations above
remain open. Recheck cost:2 source diffs and1 evidence report, no new searches or
rendered frames.
