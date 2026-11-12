# Task 04 implementation evidence

12 November 2026. Implementation is ready for the parent's planted-copy
calibration and a different independent reviewer; this document does not certify
either review. Task files/statuses/commits remain the parent's responsibility.

Scope: shared native Rust shell and primitives, real routes and repository lock
gating. Setup/unlock lifecycle, capture, real record lists/search, recovery and
settings flows remain their numbered tasks. Pending action dialogs say so and
do not save records. No reference-app vault, real credential or default user
vault was opened during UI testing. Only the new app's explicitly disposable
gallery was exercised. Reference pixel fidelity, physical Windows DPI changes,
multiple monitors, Narrator and high-contrast modes are unverified here.

## Source and checks

Changed implementation: `src/main.rs`, `src/ui/{mod,theme,state,components,shell}.rs`,
`examples/ui_gallery.rs`, `assets/fonts/{NotoSans-Regular.ttf,NotoSans-SemiBold.ttf,OFL.txt,README.md}`,
`docs/ui-foundation.md` and this QA directory. Parent preflight edits in task 04
and tasks/README.md were preserved.

Commands (repository root, PowerShell; all prefixed with RTK):

- `rtk cargo check --all-targets` — passed after correcting egui 0.37 API names.
- `rtk cargo test ui:: -- --nocapture` — 3 new state/contrast tests passed.
- `rtk cargo test --all-targets` — 36 passed, 2 ignored in RTK's combined output;
  no failed tests. Existing explicit subprocess tests retain their separate
  task 03 invocation/evidence; no claim that ignored probes ran by themselves.
- Final `rtk proxy cargo test --all-targets` — **35 ordinary tests passed**
  (20 library +3 foundation +12 domain), 2 subprocess-helper tests ignored in
  the ordinary library run; the concurrent-instance test also invoked its
  subprocess probe successfully. The interrupted-writer process was terminated
  by its owning test as intended. All target wrappers passed; no failures.
- `rtk proxy cargo test ui::theme::tests::actual_text_and_essential_graphic_pairs_meet_project_thresholds --lib -- --nocapture`
  — passed, exact ratios below.
- `rtk cargo clippy --all-targets -- -D warnings` — passed.
- `rtk cargo fmt --all` and `rtk cargo fmt --all -- --check` — passed.
- `rtk git diff --check` — passed.
- `rtk cargo build --bin Authenticator --example ui_gallery` — passed.

Initial expected development failures were fixed: generic egui 0.37 ID bounds,
renamed Panel/style APIs, then layout overflow caused by an unconstrained
`set_max_width(640)`. A rebuild while the gallery executable was open failed
with Windows LNK1104; closing the app and rebuilding passed. One transient
LNK1104 on a unit-test binary disappeared on immediate rerun (no such repository
test process was running). No default vault/reference process was terminated.

State checks prove all three routes preserve lock, locked account/recovery
intents are refused, fresh state locks an incoming unlocked repository, and a
real encrypted disposable session loses vault access/query/pending intent and
advances its generation on lock. Production has no frontend fixture records.

Contrast calculations, from the theme's actual RGB pairs:

| Pair | Ratio |
| --- | --- |
| Text / background, card, field, hover | 16.73, 14.32, 15.53, 11.41 |
| Secondary / card, hover | 8.12, 6.47 |
| Primary text and icon stroke / blue | 5.85 |
| Error / error holder | 8.69 |
| Success / card | 10.46 |
| Blue / card | 5.01 |
| Focus / background, card, field, hover | 9.49, 8.12, 8.80, 6.47 |

Control border pairs on the same dark holders also pass the test's 3:1 target.
Focus is drawn outside filled controls against those holders. Every proposed
typography bundle includes applied actual weight, size, family, line height,
tracking and case; see `docs/ui-foundation.md` for calibration source paths.

## Real interaction evidence

Built gallery launched unelevated through documented `sky.launch_app` from the
computer-use skill (`@oai/sky`), then selected from `sky.list_windows`. No shell
UI automation or synthetic element `.click()` was used. Coordinate clicks on
rendered screenshots and the helper's native indexed clicks were used; keys
were physical `sky.press_key`. Immediately refreshed observations supplied all
indexes/coordinates. Additional refreshes captured settled focus/resize states.

The first launch screenshot was captured before activation and showed the
occluding app. It was neither saved nor used as app evidence. Explicit
activation fixed capture; all committed images depict only the disposable new
app. JPEG screenshots retain their original bytes and use `.jpg` extensions.

Observed flows:

1. At 420 × 740 content size/100%, coordinate click Backup Codes `(210,259)`
   changes heading/selected tab while still showing the lock gate.
   `locked-backup-pointer.jpg` and `locked-420-100.{jpg,ax.txt}`.
2. Tab order from the root: Shell → Controls → Preview tools (gallery-only),
   Help → 2FA Codes → Backup Codes → Settings. Return on Settings changes its
   heading and selected state without unlocking. `tab-keyboard-focus.{jpg,ax.txt}`
   shows the visible border and AX tab item; production omits the gallery toolbar.
