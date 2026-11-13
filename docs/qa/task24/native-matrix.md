# Task 24 native matrix — author checkpoint

Status: scoped observations, independent review pending. This is not the full Task 21 acceptance matrix. The [manifest](frame-manifest.json) binds every retained PNG to its root, executable SHA256, dimensions, available metadata and inspection status. Screens are production components running against an isolated encrypted public fixture, with an added QA banner; they are not the packaged executable. Parent must review new source and images before acceptance.

## Fixture, input and capture policy

No-argument `examples/final_matrix.rs` creates a fresh canonical workspace-owned target/task24-owned-* root and never opens the default vault. Seven published-vector accounts initially include six active/one Trash, one folder/membership and one recovery set with one available/one used value. Later Keep Both and QR imports intentionally change counts. The generated backup key differs from the fixture's local key; no filled phrase/OTP/QR image is persisted in this report.

All interaction used actual Sky pointer/keyboard/native dialogs after reading the computer-use skill. F3/F4/F5 set application zoom100/150/200; F6/F7/F8 request logical widths340/600/960, logical heights780/800/900 bounded to1000/zoom to keep the fixture on the baseline100% desktop. Physical Windows scaling is separate: a600×800 logical window at200% yields1200×1600 pixels and can extend below the1440-pixel display. Native WGC/F10 may include offscreen content; those pixels alone do not prove that content was physically visible. Actual keyboard Save/HMAC and separate pointer Maximize observations are attributed below. Thus zoom150/200 reduces available logical height; these cells do not represent Windows display scaling. Controls below the viewport were reached with real wheel/Tab/Space; a viewport screenshot is not proof of an entire scrollable page.

F10 requests the renderer image, replaces evidence-sensitive rectangles (expanded4 logical points) in memory using actual pixels_per_point, then writes the only PNG. Secrets, live codes and recovery input/values are masked before disk write, including revealed account fields from prior Task 21 fixes. All retained PNGs were visually inspected; a few repeated current folder cells were byte-identical to previously inspected frames. A .ready marker plus PNG signature/chunk CRCs/IEND/inflated RGBA dimensions and SHA256 were checked. The195 retained images total51,574,899 bytes, copied unchanged. Older8MorzW helper has no per-frame DPI JSON; its labels/native observations are explicitly distinguished from later measured metadata.

Release F10 screenshot persistence is disabled. F9/F11 in matrix move to observed left/primary positions; physical coordinates are divided by actual pixels_per_point. In qr_fixture, F9 instead toggles blank and F10 moves left. F12 in the latest helper writes only existing selected-region left/top/width/height to capture-bounds.json; it compiled but has not been exercised. Never infer a visible GDI QR from a WGC window snapshot alone.

## Exact software grids

Each range below contains all nine cells: zoom100 widths340/600/960, then zoom150 same widths, then zoom200 same widths, in that order. Manifest labels are authoritative. “Observed” is author inspection, not independent acceptance. No omitted cell may be filled from inference.

| Page/state | Root/frame range | Source and result |
|---|---|---|
| Settings summary/populated |8MorzW001–009|Settings minimal-scroll fix, before central reset; readable summary/actions observed. Needs current route-reset attribution/retest where affected.|
| Export & Import hub |8MorzW010–018|Pre-central source; nine layout observations. Keyboard deep Import focus019 intentionally scrolls heading out; not an initial-heading pass.|
| Blank encrypted export |8MorzW020–028|Pre-central source; nine observations, blank phrase inputs. Submit-focus029, no-destination error030, wheel031 and native picker Cancel032 also observed. Error increases content; scroll reaches complete action.|
| Blank import |8MorzW033–041|Pre-central source; nine observations; missing-file error042.|
| Authenticated import/conflicts |8MorzW046–054|Pre-central source; nine observations with authenticated counts, no secret fields. Open Keep Both055/selected056 real pointer.|
| Exact Keep Both candidate review |8MorzW057–065|Pre-central source; nine observations. Page2 keyboard066, Apply focus067; success068 and Settings counts069.|
| Locked |8MorzW070–078|Pre-central source; nine observations, empty concealed passphrase control, scroll at200. No filled passphrase retained.|
| Recovery list |8MorzW079–087|Pre-central source; nine observations; scrolled ViewSet focus088.|
| Recovery editor, existing masked values |8MorzW098–106|Pre-central source; nine observations; source masks correct at actual appzoom PPP1/1.5/2. Discard107 mask covers part of dialog, so not whole-dialog readability proof.|
| Recovery editor, new empty long-name account |8MorzW110–118|Pre-central source; nine observations; empty-preview error119, no-match108/chooser keyboard109 at600×800 only.|
| Codes no matching accounts |8MorzW120–128|Pre-central source; nine no-match observations. No claim for every Codes list/grid/menu state.|
| Folder list |Ru9vqf001–009|Current product ScreenKind reset; nine cells inspected (four byte-identical to inspected TfS9eB frames).|
| Create folder |Ru9vqf010–018|Current product; nine cells inspected (six byte-identical to inspected TfS9eB frames).|
| Edit existing folder |Ru9vqf020–028|Current product; nine inspected. Opener keyboard019 and deep first checkbox029 at340/zoom200; Escape preserves original data.|
| Recovery detail |Ru9vqf030–038|Current product; nine inspected after actual scrolled list→ViewSet. Heading/Back now visible; earlier pre-reset detail89–97 excluded from final pass.|
| Populated Trash |Ru9vqf039–047|Current product; nine inspected. Existing21 independently reviewed real Restore/permanent/Empty paths remain separately attributed.|
| Editable screenshot QR review |Ru9vqf048–056|Current product; nine inspected, secret concealed and masked before persistence. Save keyboard057 at600×800; explicit Save058.|

