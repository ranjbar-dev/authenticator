# Task14 implementation evidence

Implementation date: 13 November 2026. Ready for distinct review; this document
does not mark Task14 accepted or complete. Parent owns task/index/coverage status,
integration, independent UI/security review and commits. Earlier task native and
offline gates remain open.

## Scope and source

- `src/ui/backup_codes.rs`: real associated-account set list; concealed available/
  used counts; empty/no-match/error recovery; account selector; exact-ID selected
  summary; six sorts; pagination; safe typed editor/detail handoffs.
- `src/ui/codes.rs`: extracted shared all-account ordering entry point. Existing
  Codes ordering still uses its original encrypted query/sort/folder preferences.
- `src/ui/shell.rs`, `src/ui/mod.rs`: route integration, local help, import/worker/
  Lock/tab invalidation, Escape/Back and existing real manual-account fallback.
- `src/ui/backup_codes_tests.rs`: seven new regressions with disposable real
  encrypted records, model derivation and synthetic ROOT/AccessKit events.
- `examples/backup_codes.rs`: argument-free own preview, fresh TempDir and injected
  VaultStore, public fixture only, F6/F7/F8 widths and F10 native framebuffer
  capture with code bounds redacted before encoding/writing.
- `docs/ui-foundation.md`, `docs/architecture.md`: current shared components,
  count/cache/route policy and downstream boundaries.

No schema, dependency, encrypted backend, network, clipboard or save-intent change.
Production UI has no seeded business records. No default, real, reference or
unrelated preview vault/account/backup/clipboard was opened or used. No PRD/task/
index/coverage edits, commit, push or staging operation was performed by this child.
The task/index changes already present at dispatch belong to the parent.

## Design basis and pre-code workflow

Sized as a whole UI phase before implementation. Applied `ui-consistency` finding-
patterns, values, decisions, planning and implementing in the existing parent
process. Read the actual Task14 checklist, PRD §§5.3/FR3/7/12, dependency source,
shared UI foundation, reference inventory and the user's reusable preference file.
The session plan was shown before UI code: list first, selector and safe typed
handoffs next, meaningful tests/evidence, then the parent's separate review.
The task brief already authorized implementation; no approval stop was introduced.

The only existing production account-list member was Codes in `shell.rs` (one
member from the three route candidates; Backup Codes was a placeholder, Settings
is a settings form). This establishes a reference, not a majority convention.
Reference roles come from PRD/reference inventory; mechanisms come from the shared
ROOT controls (decision levels 3/4). The source authority remains the vault.
Grouped reach search `rtk rg -l 'c::heading\(|c::labeled_field\(|c::labeled_choice\(|c::button\(|c::card\(|c::status\(' src/ui --glob '*.rs'` found shell,
import_review, export, backup_import and the new backup_codes module. These are
shared-use evidence, not five peer list pages. No invented visual measurements.

The tree/checklist used in-session, refreshed from current source before writing:

| Task14 position | Implementation mechanism / decision |
| --- | --- |
| Shared holder, heading and three tabs | Existing ROOT vertical ScrollArea, drag Never; shared heading level1 and tabs; preserve Codes and Settings (as asked/shared pieces). |
| Concise recovery introduction | Shared BODY near Add; service-issued recovery purpose and local-only context (reference shape/shared text). |
| Add Backup Codes | Shared primary full-width button immediately above search/list; empty and populated states both expose it (as asked). |
| Set rows | Shared card, opaque BLUE/ON_BLUE Recovery icon, section service, body identifier, optional website, real active counts; no values (shared pieces/as asked). |
| Empty/no-match/error | Shared empty state; Add, Clear search, or actual manual-account fallback; bounded-query error beside field; no fabricated success (as asked/shared pieces). |
| Selector search and sort | Shared labeled field and native labeled choice; live Unicode case-fold filter; all six existing labels; independent session sort/query (as asked/shared ordering). |
| Selected account / existing status | Exact-ID selection, selected radio semantics and opaque primary button; full account IDs when service/identifier pair is identical; selected summary above list and explicit Continue to the real existing set or no-set editor (as asked). |
| Help/focus/theme/responsive/Lock | Shared ROOT dialog and controls; current complete Noto typography; wrapping; stable scoped IDs; non-dragging scroll; Lock discards IDs and query and renders no credential nodes (shared pieces/as asked). |

