# Task 03: Encrypted local persistence and key management

Status: done
Priority: P0 foundation / screenshot-first
Dependencies: 01, 02
Requirements: FR-7.1, FR-8.2; PRD §§8–10

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Implement a versioned authenticated-encrypted local vault containing all user data: metadata, seeds, recovery codes, organization, trash and preferences. Only public cryptographic envelope/version information may sit outside ciphertext.
- [x] Generate keys/salts/nonces with the OS CSPRNG. Use maintained cryptographic libraries, an Argon2id passphrase KDF with stored calibrated parameters, and authenticated encryption such as XChaCha20-Poly1305; verify current library guidance during implementation. Bound untrusted KDF parameters before allocation.
- [x] Use a random data-encryption key with passphrase-based wrapping so passphrase changes can rewrap keys. Keep backup encryption independent of device keys. No forgotten-passphrase reset/backdoor.
- [x] Store files under the per-user Windows application-data directory with appropriate ACLs. No plaintext database journal, cache or temporary serialization. Use atomic encrypted writes, integrity checks, schema migrations and rollback-safe error handling.
- [x] Implement serialized transactions and concurrent-instance protection. On failure/cancellation preserve the last valid vault; never silently initialize over a corrupt vault.
- [x] Keep decrypted state scoped to unlock sessions, redact Debug/error output and zeroize owned secret buffers where feasible. Expose explicit lock/unlock operations and a safe persistence API.

## Acceptance and verification

- [x] Restart preserves all entities/settings, while recognizable disposable metadata and seeds are absent from on-disk vault/temp/journal files.
- [x] Wrong passphrase, tampering, truncation, oversized envelope/KDF settings and future schema versions fail safely without changing the vault.
- [x] Inject interrupted/failed writes, migration failure and concurrent access; the last committed vault remains readable.
- [x] Verify passphrase change, key wrapping and lock cleanup; record limitations of memory clearing and OS protection.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Accepted 2026-11-12 after focused independent security/data-loss review_03 and correction re-review. No material finding; sole low-priority SHGetKnownFolderPath failure allocation leak corrected and rechecked. Exact files/format/AAD/bounds/calibration/fault tests: docs/storage-format.md and docs/qa/task03/implementation-evidence.md. Independent locked all-targets test run passed32 main tests; both subprocess helpers exercised (lock probe passes, interrupted writer exits73 as expected); fmt/Clippy/diff checks pass. Full native ACL/currentSID, actual sharing violation, ciphertext-only orphan and transactional failure coverage. Debug KDF probe1990ms at64MiB/3/1; release performance and UI lifecycle remain later tasks, no absolute power-loss/memory-erasure claim. Commit SHA recorded in following checkpoint.

