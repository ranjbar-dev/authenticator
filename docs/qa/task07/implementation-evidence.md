# Task07 implementation evidence

Implementation handoff, 12 November 2026; parent review update below. Native/security/UI
implementation review has passed within the evidenced scope; hardware/integration acceptance remains open.
Reference pixels remain unavailable; PRD inventory and the task07 screen checklist
are the reference. The installed reference vault was not opened.

## Scope and APIs

`src/platform/capture.rs` now owns checked physical `Point`/`Region`, safe closed
capture errors, `display_bounds`, `CaptureTask` and redacted `ScanReview` outcomes.
`capture/windows.rs` owns PMv2 thread context, monitor snapshots, geometry-only
selection windows, native named instructions/Cancel, SetCapture, selected-RECT
GDI/DIB capture and handle cleanup. `LifecycleWatch` observes an epoch independent
of the main UI's consumable notification flag. `AppState` owns one capture task,
generation-gated review, safe `CaptureEvent`, and `ScanAlternative::{Image,Manual}`
handoffs; `take_scan_review` transfers drafts to task08 without importing.

The prominent **Screenshot to Scan** action is the only product capture trigger.
It renders a nonsensitive preparation frame/AccessKit update before hiding ROOT.
Preparing/busy ROOT omits query, search, account counts and review widgets and
clears widget histories. Cancellation, generation changes, lock and shutdown
discard queued results. The native timer observes cancellation/native lock/idle
without needing a visible main UI frame. ROOT restores after lifecycle/result
processing; the native thread never reveals a previous unlocked framebuffer.
After selected pixels are acquired, ROOT may show cancellable local scan progress.

Task08 still owns editable candidate choice/review, explicit atomic Save and the
current-code journey. Image/manual alternatives queue typed intents and state
plainly that their editors are not implemented. No fake successful import exists.

## Coordinate and memory contract

* Selection thread alone enters `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2`,
  restoring its old context on exit. Process/user DPI settings are unchanged.
* Monitor rectangles, virtual desktop, event coordinates and BitBlt source RECT
  are signed physical pixels. No main egui app zoom or single monitor scale is
  multiplied into the rectangle. Negative origins are preserved.
* Signed WM_MOUSE button/move **event** client coordinates pass through PMv2
  `ClientToScreen`. `GetCursorPos` is used only for initial panel placement;
  sampling its latest position at queued button-down can collapse a fast real
  drag to its final point. Real SendInput testing found and corrected that bug.
* Native panel dimensions/type roles convert logical pixels with `round(dip ×
  GetDpiForWindow / 96)`. The opaque panel stays on its initiating monitor. DPI
  or display-change messages cancel rather than reinterpret an active drag.
* Normalize either drag direction, clip to virtual bounds, reject empty and
  checked-coordinate overflow. Virtual desktops exceeding 32,767 pixels on an
  axis fail closed because WM_MOUSE client fields are signed 16-bit. This is a
  documented adapter limit, distinct from the much tighter selected-image cap.
* Before any bitmap/owned pixel allocation: nonzero sides <=4,096, checked total
  <=4,194,304 pixels, checked four-byte size <=16,777,216 bytes. A full 3840×2160
  screen exceeds this total and is rejected; select around the QR instead.
* No full-desktop background capture exists. The dim scrim and opaque border
  paint geometry only. Hide/destroy all overlay HWNDs, call `DwmFlush`, recheck
  current monitor snapshot/cancellation, then BitBlt only the approved rectangle.
  DwmFlush covers the caller's queued changes, not a guarantee about arbitrary
  other applications' rendering/content changes.
* DIB is top-down, 32-bit packed BGRA. Normalize every reserved alpha byte to255
  before `decode_pixels`, then consume the owned `Zeroizing<Vec<u8>>`. Flush and
  zeroize mapped DIB bits before deleting the bitmap; restore selected GDI
  objects, DeleteDC/ReleaseDC, destroy HWNDs and release pointer capture on exit.
  GDI/driver/compositor, allocator/compiler/OS and decoder internal copies are
  outside complete erasure guarantees. Decode cancellation is cooperative and
  bounded by task06 limits, not instantaneous interruption or a hard deadline.

## Checks

Commands use the required RTK wrapper. Observed on Windows/MSVC Rust 1.98.

