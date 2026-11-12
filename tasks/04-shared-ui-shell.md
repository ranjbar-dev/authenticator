# Task 04: Reference-based Rust shell and shared controls

Status: done
Priority: P0 foundation / screenshot-first
Dependencies: 01–03
Requirements: PRD §§5, 7, 10; FR-8

## Objective and boundaries

Implement this task in the Rust Windows desktop app **Authenticator**, using the current PRD.md and the settled decisions in its section 12. English-only; all persisted user data encrypted locally; no app accounts, Google sign-in, sync or cloud storage. Do not use real credentials. The reference is **Authenticator App · 2FA**, inventoried in PRD.md §5; concrete screenshot paths are recorded by task 01 in docs/reference-ui.md. If unavailable, follow the inventory and record unverified visual details honestly.

## Implementation

### Execution preflight, 12 November 2026

Sized as a new shared shell under the authorized 24-task process. The application bound is Authenticator and src/ui plus imported libraries. One rendering candidate (FoundationApp in src/ui/mod.rs) was inspected; it is a diagnostic spike, not a production shell member. The example is a dispatcher, not another screen. Thus there are zero existing production-shell members and no counted screen conventions. PRD section5 and docs/reference-ui.md are the inventory reference; exact reference pixels remain unavailable. No DESIGN.md exists.

Actual reusable source is src/ui/theme.rs, applied by FoundationApp: named dark/background/text/muted/blue pairs, gap12 and control36. These few dimensions do not establish a complete spacing or typography scale. Task04 extends this one module with named roles and a coherent proposed scale, explicitly as new choices. Create the user-requested shared shell/controls under src/ui (shell/state/components modules as needed), not per-screen copies. Real domain/storage APIs are src/domain/records.rs and src/storage/mod.rs; UI must preserve them. Production starts locked. Disposable gallery/empty-vault previews belong only in examples/tests, with isolated storage.

The supplied Screen checklist below remains the page tree: outer holder → header/help → shared content/status → exactly three navigation tabs → native actions → labeled fields/dialogs → central theme → lock/focus/narrow behavior. Add whole typography-role bundles to the shared theme and document them so the calibration checks weight as well as size. Contrast uses the user's documented4.5:1 normal-text and3:1 meaningful-graphic/large-text targets; calculate actual pairs, including blue action foreground and focus. No reference measurements are invented. New theme/components are expressly authorized by this task; the existing plan is approved, with no additional approval stop.

Execution order stays task04 implementation → parent plants temporary outside-checkout shell copy → distinct calibration checker → different independent implementation reviewer → sequential fixes/re-review → acceptance. This follows the task's explicit before-dependent-screens calibration timing, because a production-shell reference does not exist before task04. Parent owns the calibration and statuses; implementation child must not spawn. Later task05 owns real setup/unlock/lifecycle; task07/08 own actual capture/import. Keep those obligations explicit rather than inventing usable flows or business records.

- [x] Implement the shared Rust theme, screen holder, three-tab navigation, toolbar, card, labeled field, button/icon button, dialog, error/progress/empty-state and status feedback primitives.
- [x] Build the compact dark window and blue accent using reference evidence or the explicitly documented PRD inventory fallback. Put shared colors, typography, spacing, radii and icon styles in one theme module; values are design choices until observed.
- [x] Provide semantic framework widgets/accessibility roles, keyboard navigation, visible focus and consistent active/disabled states. Never make active icons appear disabled.
- [x] Wire real application state/routes and lock gating; no hardcoded business records or placeholder screens claimed as complete. Disposable fixtures belong only in tests/demo builds.
- [x] Before dependent screens, calibrate UI checking with a temporary shell copy outside the checkout containing a deliberate visible/focus difference. Have a separate checker find it, record scope, discard it and use a different reviewer for implementation checks. Keep at most one child active; parent may plant the difference.

## Screen checklist

