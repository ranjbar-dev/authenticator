# Task 08: Complete screenshot-to-2FA workflow

Status: in_progress — functional implementation independently approved; disconnected-machine QA open
Priority: P0 foundation / screenshot-first
Dependencies: 01–07
Requirements: FR-2, FR-2A; PRD highest-priority success criterion

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Deliver the first complete feature: Screenshot to Scan → region selection → local decoding → candidate selection → editable review → encrypted save → current code.
- [x] Review service, website/account identifier and TOTP parameters with the secret concealed by default; retain supported URI parameters. Let users select among multiple credentials and edit before saving.
- [x] Show a non-blocking duplicate warning and require explicit decisions before overwriting. Prefer creating a separate entry when desired; never silently replace.
- [x] Revalidate reviewed edits before atomic save. For selected multiple candidates, commit the approved batch atomically; errors/cancellation leave current vault unchanged. Clear capture/QR review buffers on every exit.
- [x] After success navigate to and identify the imported account; verify its current code. Provide retry/image/manual alternatives for all failure classes.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [x] Entry point preserves the compact codes-screen/add-flow structure and prioritizes Screenshot to Scan. — PRD/reference structure; shared implementation from task 04.
- [x] Candidate chooser lists recognizable service/account labels with explicit selection. — PRD/reference structure; shared implementation from task 04.
- [x] Review form uses shared labeled controls for metadata and algorithm/digits/period. — PRD/reference structure; shared implementation from task 04.
- [x] Secret field is masked and its reveal action is intentional. — PRD/reference structure; shared implementation from task 04.
- [x] Duplicate/validation messages explain action without leaking secrets. — PRD/reference structure; shared implementation from task 04.
- [x] Save/Import is explicit, visibly busy and protected from repeated activation. — PRD/reference structure; shared implementation from task 04.
- [x] Success shows the new real vault entry and code; failure offers actionable recovery. — PRD/reference structure; shared implementation from task 04.
- [x] Cancel/back/lock release sensitive state and restore logical focus; no excluded reference features appear. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] With networking disabled, capture a disposable QR in another app, review it, save and compare the resulting TOTP to an independent implementation.
- [x] Test multi-candidate choice, duplicates, invalid edits, cancel, lock during decode/review/save and write failure; no partial batch import occurs.
- [x] A user never has to type the QR secret or save a screenshot file.
- [x] Independent pointer/keyboard and visual review approves the complete flow.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Functional implementation approved by distinct read-only review_08 with no findings requiring changes. Exact changed paths, commands, screenshots, two author independent-code proofs, faults/fixes and limits: [implementation-evidence.md](../docs/qa/task08/implementation-evidence.md). Independent commands, pointer/keyboard journeys and fresh HMAC comparisons: [independent-review.md](../docs/qa/task08/independent-review.md).

The author full suite passed76 ordinary tests with two helper entrypoints ignored in the ordinary listing and exercised by subprocess tests. Final focused UI14, duplicate3, domain12, atomic-batch regression, build/fmt/Clippy passed. Reviewer independently repeated real capture→masked/editable review→invalid-edit recovery→explicit encrypted save→current code, two-candidate selection, preserved SHA512/8/120, write-failure/retry, duplicate preservation, Help/Cancel and picker/lock privacy at340/600/960 widths. Independent live SHA1 and SHA512 HMAC comparisons matched. Parent inspected source/diff and sanitized representative components.

Cancellation must win the commit guard to prevent saving. If the atomic save has already won, later Cancel/Lock may leave the complete encrypted batch, which the busy UI explains; no partial batch or false rollback is accepted. Before-commit cancellation/fault invariants are covered by deterministic storage/worker/publication tests. Precise manual interruption of short decode/save was not claimed.

Networking-disabled execution remains unverified, so the first acceptance checkbox stays open and this task is not fully done. Hardware DPI/OS lock/suspend, Narrator/reference fidelity and later full feature QA remain their recorded gates. Functional APIs are ready for task09. Commit reference is recorded in the following checkpoint.

