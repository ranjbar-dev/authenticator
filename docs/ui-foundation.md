# Shared Rust UI foundation

Task05 connects the ROOT locked forms to real local create/unlock and reuses
shared card, field, button, heading, feedback and typography roles. Setup has
masked confirmation and adjacent passphrase-loss/independent-backup guidance;
unlock errors retain retry without offering replacement. Busy renders one
cancellable action and removes submit fields. New focus scrolls into the shared
holder; Cancel/search/form receive initial focus at their lifecycle transitions.
Ctrl+L locks, Escape cancels passphrase work, and help documents these shortcuts.
Secret editors now discard undo history after every pass; form exit/lock clears
the complete widget data map as well as zeroizing the owned strings. ROOT native
password semantics and pre-layout bullets from task04 remain in force. Actual
task05 UI evidence and current limitations are in `docs/qa/task05/`.

Task 04 uses PRD section 5's inventory fallback: compact dark holders, blue
actions, cards, local help and exactly **2FA Codes / Backup Codes / Settings**.
The installed reference vault was not opened. Colors, dimensions and fonts below
are proposed implementation choices, not observed reference measurements.

`src/ui/shell.rs` owns the one shared scroll holder and header, navigation and
content regions. It constrains content to the smaller of the available width
and 640 logical points. At small logical widths navigation stacks rather than
clipping its labels; content remains vertically scrollable at high app zoom.
There are zero preceding production shell members, so these are new shared
choices rather than counted conventions. The original `FoundationApp` and
`examples/framework_spike.rs` remain explicitly diagnostic.

`src/ui/theme.rs` is the only source for colors, spacing, radii, typography and
icon/focus strokes. A proposed four-point rhythm gives 8-point small gaps,
12-point regular gaps, 16-point padding and 36-point minimum controls. Text
uses the following complete bundles, all with zero tracking and as-written
English case except the code role's one-point tracking:

| Role | Font / actual weight | Size / line height (logical points) | Use |
| --- | --- | --- | --- |
| TITLE | Noto Sans / SemiBold 600 | 24 / 32 | Main screen title, heading level 1 |
| SECTION | Noto Sans / SemiBold 600 | 18 / 24 | Card/dialog title, heading level 2 |
| BODY | Noto Sans / Regular 400 | 14 / 20 | Explanations and status |
| LABEL | Noto Sans / SemiBold 600 | 14 / 20 | Fields, tabs and actions |
| CAPTION | Noto Sans / Regular 400 | 13 / 20 | Secondary instructions |
| CODE | egui bundled Monospace / inherited regular | 28 / 36 | Future code display |

`Typography::text` applies size, actual named font family/weight, line height and
tracking together. `Typography::font` selects the actual regular/semibold font
bytes; `RichText::strong` is not used as a font-weight substitute. The unmodified
official Noto font files and OFL license/provenance are in `assets/fonts/`.
Font data is embedded; there is no runtime font download. The code role uses
egui's default bundled monospace fallback, not a newly claimed measured font.

`src/ui/components.rs` provides heading/body/secondary text, toolbar, card,
native framework buttons, named icon buttons, opaque vector icon tiles,
semantic selected tabs, associated labeled text fields and inline descriptions,
polite status regions, empty/progress states and in-root modal dialogs. Keep
future pages on these helpers rather than setting one piece of a typography
bundle or making private copies. Icons are decorative inside text-backed empty
states; icon-only actions have explicit accessible names. Active icon tiles use
opaque dark strokes on solid blue. Keyboard focus uses a quiet outer border;
its visible portion is against the holder, not the blue button fill.

Modal content is in ROOT because eframe's secondary-viewport AccessKit adapter
was not initialized in task 01's tested backend. `Dialog` sets its native
framework dialog name, initial control focus and focus containment. The root
calls `restore_dialog_focus` before rendering to return focus after the previous
modal layer clears. Escape, backdrop click and explicit Close use that same
path. Long dialog content scrolls inside the current root.

Editable fields apply the full BODY bundle through the shared native TextEdit
adapter and `Typography::singleline_job`: actual 20-point row height, named
regular font/weight and role tracking, while keeping native cursor/selection
behavior. Use `labeled_field` for ordinary text and the explicit
`labeled_secret_field` for secret text. The secret variant sets native
`password(true)` semantics and masks the display string before custom font
layout (one bullet per Unicode scalar). Custom egui layouters otherwise bypass
the framework's default masking. Neither the retained display Galley nor its
accessibility value contains the raw secret. This is a display boundary, not
a guarantee of clearing all framework/OS editing-buffer copies.

`src/ui/state.rs` owns the actual `VaultStore`. A new state owner always locks
the repository. Routes do not unlock it. Locked rendering does not request any
account metadata, counts, names or codes. Add-account/recovery intents require
an unlocked repository; the lifecycle intent may be opened while locked.
`AuthenticatorApp::lock` clears the pending dialog/status, while `AppState::lock`
drops the repository session, zeroizes the query, clears pending intent and
advances a generation token for later asynchronous integration.

Task 05 must connect setup/unlock and lifecycle behavior through this owner.
Tasks 07–10 and 14–19 supply capture, entry, lists and settings. Current intent
handlers explicitly say these workflows are not available and do not pretend
to add a record. Search input is retained for those later list modules; the
shared shell does not claim to implement a complete search or credential list.
No sample business records exist in production. The separate UI gallery creates
an encrypted empty vault in a disposable temporary directory and never opens
the default user vault.

Normal text pair target: 4.5:1; meaningful graphic/focus target: 3:1, from the
user's documented preferences. The theme test calculates actual pairs. Current
normal text pairs span 5.01:1 (blue on card) to 16.73:1 (body on background);
primary dark-on-blue text/strokes are 5.85:1. The focus border on its actual
dark holders ranges from 6.47:1 to 9.49:1. Disabled controls are separately dimmed
by the framework. These calculations are not a claim of complete WCAG or
screen-reader conformance; see the task 04 interaction/visual evidence.
