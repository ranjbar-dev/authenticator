# Task13 independent UI, security and data-loss review

13 November 2026. Distinct read-only `review_13`, neither implementation author
nor calibration checker. **Changes requested: two P2 source findings.** No P1
authentication, partial-write or late-unlock defect was established. Source
approval is withheld pending the focused corrections below. Native acceptance
also remains open independently; this report does not complete Task13.

## Findings

### P2 — offer only folder decisions the engine can accept

`src/ui/backup_import.rs:449` always includes **Keep both · separate folder copy**
in every incoming folder's selector. `src/backup/merge.rs:372` rejects every
explicit folder decision when that incoming folder has no current same-ID/name
folder and its ID is not occupied by another current record. Consequently a
normal backup with a new folder, imported into a fresh vault, offers a choice
that fails **Review exact import changes** with the generic InvalidDecision error.
The default choice succeeds. No data is lost, but a valid-looking selection
blocks the intended restore and does not identify which folder must be reset.

Reproduction: export a generated vault containing one distinctly named folder;
preview into an independently keyed empty vault; choose Keep both for that folder;
build the plan. The production form source unconditionally offers it; a disposable
external Rust probe using the actual production form module and public engine
confirmed the unique-folder selector exists, default planning succeeds, and the
explicit `(0, KeepBoth)` decision returns `BackupError::InvalidDecision`.
This is engine/framework evidence, not a native pointer observation.

Prefer engine-authoritative allowed decisions exposed with preview data, then
offer only those choices. In particular, preserve valid KeepBoth for a pure
cross-type ID collision even without a same-name/current-folder match. Alternatively
explicitly define and test redundant unique-folder KeepBoth as accepted engine
behavior. Add a regression for a genuinely new folder: the current KeepBoth test
uses an existing copy of the entire source vault and therefore covers a collision.

### P2 — disclose resulting account–folder links and membership Trash state

`src/ui/backup_import.rs:557` renders resulting accounts through `account`, which
shows credential metadata and account state but no folder links. Final review
continues with recovery records and folder resolutions, without ever reading
`memberships()`. Memberships appear only as an aggregate count at lines256–257.
`src/backup/merge.rs:474` nevertheless imports/remaps their account and folder
references and preserves their individual record/Trash state.

Reproduction: back up two accounts and two folders, with a membership linking one
account to one folder; restore/merge and review the candidate. The review cannot
identify the link's endpoints or membership Trash/dates. Two backups with the same
visible accounts/folders/counts but different links cannot be distinguished there.
A reused or copied folder resolution says that incoming memberships map there,
but never names the affected accounts. This is a source-confirmed disclosure gap
in Task13's exact affected-data/Trash review, not established graph corruption.
The full-codec round-trip test proves persistence, not informed approval.

Add bounded, paginated secret-free candidate membership review (or equivalent
per-account folder associations), including endpoint identities and individual
record state/dates. Read from the actual plan candidate so KeepBoth remaps and
folder reuse are accurately represented; include active/Trash cases in the UI
assertions. No seeds, OTPs or recovery values are needed.

## Reviewed boundary and security result

Read the task/checklist and implementation evidence first, then the new import
form/tests/example and changed shell/state/engine/picker paths, with actual export,
shared components/theme, format, domain and storage dependencies. The small engine
diff was reviewed against Task12's accepted source and the prior independent
review; prior status labels were not treated as acceptance proof.

Preview owns immutable incoming/local graphs plus exact source and destination
ciphertext revisions. Borrowed planning clones only the opaque source binding
in addition to constructing the candidate; it does not clone the store or DEK.
Apply consumes the concrete plan, rechecks both revisions and uses the existing
atomic local transaction. Changed-file/local-revision and injected-write failures
preserved ciphertext in the exercised cases.