Current account/manual editor all-nine grid was not repeated; existing Task 09 functional/native evidence and an earlier1ydLLc edit grid are historical, not a new current-source pass. Empty folder/Trash, all dialog/error/open-menu/focus states, all physicalDPI/page combinations, and full reference comparison remain attainable/unperformed. Current ScreenKind includes the shared Review path used by manual/account editing; source coverage is not native proof of those cells.

## Physical scaling and capture

All three extended monitors were read at100%(Recommended) before changes. Only monitor1 was temporarily changed using normal Windows Settings UI. Appzoom stayed100; actual GetDpiForWindow96/144/192 and pixels_per_point1/1.5/2 were observed separately from app zoom. See [capture booleans](native-capture-results.json).

| Case | Actual result / safe frames |
|---|---|
|96 DPI primary|Real pointer QR crop→editable review; source cipher unchanged before Save; explicit import, independent SHA1 HMAC/counter match19:03:17.159Z13Nov2026. Escape separately closed overlay/restored Screenshot focus/preserved cipher. Ru9vqf057–058.|
|144 DPI primary|Real crop of public QR at[12,135,660,783]→review; source unchanged before Save. Unique edited label Public DPI150 unique fixture, exactly1matching record, explicitSave and independent current HMAC/counter match19:34:15.510Z. Ru9vqf060–061.059 is actually locked, corrected in manifest.|
|192 DPI primary|Real crop at[16,181,880,1045]→review; source unchanged before Save. Unique edited label Public DPI200 unique fixture, exactly1matching record, explicitSave and independent current HMAC/counter match20:07:22.324Z. Ru9vqf065–067.|
|Mixed monitor movement|Corrected QA physical-position conversion: left96→primary144→left96 measured, actual on-monitor geometry. hlricp001–004; frames002–004 show locked state, not Codes. [Measured result](mixed-transition.json).|
|Mixed monitor QR crop|While primary144/left96, negative QR target[-1824,411,-1392,843], two changed mappings returned NoQR, both preserve ciphertext. No decode/Save/HMAC pass. Actual selected rectangle not yet recorded; tooling mapping/visibility vs product cause unresolved.|
|Physical rendering/Maximize|8MorzW150 locked150;151 no-match150;152 populated150;153 populated200;155 actual pointer native Maximize200;156 deep Copy focus200.154 was not successfully maximized and excluded.|

Physical image/capture masks at PPP1.5/2 were visually checked before curation. These are debug component/adapter workflows, not smoke of a future rebuilt release artifact. No raw QR or code pixels are in retained evidence. HMAC comparisons used only the exact newly imported unique record at144/192, not an older same-name record.

## Accessibility and OS settings

Narrator baseline OFF. Actual Narrator Speech Recap observed safe spoken region/Help button, Offline help dialog/Close/heading and, after app Lock, Vault passphrase/edit. No OTP/filled phrase was focused; no raw transcript/capture persisted. This proves those limited names/roles and focus transitions, not a whole-app screen-reader pass or explicit password-protected announcement. Insert+Escape exit was verified process absent; baseline OFF restored. Non-code-focus30s rollover announcement check remains unperformed.

Windows Contrast themes baseline None→Aquatic→None applied and visually verified through native Settings. Actual folder/locked controls and keyboard focus remained readable (8MorzW147–148). The app keeps its brand palette; this is not proof that it follows every Windows contrast color. Native Windows Settings eventually worked through Control Panel→Ease of Access→Optimize visual display→theme link; older direct-launch-unavailable statements are superseded. No security/network/clock/logon/resolution/layout change was made.

At final boundary all monitors1/2/3=100%(Recommended), Contrast=None, Narrator=OFF, restoration verified. All owned final_matrix/qr_fixture windows and Settings265152 were closed normally, inventory confirmed absent. Old unrelated workflow fixture and other processes were untouched. No native exclusivity remains held.

## Excluded/historical evidence

Initial abandoned g9abW6 frame009 is a zero-byte placeholder caused by closing before the old asynchronous writer completed; it is not a screenshot/pass and was never copied here. 1ydLLc/TfS9eB and8MorzW pre-fix detail/folder rows remain ignored historical fixtures. Ru9vqf062–064 are failed QA movement attempts at144DPI, not mixed-monitor passes.8MorzW043 is export success already,045 authenticated preview already: neither proves a visible busy/progress state. The final manifest records observations; no full-matrix “pass” is inferred from filenames.
