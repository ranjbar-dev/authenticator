# Task 20: Failure resilience and independent security audit

Status: blocked — effective isolated offline runtime inspection unavailable
Priority: Required for complete v1
Dependencies: 01–19
Requirements: PRD §§8–11; all FR cross-cutting guarantees

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Perform a separate-agent code/security review of encryption, KDF bounds, nonce lifecycle, key wrapping, lock state, import transactions, screen capture and clipboard ownership. This is an agent review, not a claim of external cryptographic certification.
- [x] Exercise fault injection for disk full/access denied/interrupted writes, corrupt vaults, migrations/rollback, concurrent instances, cancellation and lock during background operations. Remove panic paths reachable from untrusted input.
- [ ] Inspect runtime network behavior offline, plaintext filesystem artifacts, logs, crash output and Debug formatting with disposable sentinel values. Limit diagnostics and document OS crash-dump/memory limitations honestly.
- [x] Review dependencies/licenses/advisories with current official tooling and guidance; remedy material findings. Verify source/build/test assets contain no production credentials.
- [x] Produce docs/security-review.md with findings, fixes, residual limitations and reproducible evidence. Do not mark unresolved material vulnerabilities as accepted just to finish.

## Acceptance and verification

- [x] No material security or data-loss finding remains unresolved; independent reviewer rechecks fixes.
- [x] Fault-injection/adversarial tests preserve the last committed vault and never produce partial imports.
- [ ] Sentinel scan finds no user data in logs/temp/journals; runtime performs no required network operations.
- [x] Security documentation distinguishes agent review, tested guarantees and endpoint/clipboard/backup/secure-erasure limitations.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Limited source approval by distinct review_20, no material source security/data-loss findings. Minor exact-command transcription corrected to `--format-version 1`; reviewer independently executed that correct command. No external certification, native/offline-machine or production-readiness claim.

- [Current audit report](../docs/security-review.md).
- [Author paths/commands/57 unique focused cases/cleanup](../docs/qa/task20/implementation-evidence.md).
- [Independent source review/25 focused cases/core probe/dependency verification](../docs/qa/task20/independent-review.md).

Release demo entry is debug-only; bounded plaintext conversions avoid avoidable reallocations; explicit StorageFull rollback and attested panic-output checks pass. Independent core restore/sentinel probe passed with62 own-PID TCP/UDP samples showing0 observed entries; adapters stayed enabled and sampling can miss transients. Cargo audit found0 known matches and3 unsuppressed Unix-only maintenance notices; Windows normal/build license inventory185entries independently matched. Actual notice distribution remains23.

Current-source formatting/strict lint/release configuration/prod-bin+lib+own-example build and targeted tests pass; new hosted full suite is required. Native screenshots/input0, interactive Copy→Changed cause unknown, disconnected-machine/full native/physical/Narrator/OS-lock/suspend/logon/package checks remain OPEN. Checked boxes describe bounded source/fault/probe evidence, not those missing gates. Continue21–24 and record commit/hosted result in parent checkpoint.

[Hosted CI43397135687](https://github.com/ranjbar-dev/authenticator/actions/runs/43397135687) subsequently SUCCESS at4e0f715:205library+1main+18other ordinary tests,0failures,5direct helper entries exercised by parents; format/lint/appbuild pass. [Exact follow-up](../docs/qa/task20/hosted-follow-up.md). Hosted source gate closes; native/offline and actual packaged behavior remain separate.


Current acceptance boundary: all required feasible software-state cases have [independent closure](../docs/qa/task24/state-gaps/independent-review.md). Reviewed source109171a has clean local packaging and successful full hosted check/package evidence; postA QA-only additions preserve exact release bytes and have targeted checks/re-review. This task is blocked only by the named resource/tool-limited acceptance above, not by an unreported software TODO. Resume on a ready isolated Windows environment with effective networking disabled/no Rust and independent user recovery, then test actual OS lifecycle/logon and audible Narrator behavior using a working positive control. Existing source/adapter/AX tests do not pretend those runtime checks passed. The final integration update supplies the actual main SHA and attached CI result.
