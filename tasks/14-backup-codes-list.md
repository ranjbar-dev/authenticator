# Task 14: Backup Codes tab and account selection

Status: done — numbered UI scope independently reviewed with native state evidence
Priority: Required for complete v1
Dependencies: 04, 05, 10–13
Requirements: FR-3.1; PRD §§5.3, 7

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Implement the Backup Codes tab listing real sets by associated account without exposing code contents.
- [x] Create an empty state with concise explanation, Add Backup Codes and local learn-more help.
- [x] Add the searchable/sortable account-selection screen, excluding trashed accounts and making existing sets clear. Route to the editor/detail actions.
- [x] Compute available/used counts from real vault state and refresh after changes/import/restore/lock.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [x] Shared tab holder and heading preserve the three-section shell. — PRD/reference structure; shared implementation from task 04.
- [x] Introduction explains recovery codes concisely near the primary action. — PRD/reference structure; shared implementation from task 04.
- [x] Add Backup Codes is immediately discoverable. — PRD/reference structure; shared implementation from task 04.
- [x] Set rows identify service/account and available/used counts without showing secrets. — PRD/reference structure; shared implementation from task 04.
- [x] Empty/no-match/error states provide a useful next action. — PRD/reference structure; shared implementation from task 04.
- [x] Account-selection toolbar offers labeled live search and sort. — PRD/reference structure; shared implementation from task 04.
- [x] Selected account and existing-set status are unambiguous before continuing. — PRD/reference structure; shared implementation from task 04.
- [x] Help, focus, active icons, theme and responsive layout follow shared components; lock hides data. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [x] Empty, populated, no-match and locked states render correctly with no raw codes exposed.
- [x] Search/sort selects the intended real account, including similar service names/usernames.
- [x] Counts and associations remain correct after restart/import and deletion/restoration.
- [x] Pointer/keyboard/tab navigation and visual checks pass.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

**Current accepted outcome:** The [same-checker re-review](../docs/qa/task24/state-gaps/independent-review.md) recommends closing this task after combining the original source/security/fault tests, reviewed Task21/23 native recovery and the Task24 focus/state evidence. All listed checkboxes are now reconciled to that evidence. Actual offline/other-user/no-Rust, Windows lifecycle/logon and heard-speech gates remain with their explicitly blocked owners; no global certification follows from this page. Earlier evidence paragraphs below retain their historical launch/source context and are superseded only for the now-tested scope.


Current native21 used the actual empty list/account selector, created a real set and verified available/used counts after Lock/unlock, account Trash/restore and full-graph backup restoration. [Journey and frames](../docs/qa/task21/remaining-native-workflows.md), [independent review](../docs/qa/task21/remaining-native-independent-review.md). This supplies previously unavailable native positive-path evidence; all-state/large-list/environment checks are not newly certified.

Corrected list/selector/typed metadata handoff source approved by distinct review_14
and focused re-review. Account selection and exact-ID routes are implemented;
Tasks15/16 now supply actual editor/Edit/Delete Set and detail Reveal/Copy/used/Undo
actions, with reviewed identity assertions and Cancel/Discard ownership. Source boxes above
do not certify native rendering or the full task.

- [Author checklist, policy and exact checks](../docs/qa/task14/implementation-evidence.md).
- [Independent finding, re-review and limits](../docs/qa/task14/independent-review.md).
- [Parent identity correction and checks](../docs/qa/task14/parent-fixes.md).
- Final151 all-target/all-feature tests passed,4 parent-exercised helper entrypoints,
  17 suites103.62s; strict Clippy, fmt/diff and production/lib/owned-example build pass.
- Independent original7 tests plus ordering/query/contrast and focused collision1
  (7.29s)/route-help-Lock1 (21.36s) passed. P2 missing-vs-literal identifier ambiguity
  fixed with one displayed-identity rule; rows, selection summaries and both handoffs
  now show exact UUIDs when rendered service/identifier pairs collide.
- Real active parent/set/code counts, existing trashed-set identity, Unicode six sorts,
  stable paging, same-timestamp invalidation, encrypted export/reopen/import and
  failed-import selection preservation are tested. No code values are read by the list.
- Task15 editor and16 detail Reveal/Copy/used/Undo/per-code Trash source are reviewed.
  Native routes and18 actual restoration integration stay open.
- Author2 normal Sky launches denied GetCursorPos0x80070005, no owned window/process/
  fixture remained. Actual pointer/keyboard, screenshots, rendered quality, physical
  DPI/Narrator and earlier hardware/offline/Copy/GUI-recovery gates remain OPEN.
  Synthetic normal-viewport keyboard and tall AX checks do not replace them.

Parent integrated14 and reviewed15, then continues16. No real/default/reference
data or unrelated preview changed; commit reference follows in parent checkpoint.