Preview/KDF/file hashing, graph construction/validation and Apply execute on the
sole repository worker. Typed Preview/Plan/Applied results, worker refusal,
generation/lifecycle/monotonic deadline checks and publication cleanup prevent
stale adoption. ROOT Lock drops the form/path, preview/candidate, phrase, dialogs,
widget history and prior data projection before the held admitted rename resumes.
The test verifies locked AX before release, then complete write and no late UI
success/unlock. This follows the documented admitted-write exception; it is not a
claim that cooperative cancellation can undo an already admitted atomic write.

Pending display intent is flushed before preview captures the destination
snapshot, including failure/retry. Imported display and lock/clipboard settings
are separate deliberate choices with local defaults; startup and local identity
remain local. The approved candidate, rather than subsequent editable choices,
drives Apply. Success clears/reloads cached data and queries from the real store.
Native recognition is by bytes; AUTHLOC1/reference/future inputs are honestly
unsupported, with no guessed parser or network operation. The Open adapter uses
FILEMUSTEXIST plus the existing no-recent/no-directory-change/path safety flags;
file selection grants no Apply consent. Duplicate workers and picker expiry/drop
were checked without introducing a second repository owner.

The reviewed tests have meaningful oracles: Task12's actual form/worker exports,
Task13's form/worker replaces into a fresh independently keyed local vault,
normalized full codec graph equality, 24 OTP parameter combinations and the
independent RFC6238 value at t=59. Other tests cover default Merge/KeepBoth,
Back/rebuild, wrong phrase/tamper/future/local/reference/oversize, stale source and
local revision, fault/retry, preference save failure, early Lock, lifecycle/idle
refusal and the admitted-write race. Some helpers set choices/call private action
methods directly; those assertions do not establish dropdown or real OS input.

## UI consistency and accessibility scope

Applied `ui-consistency:verifying` and its called values/accessibility/contrast
guidance. The checklist's initial source-absent snapshot is historical: current
shared ROOT/components/theme and the single Export form are the actual precedent.
There is no measured majority family to invent. No root DESIGN.md, repository
AGENTS.md or CSS file was found by the scoped discovery. The import module uses
the shared heading/body/caption, field, selection, card, button, progress, status
and ROOT dialog roles; no private font/color layer was introduced.

All eight checklist positions were opened and compared top-to-bottom, including
file/no-file/validation, picking, saving preferences, Preview/Plan/Apply busy,
empty/nonempty conflicts and folders, Merge/Replace, separate preference choices,
final accounts/recovery/folder pagination, replacement keep/confirm, errors/retry,
Back, success and Lock branches. The two deviations above concern selectors and
affected-data disclosure. The final same-kind comparison covered Import and its
one Export precedent, including their shared shell/modal/feedback roles. Dispatch,
worker, barrel and example wrappers are not separate product pages.

Source/framework observations and their limits are separate:

- **Contrast:** the one dark theme resolves through shared components. The actual
  luminance test passed: body16.73, card14.32, field15.53, hover11.41,
  secondary-on-card8.12, secondary-on-hover6.47, primary5.85, error8.69,
  success10.46, accent5.01:1; focus against the four holders9.49/8.12/8.80/6.47:1.
  These meet the project's4.5 text/3 graphic targets. Actual rasterization,
  composite rendered appearance and native dialog colors remain unevaluated.
- **Focus:** shared visible focus/scroll and safe initial replacement focus are
  present. Framework tests exercise return/cancel/help/Lock behavior; actual
  Windows focus visibility, restoration and Narrator behavior remain unevaluated.
- **Keyboard reach/order:** normal340×780,600×800,960×900 synthetic Tab/scroll
  tests pass for primary actions. They do not prove every native dropdown,
  picker, short-window modal, pointer hit area or OS keyboard sequence.
- **Labels:** the concealed field and selectors use shared label associations;
  masking/AX tests pass. Native screen-reader label/error announcements remain
  unevaluated. No raw seed/recovery value is used in import summaries.
