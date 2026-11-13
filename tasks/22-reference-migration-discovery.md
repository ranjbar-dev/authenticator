# Task 22: Reference-format discovery and truthful compatibility

Status: done - evidence-based unsupported reference-format decision independently reviewed
Priority: Required for complete v1
Dependencies: 11–13, 20, 21
Requirements: FR-6.4; PRD §§3, 5, 12

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Investigate reference .vault format only if public lawful documentation or disposable exports are already accessible. Do not access real secrets, enable sync, buy access or ask the user for material.
- [x] Record inspected evidence, format assumptions and unsupported cases in docs/migration-compatibility.md.
- [x] If evidence is sufficient to implement safely, add bounded parsing and disposable interoperability/round-trip tests before advertising support. Re-run security/import UI checks for any new path. — Not applicable: no sufficient evidence or converter; no compatibility advertised.
- [x] If evidence is unavailable or compatibility cannot be established, complete the discovery with the documented finding that reference .vault import is unsupported in v1. Keep native encrypted backup fully functional and show an honest unsupported-format error. Do not fabricate compatibility or block core delivery.

## Acceptance and verification

- [x] Discovery result is documented with actual evidence, including a clear unsupported decision when appropriate.
- [x] No reference compatibility claim appears without successful disposable interoperability tests.
- [x] Unsupported imports leave the current vault unchanged and offer native-backup guidance.
- [x] Any implemented converter receives independent parsing/security review and regression coverage. — No converter implemented; existing native rejection boundary independently reviewed.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Accepted after distinct security/data-loss review with no material findings. [Discovery/support boundary](../docs/migration-compatibility.md), [implementation evidence](../docs/qa/task22/implementation-evidence.md), [independent review](../docs/qa/task22/independent-review.md). Author negative worker/UI1 and independent-key/all-entity/24-OTP native1 passed; reviewer independently repeated negative1/210filtered19.41s and checked public package/primary-page evidence, source authentication/transaction boundary and actionable unsupported guidance. Generated unknown-format fixtures are not real reference exports. New public marketing-image availability is accurately recorded in docs/reference-ui.md without claiming native Windows pixel parity. No parser/converter/dependency/runtime/UI changes. Parent owns commit/push; reference follows in next checkpoint.

