# Task 09: Manual creation and account editing

Status: done
Priority: Required for complete v1
Dependencies: 02–08
Requirements: FR-2.1/3–5, FR-1.5; PRD §§5.2, 7

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Implement the Account form for service name, website address, account identifier, masked Base32 secret, advanced settings and optional backup codes.
- [x] Support SHA-1/256/512, 6/8 digits, periods 15/30/60/120 and required defaults. Share validation/review widgets with screenshot import.
- [x] Implement editing real stored entries by stable ID, preserving existing secret unless deliberately changed, and explicit Save/Cancel with safe unsaved-input behavior.
- [x] Allow adding a backup-code set during creation; save account and associated codes atomically. Use the same normalization rules as task 15.
- [x] Use non-blocking duplicate warning, explicit overwrite intent, inline errors, busy states and preserved input after validation/persistence failure.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [x] Shared holder/header presents Account with clear back/cancel behavior. — PRD/reference structure; shared implementation from task 04.
- [x] Service, website and account identifier fields follow reference order and shared labels. — PRD/reference structure; shared implementation from task 04.
- [x] Secret field is masked and supports deliberate reveal. — PRD/reference structure; shared implementation from task 04.
- [x] Advanced options expand with clear open state and validated selections. — PRD/reference structure; shared implementation from task 04.
- [x] Backup-codes toggle reveals a labeled multiline area without losing other input. — PRD/reference structure; shared implementation from task 04.
- [x] Duplicate/field errors use shared inline feedback and preserve edits. — PRD/reference structure; shared implementation from task 04.
- [x] Save/Cancel use a deliberate action row with safe busy/failure states. — PRD/reference structure; shared implementation from task 04.
- [x] Theme, active icons, focus, scaling and empty/error layouts match the shared shell. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [x] Create and edit disposable credentials for each supported parameter; correct codes and edits survive relaunch.
- [x] Malformed Base32, missing name and incompatible parameters cannot save; errors never echo secrets.
- [x] Cancel leaves existing records intact; optional backup-code save is atomic.
- [x] Real keyboard/pointer interaction, masked reveal, focus and narrow-window rendering pass independent review.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Accepted after distinct review_09 and focused re-review resolved P2 responsive-lock and P3 stale duplicate feedback findings. Source/checklist/decisions, exact commands,20 sanitized screenshots, all24 persisted parameter combinations, actual process restart and independent HMAC matches: [implementation-evidence.md](../docs/qa/task09/implementation-evidence.md). Independent source/tests/live interaction scope, findings and recheck: [independent-review.md](../docs/qa/task09/independent-review.md).

Final author full suite88 ordinary tests passed; two existing ignored entrypoints are invoked by subprocess probes. Final formatting, strict all-target/all-feature Clippy and app/examples build passed. Independent re-review passed held-save UI lock/privacy1, cancellation5, duplicate feedback1, atomic account/recovery1 and fmt/diff checks. New account+codes are atomic; stable-ID edits preserve the existing seed unless replacement is deliberate and reject stale snapshots, including same-second changes. Exact-code normalization/duplicate review is reusable by task15.

The actual process close→normal relaunch started locked and restored edited SHA512/8/120 after explicit unlock, independently HMAC matched. Independent first review additionally matched SHA256/6/60 and exercised340/600/960 pointer/key edit/create/error/discard/lock flows. The final pointer repetition hit native cursor-access denial; author final feedback pointer evidence plus independent tests/source support the scoped recheck. No settings bypass was used. Physical/mixed DPI, disconnected-machine, actual OS lock/suspend, Narrator/reference-pixel gates remain with their owning tasks. This task does not complete task10 list/clipboard or task15 standalone recovery-code workflows. Commit reference follows in the next checkpoint.

