# Task 20: Failure resilience and independent security audit

Status: todo
Priority: Required for complete v1
Dependencies: 01–19
Requirements: PRD §§8–11; all FR cross-cutting guarantees

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Perform a separate-agent code/security review of encryption, KDF bounds, nonce lifecycle, key wrapping, lock state, import transactions, screen capture and clipboard ownership. This is an agent review, not a claim of external cryptographic certification.
- [ ] Exercise fault injection for disk full/access denied/interrupted writes, corrupt vaults, migrations/rollback, concurrent instances, cancellation and lock during background operations. Remove panic paths reachable from untrusted input.
- [ ] Inspect runtime network behavior offline, plaintext filesystem artifacts, logs, crash output and Debug formatting with disposable sentinel values. Limit diagnostics and document OS crash-dump/memory limitations honestly.
- [ ] Review dependencies/licenses/advisories with current official tooling and guidance; remedy material findings. Verify source/build/test assets contain no production credentials.
- [ ] Produce docs/security-review.md with findings, fixes, residual limitations and reproducible evidence. Do not mark unresolved material vulnerabilities as accepted just to finish.

## Acceptance and verification

- [ ] No material security or data-loss finding remains unresolved; independent reviewer rechecks fixes.
- [ ] Fault-injection/adversarial tests preserve the last committed vault and never produce partial imports.
- [ ] Sentinel scan finds no user data in logs/temp/journals; runtime performs no required network operations.
- [ ] Security documentation distinguishes agent review, tested guarantees and endpoint/clipboard/backup/secure-erasure limitations.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

