# Task 10: Complete 2FA list, search, sort, views and copy

Status: todo
Priority: Required for complete v1
Dependencies: 02–09
Requirements: FR-1; PRD §§5.1, 7, 11

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Build real account cards showing local icon/fallback, service, optional account identifier, formatted code and accurate remaining-time progress.
- [ ] Refresh at exact configured boundaries, time changes and resume; suspend unnecessary repainting while hidden/locked. Treat implausible clock shifts as actionable guidance without claiming remote verification.
- [ ] Implement live service/account search, no-results state, all six reference sort modes, persistent list/grid preference and stable tie-breaking.
- [ ] Implement explicit copy with confirmation and configurable default 30-second clearing. Clear only if the clipboard still contains the app-owned copy; use ownership/sequence metadata when available. Do not overwrite subsequent external copies.
- [ ] Expose edit and confirmed delete-to-trash actions; code copy must not navigate/edit/delete. Use local icons without network requests.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Header uses reference-style title/help and prominent add/Screenshot to Scan access. — PRD/reference structure; shared implementation from task 04.
- [ ] Toolbar offers folder navigation, search, six-way sort and list/grid selection with visible current states. — PRD/reference structure; shared implementation from task 04.
- [ ] Cards prioritize service/account, readable code and remaining-time indicator. — PRD/reference structure; shared implementation from task 04.
- [ ] Service icons use consistent solid tiles with opaque contrasting strokes. — PRD/reference structure; shared implementation from task 04.
- [ ] Copy and account actions are explicit and individually keyboard reachable. — PRD/reference structure; shared implementation from task 04.
- [ ] Empty/loading/no-results/clock-error states retain useful actions near the top. — PRD/reference structure; shared implementation from task 04.
- [ ] List/grid preserve readable hierarchy at narrow, intermediate and desktop widths. — PRD/reference structure; shared implementation from task 04.
- [ ] Locked state removes codes and metadata from the UI/accessibility tree; shared tokens and focus rules are used. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] Cards match independent TOTP values at boundaries and after sleep/resume/clock jumps; no stale code remains.
- [ ] Search, every sort mode and both views persist correctly through restart.
- [ ] Clipboard timeout/lock clearing works without overwriting newer user clipboard content, including race cases.
- [ ] Real pointer clicks and keyboard shortcuts reach copy/edit/delete/search/sort/view without drag scrolling stealing clicks.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

