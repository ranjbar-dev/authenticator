# Task 19: Settings, auto-start and contextual offline help

Status: blocked — offline/logon and heard screen-reader acceptance unavailable
Priority: Required for complete v1
Dependencies: 05, 10, 12–18
Requirements: FR-7.2–4, FR-8; PRD §§5.4, 7, 12

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Implement Settings with accurate vault/backup-code/trash counts, lock status, app version, export/import access, auto-lock duration, clipboard timeout and optional Auto Start.
- [x] Default auto-lock is five minutes and clipboard expiry 30 seconds; retain encrypted preferences. Register/unregister startup only on explicit action and report OS failures. Reconcile OS startup registration with desired preference after unlock; never silently enable it from import.
- [x] Provide English offline help for QR acquisition, supported formats, clock errors, backup codes, passphrase loss, portability and permanent deletion. Website addresses/icons must not trigger runtime network requests.
- [x] Detect observable clock discontinuities and refresh codes; advise correcting Windows time without changing it or querying a remote time service.
- [x] Omit sign-in/profile, sync, language selector, extension/companion upsells, developer mode, rating and sharing.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [x] Shared Settings holder begins with concise real vault counts/status. — PRD/reference structure; shared implementation from task 04.
- [x] Export & Import is easy to find near the top. — PRD/reference structure; shared implementation from task 04.
- [x] Lock control and auto-lock setting show current state with labeled choices. — PRD/reference structure; shared implementation from task 04.
- [x] Clipboard timeout is clearly labeled with its practical behavior. — PRD/reference structure; shared implementation from task 04.
- [x] Auto Start shows actual success/failure and requires explicit intent. — PRD/reference structure; shared implementation from task 04.
- [x] Version and contextual local help are accessible without internet. — PRD/reference structure; shared implementation from task 04.
- [x] Clock/privacy/recovery guidance is concise and avoids unsupported security claims. — PRD/reference structure; shared implementation from task 04.
- [x] Shared row hierarchy, opaque active icons, focus and responsive layout apply; excluded controls are absent. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [x] Counts/settings persist correctly after CRUD/import/restore/restart and failures surface safely.
- [x] Enable/disable startup updates the app's own registration only; OS denial is recoverable.
- [ ] All help and core settings function with networking disabled; no excluded control or runtime network call exists.
- [x] Keyboard, high-contrast/readability and narrow layouts pass independent review.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Later 14 November execution investigated the [documented Narrator copy observer](../docs/qa/task19/speech-copy-2026-11-14.md). The shortcut copies only the last phrase, which cannot alone prove absence of intermediate OTP announcements. Two normal fixture launches failed with GetCursorPos Access denied before Narrator or any positive control ran. [Sanitized facts](../docs/qa/task19/speech-copy-2026-11-14.json) distinguish exec-caller input-desktop denial from an unknown helper cause; no new speech/UI pass, source change or task acceptance follows. [Distinct scoped review](../docs/qa/task19/speech-copy-independent-review-2026-11-14.md) approved the corrected evidence; the observed stopping decision is not an invented launch retry rule.

Current native21 verified real Settings counts across recovery used/Undo, Trash/restore, full-graph Merge and Replace, and explicit imported-display/retained-security/startup choices. A missing navigation glyph was replaced with plain English and verified in a fresh compiled native help view. [Journey](../docs/qa/task21/remaining-native-workflows.md), [independent review](../docs/qa/task21/remaining-native-independent-review.md). The example path could not register startup and no production Run/logon operation was attempted. Offline-machine, OS delivery and remaining help/state/scale checks stay open.

Corrected source approved after distinct review_19 and focused re-review. The checked implementation/core items describe source, real encrypted store/reopen/import and isolated Windows registry adapter evidence. Screen/native acceptance remains open; no production Run registration or actual logon was tested.

- [Author paths, commands, targeted results and limits](../docs/qa/task19/implementation-evidence.md).
- [Independent findings, current-source approval and exact checks](../docs/qa/task19/independent-review.md).
- [Parent red/green privacy correction and cleanup](../docs/qa/task19/parent-fixes.md).
- [Both corrected Task18 hosted failures](../docs/qa/task19/task18-regressions.md).

Author16 focused cases pass; independent15 original focused cases and two current correction checks pass. P2 Settings Lock triggering-frame metadata was fixed by consolidating Lock in the shared header before private content. The permanent all-present-actions regression checks release-frame AX and paint at340/600/960 with one/two-pass budgets; parent red8.44s, final green28.90s, independent28.92s. P3 startup claims now match UNC/prefix checks and disclose unresolved drive mappings. Both previous CI18 failures independently pass after meaningful test corrections; old hostedCI18 remains failed until a new full run passes.

Current209library entries compile; final fmt/strict all-target/all-feature Clippy/diff and production/bin+lib build pass. No local full-suite result is claimed. Native screenshots/input0; actual offline-machine, pointer/keyboard/raster/DPI/Narrator/high-contrast/logon/OS-lock/suspend/clipboard/package gates stay OPEN. No taskdone, conformance, security certification or production-readiness claim. Commit/run reference follows in the parent checkpoint; continue20–24.

[Hosted CI43391082018](https://github.com/ranjbar-dev/authenticator/actions/runs/43391082018) subsequently SUCCESS at6264c54:205library+18other top-level tests,0failures,4directignored helper entrypoints, format/lint/appbuild allpass. [Exact evidence](../docs/qa/task19/hosted-follow-up.md). This closes the hosted full-suite source gate, not native/offline/logon/manual acceptance or later Task20 changes.


Current acceptance boundary: all required feasible software-state cases have [independent closure](../docs/qa/task24/state-gaps/independent-review.md). Reviewed source109171a has clean local packaging and successful full hosted check/package evidence; postA QA-only additions preserve exact release bytes and have targeted checks/re-review. This task is blocked only by the named resource/tool-limited acceptance above, not by an unreported software TODO. Resume on a ready isolated Windows environment with effective networking disabled/no Rust and independent user recovery, then test actual OS lifecycle/logon and audible Narrator behavior using a working positive control. Existing source/adapter/AX tests do not pretend those runtime checks passed. The final integration update supplies the actual main SHA and attached CI result.