3. Coordinate Help click opens the named ROOT modal; Close receives focus.
   Tab stays within the one-control help modal. Escape closes it and returns
   focus to Help. An initial immediate focus request was lost while the prior
   modal layer was active; deferred shared restoration fixed it.
   `help-dialog.{jpg,ax.txt}`, `help-focus-restored.{jpg,ax.txt}`.
4. Controls: pointer Validate `(329,378)` with an empty field shows inline
   `Enter a preview label.` tied to the field description/invalid state.
   Click field `(184,299)`, physical `a`, Tab → Validate, Return displays a
   polite success status and no save. `controls-inline-error.{jpg,ax.txt}`,
   `controls-keyboard-success.{jpg,ax.txt}`. Error/success/info/progress/disabled
   and opaque active icon samples use the actual shared primitives.
5. Coordinate Open preview dialog `(318,509)` focuses its labeled Dialog label
   field. Tab focuses Close; Tab wraps to the field; Escape closes and returns
   focus to its opener. `controls-dialog-field.{jpg,ax.txt}` and
   `dialog-focus-restored.{jpg,ax.txt}`. Immediate AX snapshots sometimes lagged
   focus by a render pass; settled observations verified the named focused node.
6. Preview tools → Open disposable empty vault performs actual storage unlock
   only against the gallery's encrypted temporary empty vault. Shell shows real
   empty 2FA and Backup routes, not fabricated account records.
   `unlocked-empty-{codes,backups}.{jpg,ax.txt}`. Pointer Backup tab and Lock
   `(252,170)` remove the search field, empty-vault content and Lock action;
   locked AX contains no account metadata or codes.
   `lock-action-credentials-hidden.{jpg,ax.txt}`.

## Window sizes and app scaling

Window baselines were selected at 100% app zoom, then app zoom changed while
holding the physical window size. The helper screenshot bounds include native
frames: narrow **342 × 712**, intermediate **602 × 772**, desktop **962 × 832**.
The corresponding requested content sizes were 340 × 680, 600 × 740 and
960 × 800. These are app zoom checks, not physical Windows DPI certification.
The gallery toolbar is test-only and consumes extra height, especially at 200%.

| Baseline | 100% | 150% | 200% |
| --- | --- | --- | --- |
| Narrow | `locked-narrow-100.jpg` | `locked-narrow-150.jpg` | `locked-narrow-200.jpg` |
| Intermediate | `locked-intermediate-100.jpg` | `locked-intermediate-150.jpg` | `locked-intermediate-200.jpg` |
| Desktop | `locked-desktop-100.jpg` | `locked-desktop-150.jpg` | `locked-desktop-200.jpg` |

All have paired `.ax.txt` trees. At narrow high zoom, vertically stacked tabs
and scrollable content preserve readable labels. Real wheel input reached the
wrapped bottom action and complete description: `locked-narrow-150-bottom.jpg`
and `locked-narrow-200-bottom.jpg`. No horizontal truncation is accepted by the
holder. At intermediate150%, Backup Codes wraps to two lines within its native
button; it remains readable and reachable. At desktop100%, the 640-point content
cap intentionally preserves the compact layout rather than stretching cards.

An exploratory width selection while already at200% produced a larger native
window (commands are in current logical points). It was not saved or counted
as the intermediate-width matrix; zoom was reset to100%, width selected again,
and only the fixed-size results above were recorded. Native DPI, multiple
monitors and reference screenshot comparison remain task21 evidence gaps.

The parent's independent calibration/review must inspect these samples and the
full Screen checklist in task04, including whole typography bundles. This
implementation ran no planted-copy check and claims no independent sign-off.
Skill/source-read cost was not precisely counted; approximately20 project/doc
reads and12 skill/support reads preceded implementation. No conventions were
invented from the single diagnostic spike.

Final review harness: `target/debug/examples/ui_gallery.exe`, title
`Authenticator UI gallery`; last launched window ID104859714. Refresh selection
from `sky.list_windows` rather than treating this handle as durable. It starts
locked on its newly created disposable encrypted empty vault. The final small
AX-only refinement names the indeterminate progress primitive; its appearance
is unchanged from `controls-desktop.jpg`. Final build, fmt, Clippy and the full
regression above include that refinement.

## Parent acceptance

Separate calibrate_ui_shell detected the planted partial-bundle defect in the temporary full worktree; source-level proof only, live launch denied5, copy removed (calibration.md). Distinct review_04 independently inspected source,21 saved screenshots/live pointer/keyboard/modal/lock and targeted tests. Its one P2 field-layout finding was fixed and independently rechecked, including actual Galley metrics and masked PasswordInput behavior. Review approved task04 scope; parent accepted. Physical hardware/accessibility/reference limitations and later workflows remain pending. Implementation/evidence commit reference follows in next checkpoint.