No private font, color, style, card or form primitive was created. Uses existing
TITLE24/32 SemiBold600, SECTION18/24 SemiBold600, BODY/LABEL14/20 Regular400/
SemiBold600, CAPTION13/20 Regular400; four-point spacing, shared gaps/padding/
control height/content width and existing foreground/surface pairs. PAGE_SIZE20
is a data rendering bound, not a visual measurement. New copy is plain English
and states the current build's available capabilities. No excluded profile,
sign-in, cloud, language, extension, encryption toggle or swipe-only action.

This remains the ROOT screen kind calibrated in `docs/qa/task04/calibration.md`:
distinct checker detected TITLE600→400 in the detached copy. That was source
sensitivity, not a pixel/interaction proof; its copied executable did not launch.
Task07's native-selector calibration covers a different screen kind and does not
prove these ROOT visuals. No new plant or native visual comparison is claimed.
Original reference screens remain unavailable under `docs/reference-ui.md`.
Estimated research cost: about20 project files opened and about15 grouped source
searches; skill/reference documentation is additional. Counts were not used as
majority claims. Parent will dispatch `ui-consistency:verifying` separately with
this checklist and the native limitations.

## Data, navigation and privacy policy

The list requires active account AND active set. Available/used totals count only
active individual codes and inspect only `state()`/`used_at()`. No code value is
read by this module. A set with all codes used or trashed remains a real set with
corresponding zero counts. Parent-account Trash hides its set. A trashed set is
hidden from the list but clearly identified in account selection, retaining its
exact set ID: the current domain forbids adding a second set even when the first
is trashed. Trashed accounts are absent from selection.

Unicode case folding, six sorts, UUID ties and empty-identifier-last semantics
reuse the actual Codes ordering implementation with All-account scope. Session
query/sort/page/selection never mutate Codes query/view/folder/sort preferences or
the encrypted schema. Filtering resets pagination and selection. Twenty rows per
page bound widgets for the 10,000-record limit; Previous/Next identities persist
across enabled/disabled transitions. Full equal service/identifier pairs show
full UUIDs, not potentially colliding shortened IDs.

The projection contains IDs/counts/ambiguity IDs, not cloned records or names.
Every worker result invalidates it; Lock/tab departure reset it. Successful
adopted backup import clears it plus selection and query; failed import preserves
the previous selection/query. Route rendering revalidates account/set IDs against
the borrowed current vault. A timestamp is not used as cache revision because
clamped mutations can share one aggregate timestamp. Existing sole-worker,
cancellation/admission and Lock behavior remain unchanged; this adds no complete
memory-erasure or post-admission rollback promise.

Task15/16 handoff boundary: `Screen::Editor(AccountId)` shows the selected account
and honest unavailable text/file/save capability. `Screen::Detail { account, set,
from_selection }` shows concealed metadata and honest unavailable reveal/copy/
used/edit/delete capability, or the existing-in-Trash requirement. Back retains
the exact selection; Back to Backup Codes returns to list. There are no pretend
Save or secret buttons. Tasks15/16 must add those bodies with fresh-state checks,
validation and the existing worker/guarded clipboard API. They are not completed
by Task14. Real create-time recovery sets from Task09 already populate this list.
Trash-screen restoration remains Task18; authenticated fixture restoration tests
here prove derived behavior, not a completed restoration GUI.

## Verification

All shell commands began with `rtk`. Cargo commands ran sequentially; no unrelated
process was stopped or artifact relinked through a standalone all-target build.