| Command | Observed result |
| --- | --- |
| `rtk cargo check --all-targets` | pass |
| `rtk cargo clippy --all-targets -- -D warnings` | pass after fixing formatting lint in disposable fixture |
| `rtk cargo fmt --all` | applied; final check recorded in handoff |
| `rtk cargo test --all-targets -- --test-threads=1` | 69 passed, 2 intentionally ignored child entry points, 117.65s, before the additional preparation-frame privacy test |
| `rtk cargo test platform:: -- --test-threads=1` | final focused run:11 passed, 0 failed, 0.24s |
| `rtk cargo test cancellation_wins_before_or_after_secret_bearing_publication -- --test-threads=1` | pass; 16 real producer/canceller races, validated secret-bearing public fixture reviews |
| `rtk cargo test preparing_accessibility_frame_precedes_hide_and_lock_clears_pending_capture -- --test-threads=1` | pass after adding required egui texture-delta clearing in the test; final run 5.97s |
| `rtk proxy cargo test native_layout_probe_geometry_only -- --nocapture` | pass; public monitor geometry below, no pixels captured |
| `rtk cargo build --example region_capture --example qr_fixture` | pass; subsequent final source checks recorded in handoff |

Tests cover reversed/clipped/empty/overflow regions, exact image limits, invalid
owner/no-allocation rejection, consumed/pending native epochs, idle deadline,
queued and concurrently published review cancellation, locked state/generation,
duplicate begin/alternative gating, pointer release on private native
WM_CLOSE/CANCELMODE/DISPLAYCHANGE/DPICHANGED and actual private WTS notification
through the hidden timer. Tests never broadcast session events, lock the desktop,
suspend hardware, change settings or capture unrelated windows.

The new privacy test establishes that a sentinel query exists in the initial
AccessKit tree, then disappears with search/account widgets in a complete
preparation frame before the first `Visible(false)` command. Hidden/busy output
also excludes it; lock clears query, pending capture and review. Its first test
attempt mishandled egui's unapplied texture deltas and aborted on their debug Drop
assertion; clearing test-only deltas corrected the harness. No native memory fault
was involved and the passing rerun is retained as the result.

## Real pointer and keyboard observations

Only argument-free locally compiled tools were helper-launched at normal
integrity using the computer-use skill's `@oai/sky` APIs. `region_capture` creates
an actual encrypted, empty disposable TempDir vault and explicitly unlocks that
test profile. `qr_fixture` renders a generated published RFC seed only, in another
process; no source QR/image file exists. Production/default/reference vaults and
real credentials were never opened.

Native geometry enumeration observed virtual `(-1920,0)6400×1440` and monitors:

| Physical monitor rectangle | Actual coverage |
| --- | --- |
| `(-1920,209)1920×1080` | actual generated QR selection, reverse direction; fixture reports DPI96 |
| `(0,0)2560×1440` | actual generated QR selection; fixture reports DPI96 |
| `(2560,200)1920×1080` | enumerated only; no claimed QR or physical DPI test |

The helper normalizes a 6400-wide overlay snapshot to 2560 pixels; screenshotId
must identify the overlay snapshot rather than its 16×16 hidden-owner placeholder.
Tool coordinates were converted by the observed 2.5 horizontal normalization.
Initial attempts used the wrong helper coordinate frame (empty/NoQr); buffers
were discarded and no source/capture image persisted. These are recorded failures,
not physical mixed-DPI passes. The product adapter uses physical event pixels.

* Primary fixture area `[8,73..440,505]`; actual pointer drag selected
  `(7,73)433×432`; decoder returned 1 supported / 1 result. ROOT restored and
  Screenshot button regained focus. `scan-result.jpg` and matching accessibility
  text contain only safe counts/geometry and the unchanged empty vault.
* Negative monitor fixture `[-1824,411..-1392,843]`; reverse drag selected
  `(-1826,411)433×432`; decoder returned 1 supported / 1 result and restored focus.
  `negative-monitor-result.jpg` and accessibility text contain no QR/seed/pixels.
* Real pointer native Cancel returned **Selection cancelled**, restored ROOT and
  focused Screenshot. Real Escape did the same (`escape-result.jpg`).
* Real single pointer click in the blank fixture returned actionable empty-region
  feedback with retry/image/manual (`empty-selection.jpg`).
* UIA exposed named native instructions and actual **Cancel capture** button,
  with initial button focus. `overlay-instructions.jpg` is only the opaque 560×160
  app-owned panel over a blank source; `overlay-accessibility.txt` records names.
  No screenshot of a QR source, raw selected pixels or full user desktop is saved.
