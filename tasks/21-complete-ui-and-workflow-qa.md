# Task 21: Full offline workflow, accessibility and visual QA

Status: blocked — isolated runtime, OS lifecycle and heard-speech acceptance unavailable
Priority: Required for complete v1
Dependencies: 01–20
Requirements: PRD §§2, 6–7, 11; all UI checklists

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Run complete disposable-data journeys from clean install: vault setup → screenshot import → current-code verification → copy/edit/search/sort/views → backup codes → folders → trash/restore → encrypted export → clean-vault merge/replace restore.
- [x] Use real pointer/keyboard interactions for capture drag, dropdowns, buttons, multi-selection, dialogs and clipboard; synthetic click-only checks do not suffice.
- [x] Inspect actual component screenshots for every page at narrow, intermediate and desktop windows and 100/150/200% scale; use available reference evidence and record unavailable reference comparisons as unverified.
- [ ] Check empty/loading/no-results/open/error/focus/locked states, active-icon contrast, foreground/background pairs, clipping and logical hierarchy. Test Narrator/accessibility labels, roles, tab order and announcements without repeatedly reading OTP values.
- [x] Use a reviewer who did not implement the screens, following each task's checklist and ui-consistency:verifying if available. Repair findings and rerun affected checks.
- [ ] Store only sanitized/disposable evidence and a coverage report under docs/qa/. Complete all PRD acceptance criteria, not just a successful launch or test exit code.

## Acceptance and verification

- [x] Every FR and success criterion maps to passing reproducible tests or explicit honest evidence; no untested manual check is labeled passed.
- [ ] Screenshot-to-import works offline across available monitor/scaling combinations and failure/cancel/lock paths.
- [x] All pages are readable and operable with pointer/keyboard at tested sizes, and reference deviations are fixed or documented accurately.
- [x] Independent reviewer finds no unresolved material functional, accessibility or visual defect.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Later 14 November independent speech-observer route: [investigation](../docs/qa/task19/speech-copy-2026-11-14.md) established the official last-phrase-only shortcut semantics, then two normal fixture launches failed with GetCursorPos Access denied. Narrator, a known-label positive control and rollover observation never ran. This is a failed native prerequisite, not another failed speech observation or a new pass. All original acceptance remains open.

Current remaining native journeys and all18 sanitized frames received distinct review with no material security/visual finding. [Author evidence](../docs/qa/task21/remaining-native-workflows.md), [independent review](../docs/qa/task21/remaining-native-independent-review.md). Two minor presentation findings were corrected and independently rechecked: accurate global-ID caption and no duplicate Account ID. [Parent checks](../docs/qa/task21/parent-review-fixes.md). Test-only compiler compatibility is corrected locally; new hosted full checks are required. No unavailable environment/state/scale check is marked passed and Task21 remains incomplete.

Current source mask repair and two new locked/error frames received distinct [independent review](../docs/qa/task21/current-independent-review.md), with one exact painted-shape/frame-lifetime regression passing. This resolves the QA-only revealed-account-field capture gap. Prior [native milestones](../docs/qa/task21/resume-implementation.md) record screenshot import/HMAC/Copy feedback and one-account export/fresh Merge restore as attributed author evidence. Complete recovery/folder/Trash, clipboard lifecycle, full-graph/Replace and all-page/environment gates remain open; this is not full21 acceptance.

New execution request resumed native work on 13 November 2026. Earliest unfinished Task05 received sequential implementation/review: disposable Create → empty Codes → Lock succeeded, limited evidence independently approved. Its reviewer found a QA-capture redaction gap for revealed Account fields (no leaking capture created). Task21 now owns correcting and independently reviewing that helper before populated workflow captures. See [resumed review](../docs/qa/task05/resume-independent-review.md); previous stop below remains historical.

Interrupted by the user's physical Escape stop during native QA; all Computer Use stopped immediately. No successful native vault creation/capture/import/Copy was observed. Initialization and an isolated clean-setup app launch worked after C: cache recovery, but launch is not workflow acceptance. Five current focused framework checks passed, source harness/build and one clean setup frame are attributed evidence only. No independent21 review or completion claim.

Exact outcome, uncommitted paths, open owned app/root, reported test PIDs, cleanup limits and later-resume instructions: [checkpoint](../docs/qa/task21/implementation-evidence.md). [Environment recovery](../docs/qa/task21/environment-recovery.md) records the preserved public Cargo archive cache on D:. Leave all acceptance boxes unchecked. Do not resume Computer Use in the stopped turn or terminate the app as a bypass; retain owned resources for a later authorized resume. Tasks22–24 remain unexecuted.


Current acceptance boundary: all required feasible software-state cases have [independent closure](../docs/qa/task24/state-gaps/independent-review.md). Reviewed source109171a has clean local packaging and successful full hosted check/package evidence; postA QA-only additions preserve exact release bytes and have targeted checks/re-review. This task is blocked only by the named resource/tool-limited acceptance above, not by an unreported software TODO. Resume on a ready isolated Windows environment with effective networking disabled/no Rust and independent user recovery, then test actual OS lifecycle/logon and audible Narrator behavior using a working positive control. Existing source/adapter/AX tests do not pretend those runtime checks passed. The final integration update supplies the actual main SHA and attached CI result.