| Command | Result |
| --- | --- |
| `rtk cargo check --locked --all-targets --all-features` | Initial source compiled; unused obsolete workflow method then removed. |
| `rtk cargo test --locked --lib route_searches_stay_separate_and_corrected_input_clears_only_query_feedback` | 1 passed, 130 filtered; existing Codes/Backup query regression preserved. |
| `rtk cargo test --locked --lib backup_` | 19 passed, 117 filtered before the final two new tests; includes actual encrypted round-trip/import and same-timestamp mutation cases. |
| `rtk cargo test --locked --lib backup_root_empty_populated_no_match_selection_back_help_and_lock` | 1 passed, 133 filtered after correcting test helper order. |
| `rtk cargo test --locked --lib backup_normal_viewports_keyboard_live_search_intended_account_and_wrapped_identity` | 1 passed, 137 filtered after correcting AX test-bound lookup. |
| `rtk cargo test --locked --lib backup_account_pagination_retains_control_ids_and_search_resets_page` | 1 passed, 137 filtered. |
| `rtk cargo test --locked --all-targets --all-features` | Final post-lint run: 150 passed, 4 ignored helper entry points, 137 filtered, 17 suites, 101.57s. The first run also passed150 in103.96s; rerun followed the two lint cleanups. |
| `rtk cargo clippy --locked --all-targets --all-features -- -D warnings` | Passed, no issues after collapsing a nested if and replacing deprecated test Id::new with Id::unique. |
| `rtk cargo fmt --all -- --check` | Passed. |
| `rtk git diff --check` | Passed. |
| `rtk cargo build --locked --bin Authenticator --lib --example backup_codes` | Passed; explicit production/lib/own preview build, no warnings. |

The four ignored tests remain isolated subprocess helper entry points in storage,
backup and clipboard tests. Their parent tests exercise them; none was newly
ignored. All 143 prior tests remain, plus seven Task14 regressions.

The initial targeted selection test failed because its helper treated AX's node
vector as screen order; it now sorts observed button bounds by y before choosing
an occurrence. The initial wrapping test looked only at a node label while the
body's native AX text can be a value; it now accepts either returned label/value
and measures actual bounds. These were test-harness corrections, not claimed
native input fixes. Clippy's two findings were corrected before the final build.

Coverage: six sorts/casefold/ties/missing IDs, independent preferences, active
account/set/code Trash policy and restored validated graph, exact selected and
handoff IDs, similar service/usernames and identical labels, empty/populated/no-
match/help/Back/Lock, no secret text nodes or CopyText command, 23-account paging,
live TextEdit input, actual encrypted export/import/reopen (including used/code/
set/parent Trash), changed source apply preserving selection/query, adopted
success clearing stale IDs, and real save-worker deletion with unchanged aggregate
timestamp invalidating counts. Native image/QR/clipboard behavior was not broadened.

Normal synthetic viewports are340x780,600x800,960x900: Tab/Space reach Add, Select,
Continue and the intended account, with focused control bounds scrolled inside
the viewport. Separate 16,000-point-tall ROOT passes inspect complete metadata and
wrapped long service/identifier bounds; those are not normal-viewport evidence.
Context pointer events use observed AX bounds and never count as physical/native
pointer proof. No image was manufactured from Context output.

## Native attempts and remaining gates

Read the bundled computer-use SKILL.md, guidance, API and confirmations; initialized
documented `@oai/sky` through `node_repl`. Fresh `list_apps` found no owned Task14
window; only unrelated `vault_lifecycle` was present and was left untouched.

Two normal argument-free `sky.launch_app` calls targeted only
`D:\Projects\github\authenticator\target\debug\examples\backup_codes.exe`.
Each returned exactly: `Error: GetCursorPos failed: Access is denied. (0x80070005)`.
Between calls and afterward fresh `sky.list_windows` returned zero own Task14
windows. Process/TempDir inventory before and after found no `backup_codes`
process or `authenticator-task14-*` directory. Get-Process's no-match exit1 is an
empty inventory, not an observed launched process. Nothing owned required cleanup.
The cause is unknown; no OS-lock assertion is made.

Stopped after the two normal failures. No security/global settings, adapters,
elevation bypass, direct UI Automation/raw input/private protocol or elevated
Start-Process was used. No unrelated preview/window/process/temp path was touched.
No actual screenshots exist for Task14, so there is no PNG completion/IEND or
pixel result to claim. The helper's F10 path is available for later successful
native testing and redacts code bounds before disk.

Open: real pointer and keyboard/tab/dropdown/scroll actions, 340/600/960 visual
inspection of populated/empty/no-match/selection/Trash-status/handoff/locked/help/
focus states, physical DPI/Narrator, original-reference pixel fidelity and full
Tasks12→13 GUI import refresh. The existing Task10 actual Copy Changed issue is
unchanged and is not asserted fixed. Source/Context passes are not a native pass.
Parent's distinct review must report its own result; this author does not approve
the work or close earlier manual gates.
