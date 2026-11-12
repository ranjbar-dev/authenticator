# Task 04: Reference-based Rust shell and shared controls

Status: todo
Priority: P0 foundation / screenshot-first
Dependencies: 01–03
Requirements: PRD §§5, 7, 10; FR-8

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

- [ ] Implement the shared Rust theme, screen holder, three-tab navigation, toolbar, card, labeled field, button/icon button, dialog, error/progress/empty-state and status feedback primitives.
- [ ] Build the compact dark window and blue accent using reference evidence or the explicitly documented PRD inventory fallback. Put shared colors, typography, spacing, radii and icon styles in one theme module; values are design choices until observed.
- [ ] Provide semantic framework widgets/accessibility roles, keyboard navigation, visible focus and consistent active/disabled states. Never make active icons appear disabled.
- [ ] Wire real application state/routes and lock gating; no hardcoded business records or placeholder screens claimed as complete. Disposable fixtures belong only in tests/demo builds.
- [ ] Before dependent screens, calibrate UI checking with a temporary shell copy outside the checkout containing a deliberate visible/focus difference. Have a separate checker find it, record scope, discard it and use a different reviewer for implementation checks. Keep at most one child active; parent may plant the difference.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [ ] Outer window uses the shared screen holder and theme; the compact reference structure is preserved. — PRD/reference structure; shared implementation from task 04.
- [ ] Header shows the screen title and relevant help/action controls without a profile/login affordance. — PRD/reference structure; shared implementation from task 04.
- [ ] Content uses shared cards and status holders with deliberate empty/loading/error layouts. — PRD/reference structure; shared implementation from task 04.
- [ ] Navigation shows exactly 2FA Codes, Backup Codes and Settings with clear active state. — PRD/reference structure; shared implementation from task 04.
- [ ] Actions use native framework buttons and opaque, contrasting active icons. — PRD/reference structure; shared implementation from task 04.
- [ ] Fields/dialogs use common labels, inline errors and logical focus handling. — PRD/reference structure; shared implementation from task 04.
- [ ] Theme values are centralized; no unverified reference measurements are presented as facts. — PRD/reference structure; shared implementation from task 04.
- [ ] Excluded reference controls are omitted; focus, keyboard and narrow-window behavior are checked. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [ ] The three tabs and shared states render with real pointer and keyboard input; locked navigation exposes no credentials.
- [ ] Window is readable at narrow, intermediate and desktop sizes and 100/150/200% scaling.
- [ ] Independent checking detects the planted deviation; if it misses it, improve the check and repeat before UI sign-off.
- [ ] No sign-in/profile, sync, language, companion upsell or telemetry controls exist.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Not started. On execution record changed files, test commands/results, sanitized UI evidence paths where applicable, independent reviewer result and commit reference. If blocked, retain incomplete status and record exact cause, attempted remedies and resumption steps.

