# Task 17: Folder management and filtered navigation

Status: todo
Priority: Required for complete v1
Dependencies: 10, 14–16
Requirements: FR-4; PRD §§5.1, 7, 9, 12

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Implement folder creation/rename with name, six reference-style colors and searchable/sortable multi-account selection.
- [ ] Use many-to-many memberships; include All accounts and Unfiled views with visible keyboard-operable navigation.
- [ ] Persist membership, selected folder and organization preferences inside encrypted storage. Validate duplicate/empty folder names consistently.
- [ ] Confirm folder removal where useful and explain that removing a folder preserves accounts. Handle trashed/restored members and imported folder conflicts predictably.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Shared header names folder creation/editing with back/cancel. — PRD/reference structure; shared implementation from task 04.
- [ ] Folder-name field has a label and inline validation. — PRD/reference structure; shared implementation from task 04.
- [ ] Six color controls show selected state with a non-color cue. — PRD/reference structure; shared implementation from task 04.
- [ ] Account search/sort appears before selectable membership rows. — PRD/reference structure; shared implementation from task 04.
- [ ] Rows identify service/account and have accessible selected state. — PRD/reference structure; shared implementation from task 04.
- [ ] Save action persists reviewed membership without deleting accounts. — PRD/reference structure; shared implementation from task 04.
- [ ] Folder navigation clearly distinguishes All, Unfiled and named folders. — PRD/reference structure; shared implementation from task 04.
- [ ] Empty/error/delete states, shared tokens, focus and narrow-window layout remain consistent. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] Create/rename/recolor folders, place an account in multiple folders and verify All/Unfiled behavior.
- [ ] Restart and export/import preserve folders/memberships without duplicated records or orphan references.
- [ ] Deleting a folder leaves all accounts and backup codes intact.
- [ ] Pointer clicks, keyboard search/selection and compact layouts pass independent review.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

