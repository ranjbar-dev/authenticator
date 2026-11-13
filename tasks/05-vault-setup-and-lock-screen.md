# Task 05: Local vault setup, unlock and lock lifecycle

Status: in_progress — implementation reviewed; hardware/integration QA pending
Priority: P0 foundation / screenshot-first
Dependencies: 01–04
Requirements: FR-7.1, FR-8; PRD §§7–8, 12

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Implement local vault creation with passphrase confirmation and concise unrecoverable-passphrase/backup guidance; this is not app registration.
- [x] Implement unlock, explicit Lock and configurable idle locking (default five minutes). Lock on OS session lock and suspend; require unlocking after relaunch.
- [x] On lock remove codes, revealed backup codes, decrypted view models and pending capture/import state from UI/accessibility trees; cancel in-flight sensitive operations and release keys/buffers.
- [x] Use a monotonic idle timer, bounded safe failure feedback and preserved existing ciphertext. Optional Windows-backed protection must be opt-in, remain subordinate to the passphrase/recovery design and never be required for portable backups.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [x] Outer holder uses the shared theme and compact shell. — PRD/reference structure; shared implementation from task 04.
- [x] Heading clearly distinguishes Create vault from Unlock vault, without account/login language. — PRD/reference structure; shared implementation from task 04.
- [x] Passphrase fields use masked native inputs with accessible labels. — PRD/reference structure; shared implementation from task 04.
- [x] Setup confirmation and recovery guidance appear beside the relevant fields. — PRD/reference structure; shared implementation from task 04.
- [x] Primary action reflects busy state and cannot submit twice. — PRD/reference structure; shared implementation from task 04.
- [x] Validation appears inline and never repeats a passphrase or decrypted data. — PRD/reference structure; shared implementation from task 04.
- [x] Unlock failure/corrupt-vault states offer safe retry without overwriting data. — PRD/reference structure; shared implementation from task 04.
- [x] Lock screen and accessibility tree conceal all credential content and preserve visible keyboard focus. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] Create/unlock/relock/relaunch a disposable vault without internet; wrong passphrase cannot disclose state.
- [ ] Idle, OS lock, suspend and lock during capture/review/import conceal and discard sensitive state.
- [x] Screenshots and accessibility inspection of locked UI contain no credential/account details.
- [x] Passphrase confirmation, keyboard operation and failure/retry/cancel paths work.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Resumed 13 November 2026: real native disposable creation, busy state, empty Codes and explicit Lock were observed. Distinct `review_resume05` approved this bounded evidence after independent locked-window/Tab observation and one current held-save privacy regression. [Resume evidence](../docs/qa/task05/resume-2026-11-13.md) and [independent review](../docs/qa/task05/resume-independent-review.md). Task05 remains incomplete for disconnected-machine and actual OS-lock/suspend gates. The reviewer also found a QA-only screenshot-redaction gap in revealed account fields; Task21 must repair/re-review it before capturing those states. No leaking image was created.

Implementation reviewed 2026-11-12 by distinct review_04 with focused security/data-loss assessment; no material findings. Optional technical-copy issue corrected and independently rechecked. Full commands/files/screenshots/AX/manual limits: docs/qa/task05/implementation-evidence.md. Full45 tests pass with subprocess probes exercised; independentstate5/nativeprivateHWND1/cancellation3/editor3/fmt/Clippy pass. Realpointer/key create/mismatch/busy/wrongpass/retry/Lock/CtrlL/KDF cancellation/relaunch/persistedtimeout/3secidle verified; sevenwindow/appzoom combinations. Exact12..1024-byte policy unchanged, productguidance nowplainlanguage. ActualOSsessionlock/hardwaresuspend delivery is UNVERIFIED: tests invoke actualprivateHWND messagepath, not hardware. Futurecapture/import/revealed-code lock integration remains tasks07–10/13/16/21. That acceptance checkbox stays open; do not label task fullydone. Nextindependent06 dependsaccepted02/03 and canproceed; task05implementation/API prerequisite is reviewed. No realvault or disruptiveOSlock/settings operation. Commitreference in followingcheckpoint.

Task08 integration update: author and distinct review_08 verified lock clearing of editable QR/manual review, image picker/queued results and committed account/code nodes, plus worker stale/cancellation boundaries. Task07 supplied native capture/private-WTS cancellation. Actual OS notification delivery remains unverified, as do later clipboard/recovery-code integrations. The first acceptance checkbox was corrected to open: create/unlock/relaunch and wrong-passphrase behavior were observed, but ordinary machine connectivity was not disabled. Source-local operation is not proof of the disconnected-machine gate.