- **Text alternatives:** named shared Help/Lock controls are reused. Actual native
  assistive-technology interpretation remains unevaluated.
- **Targets:** the shared36-point wrapped button/field roles are used; selectors
  inherit the existing component behavior. Actual DPI target sizes/overlap,
  overflow, clipping, scrolling and pointer capture remain unevaluated.
- **Structure:** ROOT title, heading levels, dialog semantics and status roles
  reuse the shared helpers. Runtime reading order and complete language/region
  exposure remain unevaluated; no complete accessibility conformance is claimed.

The dispatcher supplied existing source sensitivity proof in
`docs/qa/task04/calibration.md` (detached temporary copy, TITLE600→400 detected) and
`docs/qa/task07/calibration.md` (native copied Cancel600→400 detected). Their exact
copy paths and cleanup are recorded there. These plants establish source bundle
detection; they do not establish pixel/live contrast, layout or interaction
sensitivity. No new calibration or child agent was created by this reviewer.

## Commands and native limits

All shell commands began with RTK. Cargo operations were sequential; no unrelated
lifecycle preview was stopped or touched and no all-target standalone build ran.

| Independent command/check | Result |
|---|---|
| `rtk cargo test --locked --lib backup_import_` |9 passed,0 failed,40.10s |
| `rtk cargo test --locked --lib backup_import::tests::expired_picker_never_publishes_selected_metadata_and_drop_never_waits` |1 passed,0.01s |
| `rtk proxy cargo test --locked --lib ui::theme::tests::actual_text_and_essential_graphic_pairs_meet_project_thresholds -- --nocapture` |1 passed; ratios above |
| `rtk cargo run --offline --manifest-path D:/Projects/github/.authenticator-review13-folder-20261113/Cargo.toml --target-dir D:/Projects/github/authenticator/target` |Final external probe passed; actual engine default succeeds/KeepBoth rejects for unique folder |
| `rtk git diff --check` |Passed |

The owned external probe depended on production source; it did not mutate
production or tests. Its first harness attempts hit an unapplied TexturesDelta
assertion, initial AX setup, and an unproven synthetic popup click. Those were
probe limitations, not app findings. The final probe deliberately establishes
only engine rejection plus selector existence; option availability is established
from the opened production source. Its fixture is generated/public only. Probe
compilation warnings concern unused included module items/deprecated probe ID,
not the application's strict-Clippy result. No source-copy cache substitution was
used; the repository test target was rerun after the probe compilation.

Read the computer-use skill/guidance/API/confirmations. Through documented
`@oai/sky` in node_repl: fresh exact-title inventory found zero owned windows;
one normal argument-free launch of
`D:/Projects/github/authenticator/target/debug/examples/import_backup.exe`
returned **GetCursorPos failed: Access is denied (0x80070005)**; fresh inventory
again found zero. Exact-path process and `authenticator-task13-*` TempDir checks
were empty. Cause is unknown; no OS-lock inference, bypass, permission change,
alternate input or production/reference launch occurred. No framebuffer was
obtained or persisted.

Therefore actual pointer/keyboard/dropdown/picker interactions, Task12→13 native
interactive restore, screenshot/visual quality at340/600/960, rendered overflow/
overlap/contrast/DPI, Narrator, hardware lock/suspend, offline network-disabled
operation and second-user/device recovery remain OPEN. The24000-height AX cases
test text/masking geometry only. Normal-viewport framework tests improve
reachability evidence but do not close these native gates. Existing Task12 and
earlier hardware/manual gates remain open, rather than new source defects here.

The existing shared secret editor accepts unbounded edit-time character counts;
Import checks12–1024 UTF-8 bytes before KDF. An oversized paste may occupy memory
until correction/exit. This documented existing global behavior was not introduced
by Task13; immediate complete memory erasure is not claimed.

