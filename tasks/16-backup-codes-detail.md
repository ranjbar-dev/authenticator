# Task 16: Reveal/copy, used-state and delete recovery codes

Status: todo
Priority: Required for complete v1
Dependencies: 14, 15
Requirements: FR-3.2–4; PRD §§7–9

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Implement a selected account's backup-code detail view with concealed codes, intentional reveal/copy, per-code used marking and undo.
- [ ] Copy must not mark a code used; persist explicit used changes atomically with timestamps. Apply app-owned clipboard expiration/lock cleanup rules.
- [ ] Support editing and confirmed deletion of the set, with service-issued/single-use guidance. Restore associated codes with account trash recovery; do not orphan them.
- [ ] Conceal everything sensitive on lock/navigation where appropriate and keep changing/revealed code values out of status announcements/logs.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Header identifies the account with consistent back/edit controls. — PRD/reference structure; shared implementation from task 04.
- [ ] Guidance explains service issuance and potential single use without excessive text. — PRD/reference structure; shared implementation from task 04.
- [ ] Rows conceal codes by default with clear available/used state. — PRD/reference structure; shared implementation from task 04.
- [ ] Reveal and Copy are separate labeled intentional actions. — PRD/reference structure; shared implementation from task 04.
- [ ] Mark used and Undo expose their current state accessibly. — PRD/reference structure; shared implementation from task 04.
- [ ] Delete set has clear irreversible confirmation; account data is preserved. — PRD/reference structure; shared implementation from task 04.
- [ ] Empty/error/copy-confirmation states never echo a code. — PRD/reference structure; shared implementation from task 04.
- [ ] Shared theme, focus and narrow layouts apply; lock removes revealed content and accessibility values. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] Reveal/copy/mark-used/undo/edit/delete work with pointer and keyboard; copy alone leaves used state unchanged.
- [ ] Used states persist across restart and encrypted backup round-trip.
- [ ] Lock, idle timeout and clipboard races disclose no hidden codes or overwrite newer clipboard content.
- [ ] Account trash/restore preserves its associated set and states; permanent deletion removes logical references.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

