# Task 23: Windows packaging, clean-install recovery and documentation

Status: in_progress; scoped package/native recovery reviewed; reproducibility and environment gates open
Priority: Required for complete v1
Dependencies: 01–22
Requirements: PRD §§1–3, 7–11, 13–14

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Produce a reproducible Windows release build and distributable package/installer without requiring paid signing credentials. Use app name Authenticator, correct version, local icon/assets and user-scoped data paths.
- [x] Document install/run/update/uninstall, Rust development, screenshot scan, manual/image entry, supported TOTP parameters, lock/recovery, backup codes, folders/trash and easy encrypted export/import in English.
- [x] Document that uninstall preserves user vaults unless separately deliberately removed; packaging/startup changes must not delete data or weaken ACLs. Do not ship test records or hardcoded business data.
- [ ] Smoke-test the packaged binary on a clean local profile/VM if available and restore an encrypted backup independently of original profile keys. Confirm offline runtime and startup toggle behavior.
- [ ] Add Windows CI release-build/package validation and record artifact location/checksum/license notices. Do not publish a GitHub release or upload user data as part of this task.

## Acceptance and verification

- [ ] A clean environment can build the documented package and launch Authenticator offline without Rust installed for end users.
- [x] Packaged screenshot scan, unlock, clipboard, export/import and restart persistence work with disposable credentials.
- [ ] Install/update/uninstall handling preserves vault data; recovery on another profile is verified or explicitly recorded as blocked evidence.
- [x] Docs accurately cover every shipped workflow, supported format and security limitation without invented claims.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Task24 independent review approved the scoped UI/package repairs and required restoring this original reproducibility requirement. The parent restored its wording and unchecked state; deterministic EXE/ZIP engineering and measurements remain feasible work. A repeatable command alone does not satisfy this checkbox. See [scoped review](../docs/qa/task24/independent-review.md) and [incomplete audit](../docs/completion-report.md). The earlier clean ebd artifact is identified in [clean-build.md](../docs/qa/task23/clean-build.md); current source still requires reviewed integration and fresh hosted package proof.

Implementation and actual packaged pointer/keyboard evidence: [implementation report](../docs/qa/task23/implementation-evidence.md). Distinct package/security/UI review and re-review: [review](../docs/qa/task23/independent-review.md). Parent repaired three archive-validator findings (reserved Windows device aliases, case-insensitive file/ancestor collisions, raw NUL filename normalization); independent 13 regressions, existing ZIP validation and diff check pass. Build/resource/dependency changes and public notices are reviewed. Source commit and clean-HEAD artifact follow in the next checkpoint to avoid circular attribution.

Actual production scan/review/save/HMAC/copy, native encrypted export, separate-key restore, ordinary Explorer launch, startup registration on/off and relaunch persistence passed. Three locked frames were inspected independently. Another Windows-user/VM, no-build-tools machine and effective networking-disabled runtime remain unverified. Exact-app firewall rules were present but reported CategoryDisabled enforcement; both owned rules were removed. No ready VM or other user session was available. Do not call same-user separate-key recovery a cross-profile pass. Resume on an isolated ready Windows VM/user with networking disabled; build from the clean commit, verify checksum, launch without Rust and repeat scan/copy/export/fresh-profile restore/restart, then obtain independent review. Actual logon, physical OS/DPI/accessibility and full matrix gates remain with their owning tasks.