Cleanup removed only the verified owned external probe root and specifically
named probe build artifacts. No real/default/reference data or clipboard was
read. Only this report was written inside the repository; parent owns fixes,
status updates and commits. Ready for parent correction and focused re-review.

Cost estimate:2 form pages compared (Import plus Export precedent),0 native
pages/screenshots evaluated, approximately26 distinct project source/document
files opened and13 grouped discovery/search commands; repeat ranged reads are
included in the work, but exact totals were not instrumented. Skill/runtime
documentation reads and one external probe are additional. Other accessibility
criteria were not checked.

## Focused independent re-review after parent corrections

13 November 2026. Same distinct reviewer, read-only except this appended report.
**Both P2 findings are resolved. Source is approved within the reviewed scope;
Task13 native acceptance remains OPEN.** This verdict supersedes the initial
changes-requested source verdict above. No new material finding arose.

Read `parent-fixes.md` and the actual changed engine/form/test paths, then
rechecked consuming Apply and revision binding. `FolderDecisions` is precomputed
from the immutable current/incoming snapshots on the preview worker. The engine
uses the same `for_collision`/`permits` rule when validating submitted choices;
there is no separate UI approximation. Unique folders show explanatory default
behavior instead of an invalid selector. Same-ID and same-name folders retain
both supported choices; pure cross-type collisions retain KeepBoth without
offering unsupported KeepLocal. The engine test constructs each case and compares
both accepted and rejected explicit choices against real candidate construction,
plus default success. This corrects P2#1 without relaxing the write policy.

Final review now reads the actual plan candidate's memberships, five per page,
showing account service/identifier/ID, folder identity/metadata, membership ID,
creation/update dates and its individual active/Trash state. It therefore reflects
the engine's resolved references rather than the pre-remap incoming graph. No
seed, current OTP or recovery value is introduced. The generated seven-link
test verifies exact candidate endpoint IDs, active and trashed records, first
five/last two pages and return navigation at the documented synthetic widths.
This corrects P2#2. Named pagination buttons preserve their individual AX IDs
across pages; unavailable directions remain mounted and disabled while the
collection has multiple pages, avoiding Next/Previous identity reuse.

The new UI region uses the existing shared heading/body/secondary/card/button
components. No new typography, spacing, palette, icon or native dialog layer is
added. Source comparison covered the changed choices and final-review regions
and all affected pagination collections against the same checklist and Export
precedent. Other checklist regions and the original security assessment retain
their earlier scope and limits. The source/local ciphertext binding, immutable
candidate, consuming Apply, explicit Replace confirmation, sole-store ownership,
preferences consent and Lock/cancellation publication boundaries remain intact.

Independent final-source commands:

| Command | Result |
|---|---|
| `rtk cargo test --locked --lib folder_decisions_match_engine` |1 passed,0 failed,0.00s |
| `rtk cargo test --locked --lib backup_import_` |10 passed,0 failed,40.80s |
| `rtk git diff --check` |Passed |

The focused import run includes the new unique-folder/association regression,
normal-viewport primary-action reachability, full export/restore graph and OTP
oracle, stale/fault atomicity, defaults/preferences and Lock/publication tests.
No additional full-project suite or native launch was necessary for this bounded
re-review. No production, test, task/status, PRD or settings edits were made.

These remain framework/source results. Actual pointer/keyboard/dropdown/picker
operation, native Task12→13 interactive restoration, rendered340/600/960 quality,
overflow/overlap/contrast/DPI, Narrator, hardware/offline and second-user/device
gates remain OPEN as recorded above. The prior access-denied native attempt was
not retried; no new screenshot, OS interaction or visual pass is claimed.

Focused cost estimate:6 project file openings (five previously reviewed source/
test files plus the new correction record),1 grouped source search,2 form pages
in the same-kind comparison and0 native pages/screenshots. Combined estimate:
27 distinct project files and14 grouped searches, plus the original skill/runtime
reads and disposable probe. Other accessibility criteria remain unchecked.
