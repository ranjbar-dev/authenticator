# Task 11: Authenticated encrypted export/import engine

Status: todo
Priority: Required for complete v1
Dependencies: 02, 03, 08–10
Requirements: FR-6; PRD §§8–11

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Define and document a versioned portable backup format independent of Windows/user/device-bound keys. Include all supported entities, trash, used states, memberships, preferences and metadata.
- [ ] Encrypt backups with a separately entered passphrase, unique salt/nonces, memory-hard KDF and authenticated encryption; never offer plaintext export.
- [ ] Implement user-selected destination, atomic encrypted writes, explicit overwrite intent and sanitized success/failure results.
- [ ] Implement bounded parsing, format/version/integrity/passphrase validation, preview counts/conflicts, explicit merge or replace and transactional commit. Wrong password/corruption must never appear as a valid empty backup.
- [ ] Define stable-ID/duplicate conflict policies including folder memberships, orphan references, used-code states and trash. Preserve existing entries unless an explicit preview choice replaces them.
- [ ] Prevent applying imported startup/security side effects without explicit UI intent; keep local lock policy unless deliberately selected in the preview.

## Acceptance and verification

- [ ] Round-trip all entities/parameters/states into a clean vault on a separate Windows-user/device context where available, with no original device keys required.
- [ ] Wrong password, tamper, truncation, oversized content/KDF, unsupported version and cancelled/failed write/import preserve existing data.
- [ ] Merge and replace tests cover collisions, duplicates, folders, trash, used codes and referential integrity.
- [ ] Interrupted export never exposes plaintext or destroys an existing backup; format and migration rules are documented.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