* `narrow-start.jpg` (340 logical width), `intermediate-start.jpg` (600) and
  `desktop-start.jpg` (960) cover actual shared shell/start controls. Narrow content
  scrolls and navigation stacks. These are window-size checks at 96 DPI, not150/200%
  physical scaling tests. The native panel screenshot is a readable component.

Final binary after preparation/AX staging: a blank fixture drag selected
`(-1826,411)433×432` and returned **No QR code was found** (`no-qr-result.jpg`).
Actual pointer clicks on Import from Photos and Manually Input produced the
honest unavailable-stage messages and typed safe intents; no file/editor/save
was invoked (`alternative-intent.jpg`). A subsequent real reverse drag of the
public QR at the same bounds returned 1 supported / 1 result, restored ROOT and
Screenshot focus (`final-scan-result.jpg`, `final-scan-accessibility.txt`).

The isolated preview's **Arm private WTS lock for next scan** button was armed
and observed in UIA, then Screenshot was clicked. The harness schedules a message
only after the real capture task starts, to its own registered lifecycle HWND
two seconds later. ROOT restored to **Unlock vault**, with the passphrase focused;
scan results, query and region geometry were absent (`native-lock-result.jpg`,
`native-lock-accessibility.txt`). This exercises hidden selection/native epoch
cancellation without locking the user's desktop. Actual OS delivery remains open.
Both task07 fixture/preview windows were then closed with real Alt+F4; a fresh
window inventory confirmed neither remained. The earlier task05 preview was left
alone. No Narrator/full screen-reader/offline-network/hardware acceptance is
implied by UIA or these interactions.

Final source checks after the native error/geometry cleanup:
`rtk cargo build --bin Authenticator --example region_capture --example qr_fixture`
passed, `rtk cargo clippy --all-targets -- -D warnings` passed, and
`rtk cargo test platform:: -- --test-threads=1` passed 11 tests in 0.25s.
`rtk cargo fmt --all -- --check` and `rtk git diff --check` are the final formatting
checks. No further full KDF suite was repeated for the preparation-only UI fix.

UI consistency uses the task's eight fixed screen roles and the actual shared
shell/components/theme, rather than inventing reference measurements or reuse
majorities. Native selector is the first screen of its kind. Parent owns the
separate checker/calibration. Author verification here is functional evidence,
not independent UI acceptance. Reading/search cost was not individually logged:
approximately 20 project files and 20 source searches, plus the UI consistency
and Computer Use instruction files; these are estimates.

## Open external/integration gates

Physical 150%/200%, mixed-DPI dragging and real display hot-unplug are unperformed;
adapter geometry/DIP tests cover unavailable conditions without changing Windows
settings. Actual OS desktop lock/hardware suspend remains task05/manual QA open;
private HWND messages exercise application notification routing only. A real
permission-denied/secure-desktop/DRM scenario was not induced; Win32 capture failure
maps separately from decoder failure. Protected content may return black pixels
with GDI success and then NoQr; the app neither detects every such case nor bypasses
OS protections. Native session notifications/idle cancellation use cooperative
25ms timer checks; selection preserves the existing main-session idle deadline
and does not extend unlock duration through native pointer movement.

Task08/09 own complete review/save/image/manual journeys; task20 the wider security
audit; task21 full accessibility, offline and populated-screen QA. Exact visual
reference pixels, true cross-DPI/hardware coverage and future code/review/recovery
concealment are not certified here. Parent owns calibration/reviewer/status/commit.

## Focused P3 native typography resolution

Distinct review accepted the native/security implementation with a P3 ambiguity:
the ROOT complete typography bundle did not describe STATIC/DrawText row metrics.
The intentional native exception is now explicit in architecture/UI foundation:
retain real native controls/accessibility, share family/size/weight/colors, use
OS font-cell leading and zero default native tracking. ROOT's 20-pixel line-height
and future nonzero tracking are not claimed to control these native widgets.

`native_font_metrics_and_control_fit` creates a private memory DC and loads the
same embedded fonts via the production RAII helper. It measures actual
GetTextMetricsW/GetTextFaceW/GetObjectW/GetTextCharacterExtra/
GetTextExtentPoint32W/DrawTextW(DT_CALCRECT), including every instruction line and
the exact Cancel text used by production. No window pixels/QR/screenshot are read,
no fonts are installed in Windows, and no settings change. All selected fonts,
DCs and private resources release through RAII.

