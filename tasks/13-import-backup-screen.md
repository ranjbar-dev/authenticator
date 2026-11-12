# Task 13: Easy backup import, preview and conflict decisions

Status: todo
Priority: Required for complete v1
Dependencies: 04, 05, 11, 12
Requirements: FR-6.2/4; PRD §§7, 11–12

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Build Import backup with local file selection, concealed backup-passphrase entry, validation and a safe counts/conflict preview.
- [ ] Offer explicit merge/replace and per-conflict decisions consistent with the engine; default merge preserves existing entries. Require confirmation before replacement.
- [ ] Show exactly what will change, including trash and used-code state; gate optional imported preference/security/startup effects behind deliberate choices.
- [ ] Commit only approved validated data atomically; cancel/wrong password/invalid format/lock leaves the vault unchanged. Unsupported reference .vault files receive an honest compatibility message, never guessed parsing.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Shared header names Import backup with clear back/cancel actions. — PRD/reference structure; shared implementation from task 04.
- [ ] File selection displays supported native backup format guidance. — PRD/reference structure; shared implementation from task 04.
- [ ] Passphrase entry is masked and errors reveal no payload. — PRD/reference structure; shared implementation from task 04.
- [ ] Preview puts counts and affected-data summary before confirmation. — PRD/reference structure; shared implementation from task 04.
- [ ] Merge/replace and conflict selectors have readable labels and safe defaults. — PRD/reference structure; shared implementation from task 04.
- [ ] Destructive replacement receives explicit confirmation. — PRD/reference structure; shared implementation from task 04.
- [ ] Progress/error states preserve the existing vault and offer retry. — PRD/reference structure; shared implementation from task 04.
- [ ] Success shows imported data, focus returns logically and sensitive buffers are released. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] Restore task 12's backup into a fresh local vault and verify codes, metadata, folders, backup used states, trash and supported preferences.
- [ ] Exercise incorrect passphrase, tampering, future version, conflict decisions, replace confirmation, cancel and lock/failure during import.
- [ ] Imported preferences do not silently enable startup or weaken lock settings.
- [ ] Real pointer/keyboard preview choices work and independent visual review passes.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

