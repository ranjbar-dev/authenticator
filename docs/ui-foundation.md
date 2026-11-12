# Shared Rust UI foundation

Review09 corrections preserve the screen hierarchy. Duplicate-code approval now
clears its contradictory global warning immediately; disabling backup input also
clears code-specific feedback, while unrelated metadata errors and input remain.
Changing code text still invalidates approval. Lock clears UI-owned buffers/
histories and state generation/query without waiting for an admitted filesystem
replace, and completion-slot checks are nonblocking. A held actual-save regression
verifies a credential-free locked AX frame before release. The active worker may
retain its vault/key/scratch until uninterruptible I/O/crypto returns; responsive
UI concealment does not imply forced key erasure or post-admission rollback.

Task09 keeps Account creation/editing in the existing ROOT holder, with shared
Back/Save/Cancel, a Keep editing/Discard guard, reference-order labeled fields,
deliberate seed replacement, and the same field tree as QR review. Collapsed
Advanced displays the actual parameter summary; expanded native labeled combo
boxes use complete BODY metrics for current/options text and ON_BLUE/BLUE for a
selected option. Tab/Space operate them, selection closes and restores focus, and
Escape closes a popup before considering form cancellation. A shared masked
multiline field supports optional creation-time recovery codes, duplicate review
and intentional reveal. Preview counts are cached on text changes, and changing
input clears stale validation feedback. No private visual theme was added.
The multiline input has an inner scroll viewport capped at six BODY rows plus
the shared8-point gap; focus/outer-scroll targets and AX bounds cover that visible
viewport. Text selection does not initiate content drag scrolling. Actual20-line
pointer/key/Save checks at340/600 and a40-line compact geometry regression cover
the long-input state; Save/Cancel remain near the preview instead of below every
line of content.

Concealed native password inputs mask both layout and full AX text. Intentional
Reveal uses visible layout and can expose AX TextRun children even though the
parent retains PasswordInput semantics; it must not be described as secret-free
accessibility in revealed mode. Native Copy/Cut do not emit clipboard text in
either mode; Cut may delete the selection. Undo histories are released every pass,
and lock/form exit clears widget state and owned sensitive buffers with documented
framework/OS limits. Task09's full-node/clipboard/actual multiline metrics tests
and actual340/600/960 pointer/key/screenshots are in its QA handoff. The native
Task07 GDI typography exception remains unchanged.

Task08 extends the same ROOT codes screen and scroll holder with explicit
candidate choice, labeled editable review and committed account/code cards.
Screenshot to Scan remains primary, with local image and manual entry directly
beside the add flow. All visible form/button/card/status roles use the existing
components and complete theme typography; no private stylesheet or theme is added.
Secret reveal is deliberate; default fields have native password semantics and
masked layout. Selection is explicit for multiple candidates, Save-as-new states
the duplicate policy, busy removes repeat submit/editor widgets, and errors retain
input. New review focus starts at the chooser/service field; error and imported
code cards scroll into view. Escape in contextual help preserves the underlying
review, while Cancel/Escape outside the dialog discards it and returns Screenshot
focus. Lock removes all credential nodes and focuses the passphrase field.

Actual task08 pointer/key screenshots cover chooser/review/settings/save at
340/600/960 logical widths, duplicate/invalid/write-error/unsupported states,
native image selection/cancel/lock and the new committed code. OTP values are
redacted from persisted result components. Distinct review_08 subsequently approved
the functional scope with independent live layouts/interactions and privacy checks;
see task08/independent-review.md. The native task07 19px GDI row
exception below is unchanged; physical DPI/Narrator/reference-pixel gates remain.

Task07 reuses the shared heading/button/status/progress roles for the prominent
Screenshot to Scan action, preparation/busy, result and retry/image/manual paths.
The native selector is an explicit task01 accessibility exception to ROOT egui
controls: geometry-only dim scrim, opaque blue/focus border, opaque SURFACE/TEXT
instruction panel and a real named native Cancel button with dark-on-blue text
and focus. It uses embedded Noto Sans400/ 600 and theme sizes/DIP conversion.
Escape cancels; Ctrl+L locks even while ROOT is hidden. Preparing and busy ROOT
omit metadata-bearing widgets before the hide command; restoration returns focus
to Screenshot after current lifecycle/generation processing. Lock restores the
locked form. Separate UIA observations expose instructions/Cancel; no complete
Narrator audit or secondary egui AccessKit support is claimed. Task07 evidence
covers 340/ 600/960 logical window widths and two 96 DPI monitor selections; physical
150/200% and mixed-DPI remain unverified. Editable review/Save and image/manual
editors are subsequent tasks, clearly described as unavailable in this build.

The selector has an explicit **native typography exception** to the ROOT bundle
table below. It shares the bundled family, role size, theme weight and colors,
but native STATIC and GDI DrawText keep their OS font-cell metrics instead of
forcing `BODY/LABEL.line_height = 20`. Native tracking is the GDI default zero;
this is not a general adapter for future nonzero ROOT tracking. Keeping the real
STATIC/Button controls preserves their platform accessibility and keyboard
semantics. No custom text control was introduced to change one pixel of leading.

The private GDI probe `native_font_metrics_and_control_fit` measured the actual
resolved fonts at a requested 96 DPI: BODY **Noto Sans / 400**, LABEL **Noto Sans
SemiBold / 600**, both requested `lfHeight = -14`, with cell height 19 pixels,
ascent 15, descent 4, internal leading 5 and external leading 0. Three instructions
measure 57 pixels high, widest 375, inside the actual theme allocation 528×72;
Cancel measures 102×19 inside 528×36 and is centered by native DrawText. The
instruction allocation is72 (the current central token), not an asserted 70.
These are real GetTextMetrics/GetTextFace/GetObject/GetTextExtent/DrawText
measurements, not claimed 20-pixel equivalence or physical 150/200% certification.
The adapter derives numeric weight and Win32 face from `theme::Weight`: the
SemiBold file's Win32 family is **Noto Sans SemiBold**. Asking the regular family
for 600 previously realized 700 on this host; the probe found and corrected it.

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
