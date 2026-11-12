# Task 24: Final completeness audit, PRD reconciliation and main push

Status: todo
Priority: Required for complete v1
Dependencies: 01–23
Requirements: Entire PRD and tasks/coverage.md

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Audit every FR item, security/visual requirement, success criterion and task acceptance check against actual implementation/test evidence. Repair omissions and regressions; no placeholder, stub, skipped required check or TODO may be called complete.
- [ ] Reconcile PRD.md with implemented decisions, supported formats, capture/portability behavior, exact retention/lock defaults, migration findings and known platform limitations. Preserve mandatory scope and never narrow requirements merely to close tasks.
- [ ] Update tasks/README.md, every task status/evidence section, tasks/coverage.md and docs/completion-report.md. Differentiate implemented, independently reviewed, tested and externally unverified claims.
- [ ] Run final cargo fmt --check, cargo clippy with the agreed warning policy, cargo test, release build/package and affected real-UI journeys. Review git diff for secrets/unrelated changes.
- [ ] Commit coherent project changes on main and push to https://github.com/ranjbar-dev/authenticator without force-push or rewriting unrelated history. Verify origin/main equals local HEAD and working tree is clean.
- [ ] If infrastructure/auth/tool/hardware access truly prevents a requirement, leave its task blocked, persist exact evidence/resume steps and report incomplete honestly. Do not falsely announce completion or keep retrying an unchanged permanent failure forever.

## Acceptance and verification

- [ ] All numbered tasks are done only with evidence and independent review where required; every coverage row is verified.
- [ ] Final PRD matches actual behavior and preserves Rust, reference UI, English-only, encrypted-local-only and screenshot-first requirements.
- [ ] Required checks pass, packaged output is identifiable, no real secrets are committed and no material issue is unresolved.
- [ ] main is committed/pushed and remote SHA equals local HEAD; final report names checks, artifacts and genuine limitations.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

