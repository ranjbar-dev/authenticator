# Task 15: Add/edit backup codes from text and file

Status: in_progress - native file/duplicate-preview/save and round-trip reviewed; remaining full QA gates open
Priority: Required for complete v1
Dependencies: 09, 14
Requirements: FR-3.1/3/4, FR-2.5; PRD §§7–9, 12

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Implement the account-linked editor with one-code-per-line text input, local UTF-8 text-file reading and a preview before save.
- [x] Default file limit is 1 MiB; accept UTF-8 BOM and CRLF/LF, trim surrounding whitespace and discard blank lines. Detect exact duplicate lines, show a safe count and require review before deduplication/save. Reject invalid encoding and over-limit data without altering the set.
- [x] Share normalization with account creation. Never infer used state from copying or silently merge/overwrite an existing set.
- [x] Support explicit edit/save/cancel/delete-set confirmation. On replacement preserve used state only for exact retained codes and document the preview behavior. Keep raw files out of storage.

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

Current native21 selected generated UTF-8 BOM/CRLF input through the real Open dialog; preview reported two unique codes, one blank and one duplicate, explicit consent saved the normalized set, and used history survived actual fresh native backup restore. [Journey](../docs/qa/task21/remaining-native-workflows.md), [independent review](../docs/qa/task21/remaining-native-independent-review.md). Existing adversarial/editor tests remain their own evidence; this does not assert every native edit/error/restart/scale branch was repeated.

Corrected source/Context behavior approved by distinct review_15 and focused
re-review. Implementation boxes record source/engine/framework evidence; actual
native screen/acceptance gates remain open.

- [Author checklist, policy and exact checks](../docs/qa/task15/implementation-evidence.md).
- [Independent security/UI review and focused verdict](../docs/qa/task15/independent-review.md).
- [Parent focus correction and final checks](../docs/qa/task15/parent-fixes.md).
- Final168 tests passed,4 direct helper entrypoints exercised by parents,18 suites
  110.19s; strict all-target/all-feature Clippy, fmt/diff and production/lib/owned
  editor-example all-feature build pass. Author19 recovery tests and independent19
  passed; focused direct-delete1/36.66s and original Cancel-focus1/9.37s passed.
- Shared parser handles BOM/line endings/physical blanks/exact duplicates/byte limits.
  Worker file reads reject unsafe paths and actual leaf/parent links, retain failed
  input and never copy/delete the user source. Full-content stale guards handle equal
  timestamps. Exact retained and re-added values retain IDs/used history; new values
  are unused; omitted active codes move to encrypted Trash. Delete Set moves the set
  to Trash, preserving account/children; restoration is Task18.
- Existing active joined text over1MiB is refused intact as read-only, with Delete
  Set still available; no accepted truncation. Input-bound Preview/duplicate consent,
  replacement and discard confirmations, fault/retry/reopen/portable states and
  admitted-write Lock/no late adoption are tested. Raw values are masked by default.
- Shared Dialog now remeasures on opening and uses title-specific non-dragging
  scrolling. Reviewer ordinary-size safe-focus/hit tests and prior export/import/
  Help/identity regressions pass. P2 direct detail Delete cancel focus fixed by
  binding the confirmation to the actual rendered editor Delete control.
- Both normal own native launches denied GetCursorPos0x80070005 with no owned
  window/process/fixture. No native screenshot/input/picker/restart or raster-quality
  pass is claimed. Context events/AX, including normal-sized focus tests, remain
  distinct from native proof. Earlier hardware/offline/Copy/GUI-recovery gates stay open.

Parent integrates then executes16; no Task16 reveal/copy/used/undo claim here.
No real/default/reference data or unrelated preview changed. Commit reference
follows in parent checkpoint.

