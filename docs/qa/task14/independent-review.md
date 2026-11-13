# Task14 independent review

13 November 2026. Distinct read-only reviewer `review_14`; neither the author nor
the calibration agent. Compared current source against base
`36216e67b206bc2451d62ef3e14242db22e3bc29`. Source/tests/status/PRD/index were not
edited. This report is the only repository write by the reviewer.

## Verdict and material finding

**Changes requested within the Task14 list/selector/handoff source scope.** One
account-identity ambiguity remains. This verdict does not require implementing
Task15/16 bodies and does not imply native or full Task14 acceptance.

**P2 — Display-identical accounts bypass identity disambiguation.**
`src/ui/backup_codes.rs:89`, `:223`, and `:331`.

Reproduction with accepted, validated domain input:

1. Add two active accounts with service `Public collision`, no website, and no
   backup set. One has identifier `None`; the other has the literal identifier
   `Some("No account identifier")`.
2. Open Add Backup Codes. Both cards display the same service, the same account
   text `No account identifier`, and the same `No backup-code set yet` status.
3. Neither card displays its full account ID. Select either account: its selected
   summary and editor handoff also lack an ID that distinguishes it from the
   other account.

`identity` renders the missing value using that literal text, but `Projection`'s
ambiguity map compares the original `Option<&str>`. The handoff repeats the same
original-value comparison. Thus the two different stored identities do not count
as ambiguous despite their identical rendered identities. A disposable external
probe against the built library confirmed two distinct accepted account IDs,
zero `Account ID:` nodes in the selector and both selected summaries, and routes
to the two different editor IDs. This violates checklist position 7's requirement
that the selected account be unambiguous before continuing; later code entry can
be associated with the wrong intended account.

Use a consistent displayed-identity disambiguation policy in the rows, selected
summary and handoff, or make the missing identifier distinguishable from a
literal identifier. Add a focused regression for the missing-versus-literal
collision. Existing exact service/identifier duplicates already receive full
UUIDs; retain that behavior.

No other material introduced source defect was found in the inspected scope.

## Checklist and branch coverage

Basis: all eight actual Task14 positions and the refreshed implementation
checklist in `implementation-evidence.md`, PRD sections 5.3/FR3/7, and the actual
Codes list as the sole prior production account-list reference. No majority
convention or original-reference pixel fidelity is inferred. No root DESIGN.md
was found. `shell.rs`/`mod.rs` are integration/wrappers rather than separate pages;
test/example/state files are not list-family pages.

| Position | Evidence examined / limits |
| --- | --- |
| 1. Shared holder/heading/three sections | Current ROOT scroll holder, drag Never, shared title/tabs; integration diff against Codes and Settings preservation. |
| 2. Recovery introduction | List branch body adjacent to the primary action; local service-issued-code explanation and help copy. |
| 3. Add Backup Codes | Empty/populated list action and real `Select` transition; Context keyboard activation at normal viewports in the focused test. |
| 4. Associated set rows/counts/privacy | Projection/filter/count source and active parent/set/code deletion states; active/used counts, exact set IDs, zero-count sets, concealed handoffs and locked AX trees. No code-value reader is added. |
| 5. Empty/no-match/error actions | Empty vault, active accounts without sets, no matches, trashed sets, manual-account fallback; overlength/control-character guard and shared field feedback/clear action. External overlength probe observed no selectable account rows. Storage-error recovery remains the shared shell path. |
| 6. Labeled live search/sort | Shared field/choice and six-sort helper; Unicode folding, UUID ties, missing-identifier-last, independent Codes preferences, query reset and paging. Native dropdown operation remains unevaluated. |
| 7. Selected account/status | Exact-ID selection, current-ID route validation, no-set/editor and active/trashed-set/detail routes, Back/from-selection distinction and selected summary. The P2 finding above is the deviation. |
| 8. Help/focus/theme/responsive/Lock | Shared dialog/button/card/field/choice/icon/focus mechanisms; help and Lock branches; normal Context 340x780, 600x800, 960x900 keyboard reach checks; separate tall AX identity/wrapping checks. Actual pixels, native input, Narrator and DPI remain unevaluated. |

