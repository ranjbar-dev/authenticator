# Task 11: Authenticated encrypted export/import engine

Status: done - engine approved after distinct security/data-loss review and focused re-review
Priority: Required for complete v1
Dependencies: 02, 03, 08–10
Requirements: FR-6; PRD §§8–11

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Define and document a versioned portable backup format independent of Windows/user/device-bound keys. Include all supported entities, trash, used states, memberships, preferences and metadata.
- [x] Encrypt backups with a separately entered passphrase, unique salt/nonces, memory-hard KDF and authenticated encryption; never offer plaintext export.
- [x] Implement user-selected destination, atomic encrypted writes, explicit overwrite intent and sanitized success/failure results.
- [x] Implement bounded parsing, format/version/integrity/passphrase validation, preview counts/conflicts, explicit merge or replace and transactional commit. Wrong password/corruption must never appear as a valid empty backup.
- [x] Define stable-ID/duplicate conflict policies including folder memberships, orphan references, used-code states and trash. Preserve existing entries unless an explicit preview choice replaces them.
- [x] Prevent applying imported startup/security side effects without explicit UI intent; keep local lock policy unless deliberately selected in the preview.

## Acceptance and verification

- [x] Round-trip all entities/parameters/states into a clean vault on a separate Windows-user/device context where available, with no original device keys required.
- [x] Wrong password, tamper, truncation, oversized content/KDF, unsupported version and cancelled/failed write/import preserve existing data.
- [x] Merge and replace tests cover collisions, duplicates, folders, trash, used codes and referential integrity.
- [x] Interrupted export never exposes plaintext or destroys an existing backup; format and migration rules are documented.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Engine accepted after distinct review_11 and focused re-review resolved P2 repeated
folder duplication following a cross-type ID collision. Source/path/key independence,
bounded authenticated parsing, transactional merge/replace, graph remapping, used/
trash state, local startup/security defaults and both temporary cleanup fixes reviewed.

- Normative format/policy: [portable-backup-format.md](../docs/portable-backup-format.md).
- Changed paths/author command and fault matrix: [implementation evidence](../docs/qa/task11/implementation-evidence.md).
- Findings, correction, independent checks and limits: [review report](../docs/qa/task11/independent-review.md).
- Final parent all-target/all-feature suite118 passed,4 subprocess helpers ignored
  directly but exercised by parents,14 suites75.08s. Strict Clippy/fmt/diff and
  production bin+lib build passed.
- Independent backup12/storage21/timestamp1 plus2 external probes, followed by3
  focused re-review tests passed. Process interruption preserved prior backup and
  left ciphertext only. No default/reference data or runtime network was used.
- Clean independently keyed/passphrased destination and all entities/24 OTP variants
  round-trip. Actual separate Windows user/device context was not exercised; the
  task specifies that context where available. No physical/removable/power-loss
  guarantee is substituted for source/controlled process evidence.
- Standalone all-target build encountered LNK1104 opening the existing lifecycle
  example artifact; unrelated preview was untouched. All-target test/lint and
  production build pass; complete isolated packaging checks remain23.

Tasks12/13 own real native file selection, export passphrase confirmation, user
preview/choice/confirmation and asynchronous session/lock integration. Those user
flows are not claimed by this engine acceptance. Commit reference follows in the
parent checkpoint.
