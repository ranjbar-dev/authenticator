# Task 21: Full offline workflow, accessibility and visual QA

Status: todo
Priority: Required for complete v1
Dependencies: 01–20
Requirements: PRD §§2, 6–7, 11; all UI checklists

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Run complete disposable-data journeys from clean install: vault setup → screenshot import → current-code verification → copy/edit/search/sort/views → backup codes → folders → trash/restore → encrypted export → clean-vault merge/replace restore.
- [ ] Use real pointer/keyboard interactions for capture drag, dropdowns, buttons, multi-selection, dialogs and clipboard; synthetic click-only checks do not suffice.
- [ ] Inspect actual component screenshots for every page at narrow, intermediate and desktop windows and 100/150/200% scale; use available reference evidence and record unavailable reference comparisons as unverified.
- [ ] Check empty/loading/no-results/open/error/focus/locked states, active-icon contrast, foreground/background pairs, clipping and logical hierarchy. Test Narrator/accessibility labels, roles, tab order and announcements without repeatedly reading OTP values.
- [ ] Use a reviewer who did not implement the screens, following each task's checklist and ui-consistency:verifying if available. Repair findings and rerun affected checks.
- [ ] Store only sanitized/disposable evidence and a coverage report under docs/qa/. Complete all PRD acceptance criteria, not just a successful launch or test exit code.

## Acceptance and verification

- [ ] Every FR and success criterion maps to passing reproducible tests or explicit honest evidence; no untested manual check is labeled passed.
- [ ] Screenshot-to-import works offline across available monitor/scaling combinations and failure/cancel/lock paths.
- [ ] All pages are readable and operable with pointer/keyboard at tested sizes, and reference deviations are fixed or documented accurately.
- [ ] Independent reviewer finds no unresolved material functional, accessibility or visual defect.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