Inspected current route families together: Codes and Backup Codes source reuse the
shared holder/heading/card/field/button/theme mechanisms. Backup selector and its
two metadata handoffs extend that family with intentional account selection and
capability-limit copy. No additional list-family deviation was identified. One
Backup screen family needs the identity correction; all native visual judgments
for it are parked for the reason below.

Shared values resolve through `components.rs` and the single applied dark theme:
TITLE 24/32 semibold, SECTION 18/24 semibold, BODY/LABEL 14/20 regular/semibold,
CAPTION 13/20 regular, bundled Noto, shared four-point spacing and 36-point
controls. No page-private typography, color, SVG opacity, or CSS layer is added.
Opaque recovery tiles use BLUE/ON_BLUE. WCAG luminance arithmetic on the actual
source pairs gives TEXT/BACKGROUND 16.73:1, TEXT/SURFACE 14.32:1,
TEXT/FIELD 15.53:1, TEXT/HOVER 11.41:1, MUTED/BACKGROUND 9.49:1,
MUTED/SURFACE 8.12:1, ON_BLUE/BLUE 5.85:1 and ERROR/BACKGROUND 10.69:1.
FOCUS against background/surface/field/hover is 9.49/8.12/8.80/6.47:1;
BORDER/SURFACE is 3.81:1. These are source-pair calculations, not raster tests.

Accessibility subjects remain separate: shared focus border and scroll-to-focus
source was inspected; synthetic keyboard reach passed; labels/descriptions and
radio selected semantics were inspected; icon-only shell controls use shared
names and recovery tiles are decorative beside text; declared control size is
shared 36 points; headings use shared Heading roles and level values. Native
screen-reader announcements, actual focus appearance, hit areas and operating
system keyboard/pointer behavior were not evaluated. This is not accessibility
conformance; other WCAG criteria were not checked.

## Independent checks

All shell invocations began with `rtk`; Cargo commands ran sequentially.

| Command/check | Result |
| --- | --- |
| `rtk cargo test --locked --lib backup_codes_tests` | 7 passed, 131 filtered, 32.98s. Independently read the assertions and fixtures, including real encrypted reopen/export/approved import and failed changed-source apply, Trash graph restoration, worker refresh with unchanged aggregate time, normal synthetic keyboard reach, tall AX privacy/wrapping and pagination identities. |
| `rtk cargo test --locked --lib six_sorts_unicode_query_folders_ties_and_trash` | 1 passed, 137 filtered, 0.00s; existing Codes ordering regression. The extraction itself preserves the former implementation apart from parameters. |
| `rtk cargo test --locked --lib route_searches_stay_separate_and_corrected_input_clears_only_query_feedback` | 1 passed, 137 filtered, 10.58s; existing query separation/correction regression. |
| `rtk cargo test --locked --lib actual_text_and_essential_graphic_pairs_meet_project_thresholds -- --nocapture` | 1 passed, 137 filtered, 0.00s. Pair ratios above were also calculated directly from source colors. |
| `rtk git diff --check` | Passed. |
| Disposable external Rust/Context probe linked to current `target/debug/libauthenticator.rlib` | Passed after probe-only harness correction; confirmed the P2 collision, distinct editor IDs, and overlength-query row suppression. No vault, credentials, clipboard, network or native window was used. |

The probe's first link lacked Cargo's native Windows-library search path; adding
the existing registry library path fixed the probe link. Its first assertion
counted duplicated AX text representations as if each row produced exactly one
text node; the probe was corrected to test the actual identity collision and
absence of UUID nodes. Neither issue was an application failure. Probe sources,
binary and linker leftovers were removed from the verified owned external temp
directory after checking.

The author's full test/lint/build totals were read as evidence, not independently
reproduced or used as acceptance. No standalone all-target build was run and no
unrelated executable was stopped or relinked.

## Open boundaries and unevaluated evidence

The accepted stage boundary provides real list/selector and typed metadata
handoffs. Text/file/review/save belongs to Task15; reveal/copy/used/edit/delete
belongs to Task16. Their explicit unavailable text is honest and is not this
review's source finding. Task18's restoration GUI is also unimplemented; tested
validated domain restoration is not GUI restoration acceptance. Task14 remains
in progress while downstream/native acceptance remains open.

