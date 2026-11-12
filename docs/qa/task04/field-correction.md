# Task 04 field typography correction

12 November 2026. Scoped response to independent review_04's P2: the original
TextEdit used BODY's font alone, leaving its line height/tracking to the native
default layouter. The production shell, routes and lifecycle scope are unchanged.
Parent calibration evidence in `calibration.md` is preserved.

Changed source: `src/ui/theme.rs` adds `Typography::singleline_job`, applying
font/weight, line height and tracking as one bundle. `src/ui/components.rs`
routes plain and explicitly masked labeled fields through one native TextEdit
adapter. Its sizing/layout matches egui's add_sized wrapper; actual editing,
cursor positioning, selection and native password semantics stay in TextEdit.
`examples/ui_gallery.rs` adds a disposable masked component sample inside the
existing preview dialog, not an unlock/setup workflow.

The custom layouter masks secret text before it reaches font layout because
egui's default `mask_if_password` is bypassed by custom layouters. The explicit
secret variant also sets `.password(true)` for native accessibility/clipboard
behavior. The same Unicode-scalar count as egui keeps native cursor indexes
aligned with display bullets. No real secret/passphrase was entered.

Commands/results:

- `rtk cargo test field_tests --lib` — **2 passed**, 22 filtered out. Actual
  TextEdit output has a 20-point BODY row; a varied 28-point/2-point tracking
  bundle changes measured row height and rendered width. Trailing whitespace
  remains in the display job. The masked Unicode sample produces only bullets
  in the retained Galley, preserves character counts/input and emits native
  PasswordInput accessibility role without a raw value.
- `rtk cargo fmt --all` / `rtk cargo fmt --all -- --check` — passed.
- `rtk cargo clippy --all-targets -- -D warnings` — passed.
- `rtk cargo build --bin Authenticator --example ui_gallery` — passed after
  closing only the running disposable gallery (Windows locks open executables).
- `rtk git diff --check` — passed.

The first headless test attempt dropped unapplied font textures; the test
harness now explicitly clears its unused texture delta. The corrected tests
exercise actual rendered Galley metrics, not only copied token metadata. The
broader storage/KDF suite was not repeated for this UI-only correction.

Real UI verification used documented unelevated sky APIs, fresh observations,
physical coordinate clicks and physical key presses. At420×740 content size,
100% app zoom:

1. Pointer Controls `(115,83)`, Validate `(205,379)` with empty input: inline
   error remains correctly placed/labeled. `field-fix-inline-error.{jpg,ax.txt}`.
2. Pointer field `(153,300)` focuses Preview label. Physical `a`, `b`, then
   Shift+Left selects only `b` (AX selected_text `b` and visible highlight).
   `field-fix-selection.{jpg,ax.txt}`. Physical `c` replaces it to yield `ac`.
   Tab focuses Validate; Return produces success feedback.
   `field-fix-replacement-success.{jpg,ax.txt}`.
3. Pointer Open preview dialog `(200,509)` opens the named ROOT dialog and
   focuses Dialog label. Tab moves to Masked preview; physical `x` displays
   only `•`. AX focused value/document text also contain only `•`.
   `field-fix-masked-dialog.{jpg,ax.txt}`. This is inert disposable preview text.
4. Tab moves to Close; Tab wraps to Dialog label; Escape returns focus to
   Open preview dialog. `field-fix-dialog-focus-restored.{jpg,ax.txt}`.

Final gallery is open on Controls at420×740/100%, window ID159386958, executable
`target/debug/examples/ui_gallery.exe`. Refresh window selection before input.
Ready for the distinct reviewer's correction re-review; no independent sign-off
is claimed by this implementer, and no task status/commit was changed.
