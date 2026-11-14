# Task 18: Trash, restoration and permanent deletion

Status: done — numbered UI scope independently reviewed with native state evidence
Priority: Required for complete v1
Dependencies: 10–17
Requirements: FR-1.5, FR-5; PRD §§5.4, 8–9, 12

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Implement confirmed soft deletion and a Trash screen with service/account, deletion time and the explicit no-automatic-expiry policy.
- [x] Provide real Restore, Permanently delete and Empty trash buttons/context actions; do not require gestures.
- [x] Restore seed, metadata, folder relationships where folders still exist and associated backup-code states together. Preview meaningful collisions, preserve current entries and report atomic failure.
- [x] Require confirmation before permanent deletion of one/all entries. Remove logical references and encrypted records; explain that filesystem/SSD/old-backup secure erasure is not guaranteed.
- [x] Include trash in backup round-trips; ordinary active account/backup lists exclude it.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [x] Shared Trash header shows count and Empty trash action. — PRD/reference structure; shared implementation from task 04.
- [x] Policy explains that items remain until permanently deleted. — PRD/reference structure; shared implementation from task 04.
- [x] Rows identify deleted service/account and deletion time without exposing secrets. — PRD/reference structure; shared implementation from task 04.
- [x] Restore is a visible keyboard-operable action. — PRD/reference structure; shared implementation from task 04.
- [x] Single permanent deletion is separate from restore. — PRD/reference structure; shared implementation from task 04.
- [x] Empty trash confirmation explains scope and irreversibility. — PRD/reference structure; shared implementation from task 04.
- [x] Conflict/error/empty states offer actionable recovery without data loss. — PRD/reference structure; shared implementation from task 04.
- [x] Theme, icon contrast, focus and narrow layouts match the shared shell; no swipe-only dependency. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [x] Delete/restore a disposable entry without changing TOTP settings or recovery-code used states.
- [x] Conflict/failure/cancel paths preserve existing active and trashed entries.
- [x] No expiry occurs silently; single/all permanent deletion requires explicit confirmation.
- [x] Restart/import/restore counts and references stay consistent; real keyboard/pointer actions and visual review pass.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

**Current accepted outcome:** The [same-checker re-review](../docs/qa/task24/state-gaps/independent-review.md) recommends closing this task after combining the original source/security/fault tests, reviewed Task21/23 native recovery and the Task24 focus/state evidence. All listed checkboxes are now reconciled to that evidence. Actual offline/other-user/no-Rust, Windows lifecycle/logon and heard-speech gates remain with their explicitly blocked owners; no global certification follows from this page. Earlier evidence paragraphs below retain their historical launch/source context and are superseded only for the now-tested scope.


Current native21 verified account Cancel ciphertext preservation, restore with two memberships/set/two codes/used history, folder and individual-code Trash, single permanent-delete and Empty Cancel/Confirm, plus original Trash dates/states restored by native backup import. [Journey](../docs/qa/task21/remaining-native-workflows.md), [independent review](../docs/qa/task21/remaining-native-independent-review.md). Parent fixed a timing-sensitive pending-preference test precondition without changing runtime behavior; the exact correction independently passed, but new hosted full-suite evidence remains required. [CI history](../docs/qa/task21/current-ci-follow-up.md).

Limited source approval by distinct review_18, with no material findings. Implementation checks above describe source/domain/worker/Context evidence, not native screen acceptance. Exact paths, commands, ownership/cleanup and limits: [author evidence](../docs/qa/task18/implementation-evidence.md) and [independent review](../docs/qa/task18/independent-review.md).

All20 new library cases passed individually; independently18 focused cases passed across11 bounded invocations, including all five own-Trash kinds, preserved OTP/used history, parent-first/collision consent, cascade union/final-code set disclosure, equal-time stale checks, failed writes/retry/latest preferences, held preparation/admitted Lock and shared Close. Production/lib/own-example build, strict Clippy, formatting and diff checks pass. The author full196-library/all-target run is INCOMPLETE; retained process identity/cause limitations are recorded, not diagnosed by focused success. Current-source hosted suite remains required.

Native screenshots/input:0. Normal own preview launch failed GetCursorPos0x80070005 before resources; no bypass or unchanged retry. Real pointer/keyboard, raster quality, physical DPI/Narrator/offline/hardware and earlier interactive clipboard gates remain OPEN. Screen checklist remains unchecked for native acceptance. Coherent source checkpoint commit and hosted evidence follow in tasks/README.md; continue independent19 without declaring18done.

Hosted follow-up: [CI43391082018](https://github.com/ranjbar-dev/authenticator/actions/runs/43391082018) SUCCESS at6264c54, library205passed/0failed/4directignored plus18other top-level tests, formatting/lint/app build pass. Includes all Task18 cases and independently reviewed corrections to the two previous CI18 failures. Full-suite source gate closes; native/manual acceptance remains open. See [exact follow-up](../docs/qa/task19/hosted-follow-up.md).