No new native launch was attempted: the author's two documented normal owned
preview launches already failed with `GetCursorPos` Access denied
`0x80070005`, followed by empty owned window/process/temp inventories. The cause
remains unknown. No actual Task14 screenshots exist. Pixel appearance, normal
viewport overflow/overlap/scroll/clipping, actual pointer hit-testing, physical
keyboard/tab/dropdown actions, DPI, Narrator and complete GUI import-refresh are
unevaluated. Tall AX trees and synthetic Context input do not close these gates.
Original-reference imagery is unavailable per `docs/reference-ui.md`.

Existing ROOT source calibration is recorded in `docs/qa/task04/calibration.md`:
the dispatcher used detached copy
`C:/Users/root/AppData/Local/Temp/authenticator-ui-cal-4cg1yjfn/checkout`, planted
TITLE semibold600 to regular400, and a separate checker detected it. The copy was
removed; its executable did not launch. That proves source sensitivity only,
not native/pixel/interaction detection. Task07's separate native-selector/font
plant is not ROOT visual proof. No new calibration or child agent was started.

Earlier Task05/07 physical OS/DPI, Task08 disconnected-machine, Task10 actual
Copy Changed ownership, and Task12/13 native picker/input/pixels/GUI restore gates
remain unchanged. The pre-existing Codes membership-own-Trash filter issue is
preserved by extraction and belongs to the recorded Task17/20 follow-up, not an
introduced Task14 regression.

Cost estimate: 2 account-list family members compared in source, with Backup's
selector, two metadata handoffs and local help also inspected; 0 native rendered
pages/screenshots. Approximately 18 project files opened and 8 grouped searches,
plus skill documentation and one disposable external probe. Counts are estimates;
no other checker was included in this review's cost.

## Focused independent re-review after parent correction

13 November 2026. **Approved within the Task14 list/selector/typed metadata
handoff source scope.** This supersedes the earlier source request for changes;
the P2 finding is resolved. No remaining material introduced source defect was
found in this focused re-review. Full Task14 completion is still unapproved.

Read `parent-fixes.md`, the corrected identity/ambiguity source and the new
regression. A single borrowed `display_identity` helper now supplies the service
and rendered identifier in all three roles: identity rendering, projection
ambiguity indexing, and handoff duplicate detection. `None` and the accepted
literal `Some("No account identifier")` therefore receive full UUIDs consistently
when their displayed identities collide. The helper adds no code-value read,
record clone, persistence change, action, or change to search/sort/route ID
binding. Existing exact identity duplicates retain their disambiguation.

The new regression uses four real validated accounts in an isolated encrypted
fixture, comprising missing/literal pairs with and without sets. It verifies
UUIDs on both list rows, all four selector rows, an additional occurrence of the
correct UUID in each selected summary, and actual distinct account/set route IDs.
Each handoff shows only its target UUID and passes the secret/clipboard-command
checks. List and handoff AX checks cover 340/600/960 widths in tall synthetic
viewports; selector/summary activation is at 600. This is useful identity and
privacy coverage, not normal-viewport visual or native-input proof.

Independent focused checks:

| Command | Result |
| --- | --- |
| `rtk cargo test --locked --lib backup_display_placeholder_collision` | 1 passed, 138 filtered, 7.29s. |
| `rtk cargo test --locked --lib backup_root_empty_populated_no_match_selection_back_help_and_lock` | 1 passed, 138 filtered, 21.36s; preserves original exact-duplicate, route/help/Lock coverage. |
| `rtk git diff --check` | Passed. |

No source/test/status changes, probe or native attempt was made during re-review.
The earlier report's eight-position comparison and all unevaluated visual,
native-input, DPI, Narrator, original-reference and GUI import/restore criteria
remain unchanged. Task15/16 bodies and prior Task05/07/08/10/12/13 manual gates
remain open; no source-only result closes them.

Additional re-review cost: 3 project files opened and 1 grouped search; 0 native
rendered pages/screenshots. Cumulative review cost is approximately 19 distinct
project files and 9 grouped searches, across the same 2 account-list family
members plus selector/handoffs/help. Counts are estimates.
