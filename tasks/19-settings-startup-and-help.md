# Task 19: Settings, auto-start and contextual offline help

Status: todo
Priority: Required for complete v1
Dependencies: 05, 10, 12–18
Requirements: FR-7.2–4, FR-8; PRD §§5.4, 7, 12

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Implement Settings with accurate vault/backup-code/trash counts, lock status, app version, export/import access, auto-lock duration, clipboard timeout and optional Auto Start.
- [ ] Default auto-lock is five minutes and clipboard expiry 30 seconds; retain encrypted preferences. Register/unregister startup only on explicit action and report OS failures. Reconcile OS startup registration with desired preference after unlock; never silently enable it from import.
- [ ] Provide English offline help for QR acquisition, supported formats, clock errors, backup codes, passphrase loss, portability and permanent deletion. Website addresses/icons must not trigger runtime network requests.
- [ ] Detect observable clock discontinuities and refresh codes; advise correcting Windows time without changing it or querying a remote time service.
- [ ] Omit sign-in/profile, sync, language selector, extension/companion upsells, developer mode, rating and sharing.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Shared Settings holder begins with concise real vault counts/status. — PRD/reference structure; shared implementation from task 04.
- [ ] Export & Import is easy to find near the top. — PRD/reference structure; shared implementation from task 04.
- [ ] Lock control and auto-lock setting show current state with labeled choices. — PRD/reference structure; shared implementation from task 04.
- [ ] Clipboard timeout is clearly labeled with its practical behavior. — PRD/reference structure; shared implementation from task 04.
- [ ] Auto Start shows actual success/failure and requires explicit intent. — PRD/reference structure; shared implementation from task 04.
- [ ] Version and contextual local help are accessible without internet. — PRD/reference structure; shared implementation from task 04.
- [ ] Clock/privacy/recovery guidance is concise and avoids unsupported security claims. — PRD/reference structure; shared implementation from task 04.
- [ ] Shared row hierarchy, opaque active icons, focus and responsive layout apply; excluded controls are absent. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] Counts/settings persist correctly after CRUD/import/restore/restart and failures surface safely.
- [ ] Enable/disable startup updates the app's own registration only; OS denial is recoverable.
- [ ] All help and core settings function with networking disabled; no excluded control or runtime network call exists.
- [ ] Keyboard, high-contrast/readability and narrow layouts pass independent review.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

