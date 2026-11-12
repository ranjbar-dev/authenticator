# Task 14: Backup Codes tab and account selection

Status: todo
Priority: Required for complete v1
Dependencies: 04, 05, 10–13
Requirements: FR-3.1; PRD §§5.3, 7

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Implement the Backup Codes tab listing real sets by associated account without exposing code contents.
- [ ] Create an empty state with concise explanation, Add Backup Codes and local learn-more help.
- [ ] Add the searchable/sortable account-selection screen, excluding trashed accounts and making existing sets clear. Route to the editor/detail actions.
- [ ] Compute available/used counts from real vault state and refresh after changes/import/restore/lock.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Shared tab holder and heading preserve the three-section shell. — PRD/reference structure; shared implementation from task 04.
- [ ] Introduction explains recovery codes concisely near the primary action. — PRD/reference structure; shared implementation from task 04.
- [ ] Add Backup Codes is immediately discoverable. — PRD/reference structure; shared implementation from task 04.
- [ ] Set rows identify service/account and available/used counts without showing secrets. — PRD/reference structure; shared implementation from task 04.
- [ ] Empty/no-match/error states provide a useful next action. — PRD/reference structure; shared implementation from task 04.
- [ ] Account-selection toolbar offers labeled live search and sort. — PRD/reference structure; shared implementation from task 04.
- [ ] Selected account and existing-set status are unambiguous before continuing. — PRD/reference structure; shared implementation from task 04.
- [ ] Help, focus, active icons, theme and responsive layout follow shared components; lock hides data. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] Empty, populated, no-match and locked states render correctly with no raw codes exposed.
- [ ] Search/sort selects the intended real account, including similar service names/usernames.
- [ ] Counts and associations remain correct after restart/import and deletion/restoration.
- [ ] Pointer/keyboard/tab navigation and visual checks pass.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

