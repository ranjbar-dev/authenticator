# Task 06: Bounded local image and QR decoding

Status: done
Priority: P0 foundation / screenshot-first
Dependencies: 02, 03
Requirements: FR-2.2–4, FR-2A.3/5/6; PRD §8

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Implement a local QR service accepting in-memory capture pixels and PNG/JPEG image files. Sniff content safely, bound file size, pixel dimensions, decoded allocation and payload/count limits, and return structured review candidates.
- [x] Feed every candidate through the strict TOTP parser. Distinguish no QR, unsupported QR, malformed credential and decode failure. Handle multiple supported QR codes without silently importing all.
- [x] Do not save screenshots, raw QR payloads or original image files; retain only validated in-memory review data until explicit save. No network lookups or website/icon fetching.
- [x] Use deterministic disposable fixtures covering valid algorithms/periods, noisy/rotated QR, malformed images and oversized dimensions. Document supported formats and limits.

## Acceptance and verification

- [x] PNG/JPEG and in-memory fixtures yield the same validated candidates and parameters.
- [x] No-QR, mixed valid/invalid multiple codes, unsupported schemes and corrupt/oversized images return safe actionable results.
- [x] Fuzz/property or targeted adversarial parsing checks show bounded failure without panic/large uncontrolled allocation.
- [x] Filesystem/log inspection confirms no raw source or QR payload is persisted.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Accepted 2026-11-12 after distinct read-only review_06; no blocking parser/privacy/security findings. Exact APIs/limits/dependency sources/clearing limits in docs/architecture.md and docs/qa/task06/implementation-evidence.md. Independent15QR+15domain/foundation tests, lockedallfeatureClippy/fmt/diffchecks pass; inspected full60test/audit evidence.8MiB file,4096axis,4,194,304pixels,32MiBdecoded budget,2048payload,7candidates; saturation rejected before payloaddecode. PNG/JPEG generatedinmemory tests cover24settings,alpha/16bit/rotations/noise,mixed andmalformed,hostilegeometry/ancillarychunks,readgrowth,filesystemimmutability andredaction. Memorybudget isbest-effort, dependency-owned copies/error scratch notguaranteedcleared; no capture/UI/vault writes introduced. Actualcapture/cancel/lock/review/save remain07/08. Commitreferencefollowingcheckpoint.

