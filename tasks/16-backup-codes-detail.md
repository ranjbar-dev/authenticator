# Task 16: Reveal/copy, used-state and delete recovery codes

Status: in_progress - native reveal/copy/used/Undo/restore/delete reviewed; remaining full QA gates open
Priority: Required for complete v1
Dependencies: 14, 15
Requirements: FR-3.2–4; PRD §§7–9

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Implement a selected account's backup-code detail view with concealed codes, intentional reveal/copy, per-code used marking and undo.
- [x] Copy must not mark a code used; persist explicit used changes atomically with timestamps. Apply app-owned clipboard expiration/lock cleanup rules.
- [ ] Support editing and confirmed deletion of the set, with service-issued/single-use guidance. Restore associated codes with account trash recovery; do not orphan them.
- [x] Conceal everything sensitive on lock/navigation where appropriate and keep changing/revealed code values out of status announcements/logs.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Header identifies the account with consistent back/edit controls. — PRD/reference structure; shared implementation from task 04.
- [ ] Guidance explains service issuance and potential single use without excessive text. — PRD/reference structure; shared implementation from task 04.
- [ ] Rows conceal codes by default with clear available/used state. — PRD/reference structure; shared implementation from task 04.
- [ ] Reveal and Copy are separate labeled intentional actions. — PRD/reference structure; shared implementation from task 04.
- [ ] Mark used and Undo expose their current state accessibly. — PRD/reference structure; shared implementation from task 04.
- [ ] Delete set clearly confirms its move to Trash; permanent deletion in Trash is explicitly irreversible and account data is preserved. — PRD12.5 and Task15 reversible deletion policy; permanent deletion belongs to18.
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

Current native21 exercised pointer Reveal/Conceal/Copy, keyboard Copy, Mark used/Undo/Mark used, revealed-state Ctrl+L cleanup, association-preserving account restoration and single-code permanent deletion after Cancel. Copy did not change used state. Full native encrypted restore/merge/replace preserved the reviewed used/Trash states. [Journey and340/600/960 frames](../docs/qa/task21/remaining-native-workflows.md), [independent review](../docs/qa/task21/remaining-native-independent-review.md). Shared clipboard availability/cleanup correction is separately approved in task10; every environment/editor/matrix branch is not inferred.

Limited source approval by distinct review_16; no material defect found in reviewed
source/isolated checks. Source boxes above do not close native or Task18 gates.
Item3 remains open for actual account restoration/permanent cascade integration18;
Edit and confirmed reversible set/code Trash already work through15/16.

- [Author checklist, policies and exact checks](../docs/qa/task16/implementation-evidence.md).
- [Independent security/UI review, commands and limits](../docs/qa/task16/independent-review.md).
- Author final178 aggregate tests passed,4 parent-exercised helper entrypoints,
  19 suites129.87s; strict Clippy/fmt/diff and production/lib/owned-example build pass.
  Last Help copy-only correction was included in static/build checks after full run.
- Independent domain2, detail7 (184.95s), direct DeleteSet-focus1 (40.67s), identity1,
  actor expiry1/admission1, shared contrast1 and diff checks passed.
- Full per-code/parent guards reject stale equal-time contents. Mark/available/Undo/
  Trash atomically touch logical timestamps; exact old used dates and identities
  survive encrypted reopen/portable preview. Failed writes preserve prior data/valid
  Undo; adopted poststate supplies Undo. Copy alone changes neither used nor ciphertext.
- Default concealed rows,20-row paging,4096-byte bounded value view, explicit two-column
  actions, static feedback and normal340/600/960 Context input/AX/focus tested. Later
  code20 and cross-page Undo restore visible live focus; reused-ID import success
  conceals, failed nested Apply preserves unrelated state. Lock clears before held
  admitted I/O; no late adoption. UI Clipboard uses cfg(test) Fake with real queue,
  never interactive user clipboard. Production actor unchanged.
- Task10 actual interactive Copy→Changed cause remains unknown; Fake/private-station
  success is not actual Copy/expiry/Lock/exit proof. Two normal own preview launches
  denied GetCursorPos0x80070005, no owned resources/screenshots. Native input/pixels/
  DPI/Narrator/hardware/device/offline gates remain OPEN. Backend restore fixtures
  reject orphans/preserve association, but do not implement18 Trash restore/delete UI.

Parent integrates source then executes17. No real/default/reference data or
unrelated preview changed; commit reference follows in parent checkpoint.

