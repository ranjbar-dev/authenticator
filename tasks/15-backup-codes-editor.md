# Task 15: Add/edit backup codes from text and file

Status: todo
Priority: Required for complete v1
Dependencies: 09, 14
Requirements: FR-3.1/3/4, FR-2.5; PRD §§7–9, 12

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Implement the account-linked editor with one-code-per-line text input, local UTF-8 text-file reading and a preview before save.
- [ ] Default file limit is 1 MiB; accept UTF-8 BOM and CRLF/LF, trim surrounding whitespace and discard blank lines. Detect exact duplicate lines, show a safe count and require review before deduplication/save. Reject invalid encoding and over-limit data without altering the set.
- [ ] Share normalization with account creation. Never infer used state from copying or silently merge/overwrite an existing set.
- [ ] Support explicit edit/save/cancel/delete-set confirmation. On replacement preserve used state only for exact retained codes and document the preview behavior. Keep raw files out of storage.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Shared editor holder identifies the selected service/account. — PRD/reference structure; shared implementation from task 04.
- [ ] Multiline code input is labeled and concealed where practical; reveal is intentional. — PRD/reference structure; shared implementation from task 04.
- [ ] Read Backup Codes from File uses a local picker and explains UTF-8 input. — PRD/reference structure; shared implementation from task 04.
- [ ] Preview summarizes valid/blank/duplicate lines without unsolicited disclosure. — PRD/reference structure; shared implementation from task 04.
- [ ] Existing-set replacement and retained used-state behavior are clearly explained. — PRD/reference structure; shared implementation from task 04.
- [ ] Save/Cancel/Delete use shared actions and destructive confirmation. — PRD/reference structure; shared implementation from task 04.
- [ ] File/validation/write errors retain safe unsaved input and offer recovery. — PRD/reference structure; shared implementation from task 04.
- [ ] Keyboard, focus, scrolling, theme and narrow-window actions remain usable; lock clears input. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] Text/file input, BOM/line endings, blanks, duplicates, invalid encoding and oversized file paths have deterministic tested outcomes.
- [ ] Cancel/failed save preserves existing set and edits; confirmed save survives restart.
- [ ] No raw file remains and validation messages never contain code values.
- [ ] Create-account-with-backup and standalone editor produce identical normalized results.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

