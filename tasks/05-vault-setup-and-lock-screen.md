# Task 05: Local vault setup, unlock and lock lifecycle

Status: todo
Priority: P0 foundation / screenshot-first
Dependencies: 01–04
Requirements: FR-7.1, FR-8; PRD §§7–8, 12

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Implement local vault creation with passphrase confirmation and concise unrecoverable-passphrase/backup guidance; this is not app registration.
- [ ] Implement unlock, explicit Lock and configurable idle locking (default five minutes). Lock on OS session lock and suspend; require unlocking after relaunch.
- [ ] On lock remove codes, revealed backup codes, decrypted view models and pending capture/import state from UI/accessibility trees; cancel in-flight sensitive operations and release keys/buffers.
- [ ] Use a monotonic idle timer, bounded safe failure feedback and preserved existing ciphertext. Optional Windows-backed protection must be opt-in, remain subordinate to the passphrase/recovery design and never be required for portable backups.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Outer holder uses the shared theme and compact shell. — PRD/reference structure; shared implementation from task 04.
- [ ] Heading clearly distinguishes Create vault from Unlock vault, without account/login language. — PRD/reference structure; shared implementation from task 04.
- [ ] Passphrase fields use masked native inputs with accessible labels. — PRD/reference structure; shared implementation from task 04.
- [ ] Setup confirmation and recovery guidance appear beside the relevant fields. — PRD/reference structure; shared implementation from task 04.
- [ ] Primary action reflects busy state and cannot submit twice. — PRD/reference structure; shared implementation from task 04.
- [ ] Validation appears inline and never repeats a passphrase or decrypted data. — PRD/reference structure; shared implementation from task 04.
- [ ] Unlock failure/corrupt-vault states offer safe retry without overwriting data. — PRD/reference structure; shared implementation from task 04.
- [ ] Lock screen and accessibility tree conceal all credential content and preserve visible keyboard focus. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] Create/unlock/relock/relaunch a disposable vault without internet; wrong passphrase cannot disclose state.
- [ ] Idle, OS lock, suspend and lock during capture/review/import conceal and discard sensitive state.
- [ ] Screenshots and accessibility inspection of locked UI contain no credential/account details.
- [ ] Passphrase confirmation, keyboard operation and failure/retry/cancel paths work.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

