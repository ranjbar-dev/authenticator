# Task 09: Manual creation and account editing

Status: todo
Priority: Required for complete v1
Dependencies: 02–08
Requirements: FR-2.1/3–5, FR-1.5; PRD §§5.2, 7

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Implement the Account form for service name, website address, account identifier, masked Base32 secret, advanced settings and optional backup codes.
- [ ] Support SHA-1/256/512, 6/8 digits, periods 15/30/60/120 and required defaults. Share validation/review widgets with screenshot import.
- [ ] Implement editing real stored entries by stable ID, preserving existing secret unless deliberately changed, and explicit Save/Cancel with safe unsaved-input behavior.
- [ ] Allow adding a backup-code set during creation; save account and associated codes atomically. Use the same normalization rules as task 15.
- [ ] Use non-blocking duplicate warning, explicit overwrite intent, inline errors, busy states and preserved input after validation/persistence failure.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Shared holder/header presents Account with clear back/cancel behavior. — PRD/reference structure; shared implementation from task 04.
- [ ] Service, website and account identifier fields follow reference order and shared labels. — PRD/reference structure; shared implementation from task 04.
- [ ] Secret field is masked and supports deliberate reveal. — PRD/reference structure; shared implementation from task 04.
- [ ] Advanced options expand with clear open state and validated selections. — PRD/reference structure; shared implementation from task 04.
- [ ] Backup-codes toggle reveals a labeled multiline area without losing other input. — PRD/reference structure; shared implementation from task 04.
- [ ] Duplicate/field errors use shared inline feedback and preserve edits. — PRD/reference structure; shared implementation from task 04.
- [ ] Save/Cancel use a deliberate action row with safe busy/failure states. — PRD/reference structure; shared implementation from task 04.
- [ ] Theme, active icons, focus, scaling and empty/error layouts match the shared shell. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] Create and edit disposable credentials for each supported parameter; correct codes and edits survive relaunch.
- [ ] Malformed Base32, missing name and incompatible parameters cannot save; errors never echo secrets.
- [ ] Cancel leaves existing records intact; optional backup-code save is atomic.
- [ ] Real keyboard/pointer interaction, masked reveal, focus and narrow-window rendering pass independent review.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