The initial probe failed usefully: BODY was 400 but requesting regular-family
Noto Sans at 600 realized tmWeight 700. Read-only TTF name-table inspection showed
the bundled semibold's Win32 family is **Noto Sans SemiBold**. The shared native
helper now derives both face and numeric weight from `theme::Weight` and realizes
the actual 600 face. No custom rendered text control or row-metric overhaul was
added. Existing control names/keyboard behavior remain native.

| Requested 96-DPI probe | BODY instructions | LABEL Cancel |
| --- | --- | --- |
| Actual GetTextFace | Noto Sans | Noto Sans SemiBold |
| Actual tmWeight | 400 | 600 |
| Requested LOGFONT lfHeight | -14 | -14 |
| Cell height / ascent / descent | 19 /15 /4 | 19 /15 /4 |
| Internal / external leading | 5 /0 | 5 /0 |
| Actual native character extra | 0 | 0 |
| Measured widest line / block height | 375 /57 | 102 /19 |
| Central available width / height | 528 /72 | 528 /36 |

The current central instruction height is 72, rather than 70 in the review handoff
wording. All three unwrapped lines fit with 15 pixels vertical spare; Cancel's
single 19-pixel row fits the 36-pixel native button. This records actual96DPI GDI
metrics, not physical 150/200% or mixed-DPI certification. Old opaque panel
screenshots precede the corrected realized600 face; no new pixel-fidelity claim
is inferred from those images. Parent's `calibration.md` was preserved unchanged.

Exact focused measurement command:
`rtk proxy cargo test native_font_metrics_and_control_fit -- --nocapture --test-threads=1`.
The corrected run passed in 0.01s with the values above. Final focused checks:
`rtk cargo test platform:: -- --test-threads=1` passed 12 tests in 0.26s;
`rtk cargo fmt --all -- --check`, `rtk git diff --check` and
`rtk cargo clippy --all-targets -- -D warnings` passed;
`rtk cargo build --bin Authenticator --example region_capture --example qr_fixture`
passed. No full KDF suite is repeated for this source/metrics change. Independent
focused recheck subsequently passed (see parent review below). API contracts: [selected font metrics](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-gettextmetricsw),
[TEXTMETRIC fields](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-textmetricw),
[CreateFont mapping](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createfontw).

Official Win32 API contracts checked 12 November 2026: [thread DPI context](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setthreaddpiawarenesscontext),
[GetCursorPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getcursorpos),
[button event coordinates](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-lbuttondown),
[monitor enumeration](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors),
[SetCapture](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setcapture),
[WM_CAPTURECHANGED](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-capturechanged),
[CreateDIBSection](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createdibsection),
[BitBlt](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-bitblt),
[DwmFlush](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmflush).

## Parent review and integration checkpoint

Distinct `review_07` inspected native capture/privacy/resource paths, ROOT preparation
and result gating, safe failure paths, shared theme and real native Cancel/Escape
interaction. It found no material native/security blocker. Its P3 concern about
native leading was resolved through the measured, explicit native typography exception
above, including correction of the actual selected semibold face. Focused re-review
independently passed `rtk proxy cargo test --locked --target-dir target/task02-checks native_font_metrics_and_control_fit -- --nocapture --test-threads=1`
(1 passed), `rtk cargo test --locked --target-dir target/task02-checks platform:: -- --test-threads=1`
(12 passed), `rtk cargo fmt --all -- --check` and `rtk git diff --check`.

The parent inspected representative sanitized scan-result and opaque-panel images.
No additional live render result is inferred from old screenshots after the corrected
600 face. Review approval covers the implementation and observed scenarios; it does
not close the hardware/manual or task08/09 integration gates listed above. The task
remains in progress for those gates, while reviewed APIs are ready for task08.

Subsequent task08 integration: working image/manual fallback, candidate choice,
masked editable review, explicit atomic encrypted batch save and real current
codes passed author and distinct review_08 checks. Independent HMAC comparisons
matched real committed SHA1 and SHA512 entries; lock removed review/account/code
nodes and dismissed the image picker. The earlier handoff statements that these
editors were pending describe the task07 checkpoint, not current functionality.
Physical capture/OS hardware gates remain open; see task08/independent-review.md.
