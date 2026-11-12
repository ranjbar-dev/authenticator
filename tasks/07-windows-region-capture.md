# Task 07: Windows screen-region capture overlay

Status: in_progress — implementation independently reviewed; hardware QA open
Priority: P0 foundation / screenshot-first
Dependencies: 03–06
Requirements: FR-2A.1–3/5/6; PRD §§7, 10–11

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [x] Implement a Windows capture adapter and region overlay, initiated only by Screenshot to Scan. Hide the main window if needed and restore it on every completion/cancel/failure path.
- [x] Support monitor enumeration, negative virtual-desktop coordinates, mixed DPI, scaling, drag in either direction and clipped/empty selections. Document coordinate transformations.
- [x] Capture only the selected region into memory; feed pixels to the QR service. No clipboard screenshot or disk screenshot is needed.
- [x] Use real pointer capture for drag selection, release it reliably, and handle Escape, monitor disconnection, permission denial, OS session lock and app shutdown. Prevent capture when locked.
- [x] Keep OS failures distinct from decoder failures and offer retry/image/manual paths through safe events to the UI.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [x] Capture begins only from the prominently labeled Screenshot to Scan action. — PRD/reference structure; shared implementation from task 04.
- [x] Overlay frames screen content with an obvious selected region and readable brief instructions. — PRD/reference structure; shared implementation from task 04.
- [x] Selection bounds track real pointer movement at each tested DPI. — PRD/reference structure; shared implementation from task 04.
- [x] Escape and visible cancellation return to the previous app state. — PRD/reference structure; shared implementation from task 04.
- [x] Capture has a clear busy state and prevents duplicate invocations. — PRD/reference structure; shared implementation from task 04.
- [x] Failure feedback offers retry, image import and manual entry. — PRD/reference structure; shared implementation from task 04.
- [x] Lock/session changes cancel and conceal capture/review state. — PRD/reference structure; shared implementation from task 04.
- [x] Overlay focus and pointer capture are released on every exit path; shared theme values apply to app-owned controls. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [x] Select a QR in another window using actual pointer input; selected pixels match the region.
- [ ] Exercise multiple monitors (including negative coordinates), 100/150/200% scale and mixed DPI where available; record hardware coverage honestly and use adapter tests for unavailable layouts.
- [x] Escape/empty selection/failure/lock restore the app, release capture and preserve the vault.
- [x] No unsolicited capture, retained image, secret logs or blocked pointer interaction remains.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Reviewed implementation and exact checks are recorded in [implementation-evidence.md](../docs/qa/task07/implementation-evidence.md); sanitized result/panel screenshots and accessibility trees are in the same directory. The parent inspected the source and representative component screenshots. Distinct review_07 approved the native capture, resource ownership, privacy and tested UI scope; focused re-review independently passed the font metric test and all 12 platform tests after correcting the realized semibold family. Checker calibration is recorded separately in [calibration.md](../docs/qa/task07/calibration.md).

Real pointer capture of public disposable QR fixtures succeeded on primary and negative-coordinate monitors, including reverse drag. Native Cancel/Escape, empty selection, no-QR, private WTS-message lock and restored focus were exercised. The full pre-final suite passed 69 ordinary tests; later preparing-frame accessibility/privacy and publication-race tests plus focused platform tests passed. See the evidence for exact command boundaries; no full-suite result is invented after the final font-only adjustment.

Physical 150/200% and mixed DPI, right-monitor capture, actual monitor removal/permission denial/OS session lock/hardware suspend, Narrator and reference pixel fidelity remain unverified. Task08 subsequently supplied working image/manual fallback, editable batch review, encrypted save/current code and independent lock/privacy review. Full manual editing remains task09; the capture hardware acceptance stays open. Commit reference is recorded in the following checkpoint.

