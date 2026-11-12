# Task 01: Rust foundation, reference evidence and architecture

Status: in_progress
Priority: P0 foundation / screenshot-first
Dependencies: None
Requirements: PRD §§1–5, 7–10, 12–14

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Create the Windows-first Rust workspace, executable named Authenticator, Cargo.lock, module boundaries, local test support, formatting/lint configuration and Windows CI. Document reproducible build/run/test commands and required toolchain.
- [ ] Use a Rust UI framework; evaluate egui/eframe first with a small accessibility, resizable-window and Windows capture-integration spike. If it cannot meet the PRD, select another Rust UI framework and record evidence; do not substitute a JavaScript/web UI.
- [ ] Write docs/architecture.md, docs/security-design.md and docs/reference-ui.md. Record the PRD's settled defaults and format/version boundaries before implementing persistence.
- [ ] Inspect the installed reference app or available screenshots non-destructively if accessible. Record exact evidence paths and observed structure. Never inspect real secrets or change the reference vault. If screenshots/app are unavailable, use PRD §5 as the explicit fallback, record visual details as unverified, and proceed without questions.
- [ ] Record repository baseline: no application source, components, theme or DESIGN.md existed when tasks were authored. Do not invent reuse counts. Propose one shared Rust theme/components layer for all screens.
- [ ] Establish secret-safe errors, test-only data directories and .gitignore for real vaults, backups, captures, target output and credentials. Preserve these task files and PRD.

## Acceptance and verification

- [ ] A clean Windows checkout builds and launches an Authenticator window; formatting, linting and initial CI succeed.
- [ ] Architecture identifies UI/domain/storage/platform boundaries and an injectable clock; offline runtime has no network adapter.
- [ ] Framework spike proves keyboard focus, accessible widget labels, resizing and the planned capture overlay are feasible.
- [ ] Reference evidence and limitations are recorded honestly; security decisions specify encryption, passphrase recovery, atomic writes and key lifetimes before later tasks.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Implementation and independent re-review approved 2026-11-12; hosted CI pending. Full commands and sanitized evidence: docs/qa/task01/implementation-evidence.md. Implementer implement_01; distinct reviewer review_01 initially found per-frame drag origin and missing packaging choice, both fixed and independently rechecked. Six tests, fmt, clippy and Windows build pass; real pointer/key/AccessKit, drag/Escape/Cancel/resize/app-zoom checked. Root input failure traced to elevated launch; use helper normal launch. Secondary viewport accessibility remains a documented task07 native-adapter obligation. Physical DPI, Narrator, actual capture and reference pixel fidelity are not task01 passes. Parent is pushing reviewed foundation for initial Windows CI; final acceptance/status awaits that result. Commit reference follows in next checkpoint.