Planning snapshot, 12 November 2026: no existing application source, screen family, theme or shared components was present. These are PRD/reference-derived roles, not measured project conventions. Use the shared Rust theme and widgets created by task 04 (or create those primitives in task 04 itself); reread this checklist before editing. Refresh implementation paths from the actual code.

- [x] Outer window uses the shared screen holder and theme; the compact reference structure is preserved. — PRD/reference structure; shared implementation from task 04.
- [x] Header shows the screen title and relevant help/action controls without a profile/login affordance. — PRD/reference structure; shared implementation from task 04.
- [x] Content uses shared cards and status holders with deliberate empty/loading/error layouts. — PRD/reference structure; shared implementation from task 04.
- [x] Navigation shows exactly 2FA Codes, Backup Codes and Settings with clear active state. — PRD/reference structure; shared implementation from task 04.
- [x] Actions use native framework buttons and opaque, contrasting active icons. — PRD/reference structure; shared implementation from task 04.
- [x] Fields/dialogs use common labels, inline errors and logical focus handling. — PRD/reference structure; shared implementation from task 04.
- [x] Theme values are centralized; no unverified reference measurements are presented as facts. — PRD/reference structure; shared implementation from task 04.
- [x] Excluded reference controls are omitted; focus, keyboard and narrow-window behavior are checked. — PRD/reference structure; shared implementation from task 04.

Must not be copied from the reference: profile/sign-in, cloud sync/backup, language selector, extension/companion promotions, optional encryption switches or swipe-only desktop actions. Reuse shared field, button, dialog and feedback components; do not recreate their styling per page.

## Acceptance and verification

- [x] The three tabs and shared states render with real pointer and keyboard input; locked navigation exposes no credentials.
- [x] Window is readable at narrow, intermediate and desktop sizes and 100/150/200% scaling.
- [x] Independent checking detects the planted deviation; if it misses it, improve the check and repeat before UI sign-off.
- [x] No sign-in/profile, sync, language, companion upsell or telemetry controls exist.

## Handoff and completion

1. Read PRD.md, tasks/README.md, this entire task and relevant current source before editing. Confirm dependency evidence instead of trusting status labels alone.
2. Implement real behavior and meaningful tests. Run the targeted checks above; record exact commands and outcomes. Report ready for review to the orchestrator, without self-certifying review.
3. After the implementation child exits, the orchestrator starts a separate read-only reviewer. For UI work, include this checklist, actual screenshots, interaction evidence and ui-consistency:verifying when available. The reviewer must not be the implementer. The parent integrates fixes sequentially and requests re-review when needed.
4. Only after acceptance/review, set Status to done here, update the matching row in tasks/README.md and coverage evidence in tasks/coverage.md. The parent owns status changes and commits. Never delete the user's task files at completion.
5. Keep at most one child agent active; children must not spawn children, commit, push, change PRD scope or silently skip requirements. Return changed paths, checks, findings and remaining risks to the parent.

## Evidence

Accepted 2026-11-12 after separate calibrate_ui_shell and different review_04/re-review. Exact files/commands/state checks/sanitized screenshots and AX trees: docs/qa/task04/implementation-evidence.md and field-correction.md; shared rules docs/ui-foundation.md. Last full run35 ordinary tests plus explicit subprocess coverage, then2 focused layout/password tests after UI-only correction; fmt/Clippy/build pass. Independent reviewer inspected21 saved screenshots plus livepointer/key/tabs/dialogs/empty encrypted vault/Lock and actual metrics/masking tests. One P2 (TextEdit omitted BODYlineheight/tracking) fixed via shared full-role adapter with explicit passwordmask; independent re-review approved. Size matrix340/600/960 at100/150/200 app zoom passed, not physical WindowsDPI certification. Calibration detected TITLE400vs600 in full outsideworktree source; copiedexe launchShellExecute5 preventedlivecalib, no pixel-detection claim; copyremoved and real source restored. Full proof/untestedscope in docs/qa/task04/calibration.md. PhysicalDPI/Narrator/highcontrast/multimonitor/referencepixels and later workflows remain unverified/pending. CommitSHA in following checkpoint.

