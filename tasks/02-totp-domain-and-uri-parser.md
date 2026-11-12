# Task 02: TOTP engine, data model and credential validation

Status: done
Priority: P0 foundation / screenshot-first
Dependencies: 01
Requirements: FR-1.2, FR-2.1–4; PRD §§9–11

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Implement typed vault/account/folder/membership/backup-code/preferences records with stable IDs, timestamps and deletion state. Keep service account identifiers distinct from app accounts.
- [x] Implement TOTP using maintained primitives/libraries, SHA-1/256/512, 6/8 digits and periods 15/30/60/120 seconds. Defaults are SHA-1, 6 digits, 30 seconds. Inject a clock; do not cache stale codes across period boundaries or resume.
- [x] Normalize Base32 with explicit allowed whitespace/padding rules and reject malformed or empty secrets. Validate required service name and optional website/account fields without leaking the secret.
- [x] Parse otpauth://totp locally, handling percent encoding, issuer/label precedence, defaults and supported parameters. Reject HOTP, malformed/ambiguous duplicate parameters, invalid algorithms/digits/periods and unsupported schemes with safe errors. Do not silently change supplied parameters.
- [x] Define meaningful duplicate detection without persisting a plaintext secret index. Define account editing rules that preserve stable identity.

## Acceptance and verification

- [x] RFC 6238 vectors for every supported algorithm and RFC 4226 primitive vectors pass; exposing HOTP accounts remains excluded.
- [x] Test 6/8 digits, all periods, leading zeros, exact boundaries, pre-epoch/overflow handling, timezone independence and simulated clock jumps.
- [x] Valid URI examples preserve parameters; malformed Base32/URIs/duplicate parameters cannot create invalid accounts.
- [x] Debug/error formatting and fixture names never expose real seeds; use published/disposable test vectors only.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Accepted 2026-11-12 after separate read-only review_02; no material findings. Exact implementation paths, parameters, bounds, official sources and commands/results: docs/qa/task02/implementation-evidence.md. Independently verified 20 all-target tests (12 integration plus8 library/foundation), all18 RFC6238 vectors,10 RFC4226 primitive counters,24settings, boundary/clock/overflow, strict adversarial URI/Base32 and aggregate redaction/relationships. Commands: rtk cargo test --locked --all-targets --target-dir target/task02-checks; rtk cargo fmt --all -- --check; rtk cargo clippy --locked --all-targets --all-features --target-dir target/task02-checks -- -D warnings; rtk git diff --check (all pass). Domain-only acceptance; storage and real UI/lifecycle remain later tasks. Commit reference in following checkpoint.

