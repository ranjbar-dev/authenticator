# Task 12: Easy encrypted backup export screen

Status: in_progress - corrected source approved; native UI/visual and Task13 GUI restore gates open
Priority: Required for complete v1
Dependencies: 04, 05, 11
Requirements: FR-6.1/3; PRD §§5.4, 7

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Build Settings → Export & Import → Export encrypted backup. Show a concise included-data summary and local file destination picker.
- [x] Require backup passphrase and confirmation, conceal both and explain that this passphrase cannot be recovered. Do not provide an encryption-disable switch.
- [x] Show explicit overwrite confirmation, busy state, cancellation handling and success destination without contents. Release passphrase buffers after exit/completion.
- [x] Use the real backup engine and show safe actionable failures such as denied access or insufficient space.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Shared header names Export encrypted backup and provides back navigation. — PRD/reference structure; shared implementation from task 04.
- [ ] Summary explains what is included without showing credentials. — PRD/reference structure; shared implementation from task 04.
- [ ] Local destination control uses a file picker and a readable selected path. — PRD/reference structure; shared implementation from task 04.
- [ ] Masked passphrase/confirmation fields have accessible labels and recovery guidance. — PRD/reference structure; shared implementation from task 04.
- [ ] Encryption is mandatory; no plaintext or cloud option appears. — PRD/reference structure; shared implementation from task 04.
- [ ] Overwrite confirmation names the file and requires deliberate intent. — PRD/reference structure; shared implementation from task 04.
- [ ] Export action shows progress, prevents repeated submission and handles cancellation. — PRD/reference structure; shared implementation from task 04.
- [ ] Success/error feedback is safe, readable and theme-consistent; closing clears sensitive fields. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] A new user exports a complete encrypted disposable vault through labeled UI actions without external tools or registration.
- [ ] Passphrase mismatch/cancel/access failure leaves existing files and vault unchanged.
- [ ] The output restores via task 13 and discloses no plaintext on inspection.
- [ ] Pointer, keyboard, focus, busy/error and narrow-window checks pass independent review.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Corrected source approved by distinct review_12/focused re-review after fixing
P2 export of stale debounced display preferences. Implementation boxes above
record source/engine/framework evidence. Screen and acceptance items remain open
for actual native interaction/rendering and Task13 GUI recovery; no screenshots
or new-user native completion are claimed.

- [Implementation and exact command matrix](../docs/qa/task12/implementation-evidence.md).
- [Independent finding, correction and scope](../docs/qa/task12/independent-review.md).
- Final131 tests passed,4 subprocess helper entrypoints exercised by parents,
  15 suites76.52s; strict Clippy/fmt/diff and production/lib/export-example build pass.
- Independent export9/picker2/contrast1 and focused current-preferences1,
  save-failure/retry1, unexpected-lock1 passed. The original external probe exposed
  the P2 rather than being recorded as a passing acceptance case.
- Preference preparation uses the existing guarded worker, retains reviewed bytes/
  overwrite consent and waits for current save before encryption. Failure blocks
  backup writing and clears phrases while preserving retry intent. A completed
  earlier preference save is not rolled back by a later export failure/cancel.
- Both author normal launches and the reviewer launch returned GetCursorPos
  Access denied0x80070005, with no owned window/process/fixture. No bypass or cause
  inference. Synthetic event/AX/geometry checks are not native/pixel proof.
- Actual picker/pointer/keyboard/focus, long paths/short-window modals, readable
  340/600/960 screenshots, busy/error/success/new-user flow and Task13 GUI restore
  remain OPEN. Physical/offline/device/Narrator/reference gates unchanged.

Task13 now verifies this production form/worker output through its own preview/plan/
apply into a fresh independent local vault, including full codec graph and24 OTP
settings. Its distinct reviewer approved the corrected source. Actual native
Task12-to-13 interactive recovery remains OPEN; revisit these gates in21/23.
No real/default/reference data, retained owned state or unrelated preview changes.
Commit reference follows in the parent checkpoint.
