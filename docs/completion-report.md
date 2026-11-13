# Authenticator completeness audit — incomplete

Task 24 scoped checkpoint, 14 November 2026 local. Implementation through Task 23 and extensive prior independent reviews are evidenced. This audit repaired UI and package regressions and expanded actual native QA. A [distinct reviewer](qa/task24/independent-review.md) approved the scoped repairs, independently inspected 68 current and 17 historical frames, and verified all 195 frame integrity records. The parent restored Task23's original open reproducibility requirement and replaced stale Task24 evidence. Mandatory acceptance and the feasible follow-ups below remain open; parent owns integration, Git and current-source CI. No whole-project completion is claimed.

The [48 current coverage rows](../tasks/coverage.md), [implementation and tests](qa/task24/implementation-evidence.md), [exact native matrix](qa/task24/native-matrix.md), [195 safe image manifest](qa/task24/frame-manifest.json) and [verbatim open task checklist audit](qa/task24/task-checklist-audit.json) distinguish implementation, earlier independent review, actual tests and unverified requirements. Historical working/coordinator checkpoints retain their chronology; this report supersedes their active ownership and OS-state claims.

## Implemented behavior and repairs

Rust/eframe UI and core implement local TOTP, screenshot/image/manual entry, editable masked review, explicit encrypted import, search/six sorts/list/grid, safe clipboard leases, backup codes/used/Undo, many-to-many folders, Trash/restore/permanent confirmation, locking/settings/startup/help and native encrypted export/import. AUTHLOC1 local storage and AUTHPBK1 backups encrypt metadata and preferences. Defaults are 300-second idle lock and 30-second clipboard expiry. Trash never automatically expires. Backup keys are independent of local vault keys. Private reference .vault is unsupported after accepted discovery. No app sign-in, cloud, telemetry, translation infrastructure or required runtime network service was added.

Settings initial focus aligned Export & Import to the top and hid the heading/counts. Similar alignment and inherited outer scroll state hid subpage headings, including recovery detail reached from a genuinely scrolled list. Minimal visible focus plus a one-shot page-identity reset repairs these cases while preserving ordinary wheel position and deliberate deep-row/Undo focus. Red-to-green Context paint tests and current folder list/create/edit/detail/Trash grids substantiate the repair; independent review is pending.

The hosted package job failed its vendored-notice hash guard after the check job passed. A real local Git autocrlf simulation reproduces LF conversion without attributes and preserves all pinned hashes with narrow assets/notices/** -text. Runner bytes were not captured, so the simulated boundary is a likely explanation requiring fresh hosted verification. No provenance check was relaxed. Spaced Windows device aliases such as CON .txt now fail hostile ZIP validation. Fourteen package regressions and actual existing-archive validation pass.

## All 24 task outcomes

“Reviewed” refers to an existing separate task reviewer, not this author. Task files and parent-owned statuses remain authoritative; this table proposes evidence reconciliation without changing them.

| Task | Evidenced outcome | Remaining acceptance / correction |
|---|---|---|
|01|Rust foundation, framework spike, architecture/reference inventory; separate review and Windows CI43227714011.|Accepted foundation; exact installed-reference parity not proved.|
|02|Bounded domain/URI/TOTP, standards vectors and all 24 settings; independent review.|Accepted engine; actual OS resume is a later lifecycle gate.|
|03|Authenticated local storage/KDF/keys/ACL/atomic faults and interruption probes; independent security review.|Accepted storage scope; no physical power-loss or total memory-erasure guarantee.|
|04|Shared reference-style shell, roles/contrast/password, real pointer/key and nine application-zoom samples; separate review.|Accepted shared shell. Application zoom is distinct from physical DPI; full-page parity not inferred.|
|05|Async sole-owner setup/unlock/Lock/cancellation/idle/lifecycle adapters reviewed; actual Create/Lock/relaunch.|Actual WTS session-lock/suspend delivery, effective offline and overlay Lock cases remain open.|
|06|Bounded local PNG/JPEG/QR decoding, strict candidates/parser and independent tests.|Accepted decoder; unit tests do not prove mixed-DPI capture or disconnected runtime.|
|07|Native selected-region capture, primary/negative coordinates, reverse drag/Escape and review.|Task 24 adds primary96/144/192 capture/HMAC and movement96→144→96. Mixed-DPI negative crop remains unresolved.|
|08|Complete screenshot→review→Save→independent HMAC, multiple selection/duplicates/retry; independently approved.|Disconnected-machine gate remains. Current QR review grid and exact144/192 imports add scoped evidence.|
|09|Manual create/edit/validation/advanced settings/relaunch and native HMAC reviewed.|Accepted editor functional scope; current complete manual/account-editor grid not repeated.|
|10|Six sorts/list/grid/query/filter encrypted reopen, clipboard repair/re-review and native Copy/expiry/later-copy/Lock/exit.|Hardware clock/resume and complete state/scale QA remain.|
|11|Portable graph, independent keys, authentication, merge and atomic interruption engine reviewed.|Accepted engine. Same-user independent-key restore is not another user/device recovery.|
|12|Corrected export worker/preferences/file consent reviewed;21/23 actual full-graph picker/export/restore.|Old blanket native-unavailable claims superseded. Current complete states, clean profile and offline remain open.|
|13|Corrected authenticated preview/candidate/conflicts/atomic merge/replace reviewed;21 actual fresh Merge/HMAC/Keep Current/Replace Cancel/Confirm.|24 actual Keep Both candidate/Apply agrees with associations/counts. Full matrix/cross-profile/offline remain.|
|14|List/selector/count/search/sort/privacy reviewed;21 actual file/list/detail/roundtrip and24 attributed grids.|Every empty/open/focus/error cell not exhausted.|
|15|UTF8/BOM/1MiB text/file normalization, duplicate review, retained used state and atomic editor reviewed;21 native file/preview/Save.|24 existing/new-empty editor grids are source-attributed; all invalid-file/error/scale states remain.|
|16|Conceal/reveal/copy/used/Undo/Trash guards reviewed;21 actual clipboard/used/Undo/Lock/restore.|24 current detail grid and deep-focus regression; full privacy/state/environment matrix remains.|
|17|Many-to-many/six colors/navigation/persistence/removal reviewed; hosted required portable case passed.|21 actual membership/filter/Trash/backup and24 current folder grids supersede native-unavailable. Empty/error/menu cells remain.|
|18|Five-kind Trash/restore/permanent/cascades/faults reviewed; hosted corrected preference test passes.|21 actual Restore/single/all Cancel/Confirm and24 populated grid. Complete empty/error/scale matrix remains.|
|19|Counts/preferences/English help/startup/triggering-frame Lock privacy reviewed;21/23 actual help/startup/restart.|24 Settings repair and limited Narrator proof await review. Actual logon/offline/full matrix remain.|
|20|Independent security/data-loss/fault/sentinel source review and hosted suite pass.|Fresh full-lockfile advisory check below. No formal crypto certification or complete runtime acceptance.|
|21|Independent remaining-native review covers recovery/folders/Trash/full graph Merge/Keep Current/Replace; source suite gate closed.|24 adds195 curated images, primary DPI, contrast, limited Narrator and focus repairs. Full software matrix/mixed crop/rollover remain feasible open.|
|22|Discovery independently accepted; unknown-format rejection preserves ciphertext; official phone references documented.|Private .vault unsupported; no converter or exact Windows-parity claim.|
|23|Reviewed unsigned ZIP/PE/resources/static CRT/docs/notices; native scan/copy/export/separate-key restore/Explorer/startup/restart.|Clean ebd artifact proved; latest check passed/package failed.24 repairs need review/new CI. Original reproducibility and clean-profile/offline/logon gates remain.|
|24|All coverage rows reconciled, actual decisions recorded, source fixes/targeted checks/native evidence prepared.|Distinct review, remaining mandatory QA, current full checks/release/package, final integration/push/clean SHA equality remain.|

Task12–19 implementation/native checkboxes can be reconciled by parent against specifically reviewed21/23 evidence. Their full acceptance must not be inferred. Task23 original **reproducible** wording must remain; replacing it with **repeatable** would weaken an unresolved requirement.

## PRD success, security, visual and reliability assessment

Every FR item is mapped in the 48-row coverage table. FR4.1/2/3 map to folder editing, many-to-many preservation and keyboard All/Unfiled navigation. FR5.1/2/3 map to no-expiry/confirmed permanent policy, atomic association restoration and actual buttons/keyboard. FR8.1/2/3/4 retain exclusions, encrypted persistence, mandatory offline operation and English-only behavior. The explicit offline acceptance remains unverified.

| Success criterion / validation | Evidence and limit |
|---|---|
|Add/search/current independent code|Actual08/09/21/23 and24 exact-record HMAC; standards/all24 settings reviewed.|
|Configured boundaries/refresh|02/10 injected clocks and120 independent HMAC values. Actual Windows sleep/resume/clock delivery unverified.|
|Portable clean restore|11 complete graph tests;21 native fresh independently keyed vault and23 packaged restore. Same OS user is not a clean installation/user/device.|
|No diagnostic secrets|20 source/sentinel/panic-hook probes, no raw-payload persistence/telemetry;24 masking before disk. OS dumps/pagefile/history/other-process copies remain limits.|
|Offline first use/scan/generate/export/import|No required runtime networking client; connected-host journeys pass. Firewall attempt CategoryDisabled did not enforce isolation and was removed.|
|Screenshot without secret typing|Actual primary96/144/192 QR→concealed review→explicit Save→exact-record HMAC;08 multiple-QR journey reviewed. Mixed-DPI negative crop unresolved.|
|Reference visual fidelity|PRD inventory and actual safe screenshots/focus checks. Exact installed Windows reference pixels/version unavailable; public phone pages do not prove parity.|
|Keyboard primary workflows|Actual04/08/09/10/21/23 and24 ViewSet/checkbox/Apply/Help; deep-focus regression pass. Every page/state/layout not exhausted.|
|Security §8|03/11/20 separate scoped review, authenticated bounded formats/transactions/zeroizing owners. No external cryptographic certification.|
|Data §9|Schema2 lazy migration, IDs/relationships/used/Trash/preferences tests, native graph/Keep Both counts. No trusted anti-rollback counter or secure SSD-erasure promise.|
|Architecture §10|Rust UI/domain/storage/QR/platform boundaries, injectable OTP clock, native adapters. Deterministic package reproducibility still engineering work.|
|UI §7/11|Width/application-zoom grids, primary physical-DPI capture, limited Narrator and actual Aquatic checks. Full state/physical/reference/rollover matrix remains.|
|Reliability §11|Atomic faults/storage-full/stale/cancel/interruption/concurrency/restart tests and native persistence. Physical power loss, OS lifecycle and offline gates separate.|

[Fresh advisory evidence](qa/task24/advisory-current.md): cargo-audit0.22.2 checked385 locked dependencies, including new build-only dependencies, against RustSec1509/revision4ef2df8e806a65e2f1ecc14f56f51249062b23d6. Exit0, zero reported vulnerabilities, three unsuppressed Unix maintenance warnings; none in the current184-record Windows normal/build inventory. No ignore/target filter. Historical Task20 audit is not substituted for this dated check, which does not guarantee absence of vulnerabilities.

## Artifact and check attribution

Last parent-pushed source is ebd80e25794dccd699c9b3316914fe4af1b0c11b. [Clean build](qa/task23/clean-build.md) proves source_dirty=false. The ZIP is dist/Authenticator-0.1.0-windows-x86_64.zip, 6,023,315 bytes, SHA256 dcd9bdf4680553400ee598b7b76d6287293f5ab95335b50894b2d36057563ab4. EXE SHA256 99a2df81e1792bd6eda93ae2be0b16c757402e9b6661aa1aea102e382e6ec9df matches Task23 native-tested bytes.

Hosted43465226955 on ebd80e2: check job SUCCESS (fmt/strict lint/226 ordinary all-target cases/Windows build; five direct helper subprocesses exercised separately by parents), package job FAILED notice guard; upload skipped. [Exact outcome](qa/task23/hosted-package-follow-up.md). It does not cover current repairs. Current focused45-case navigation, four width cases, deep-focus regression,14 package tests, strict all-target Clippy, fmt/diff and existing-archive validation pass; [commands/results](qa/task24/implementation-evidence.md), [final checks](qa/task24/final-checks.json). No new full-suite/release-package claim.

## Remaining feasible work

These are active work, not external blockers. Continue sequentially after distinct review of this checkpoint.

1. Launch latest F12 helper with fresh owned identity. Confirm public QR fixture visible (its F9 toggles blank, unlike matrix F9 movement). Read monitor baselines; primary150%/left100%; make one actual crop and F12 immediately after return. Compare recorded selected physical rectangle against independent ClientToScreen(0)+PPP×QR bounds. Derive input mapping from measured mismatch, or if bounds match, verify GDI foreground/occlusion before backend changes. Avoid blind repetitions. Complete mixed QR review/unique Save/exact-record HMAC; restore baselines.
2. Actual returned overlay advertises/implements Ctrl+L. Execute real Control_L+l; verify overlay cancellation, main Locked, cleared private AX and unchanged ciphertext. This is user Lock, not idle/WTS/suspend. For controlled idle case, change only disposable fixture to1minute through Settings, settle preference write/hash, wait bounded intervals and restore300seconds. Observed default autolock is not that controlled overlay test.
3. Complete current manual/account-editor nine width/zoom cells, remaining empty/error/focus/open states and affected current route-entry retests. Older attributed layouts are not current behavior proof. Complete remaining physical page/state/mixed cases, recording every unperformed cell.
4. Use official Narrator Speech Recap for a30-second rollover while focus stays on a non-code control; persist no values/transcript, restore OFF. Complete remaining keyboard/accessibility states. Prior Help/dialog/locked labels and Aquatic observations remain narrowly scoped.
5. Implement measured fixed-toolchain reproducibility: supported release linker determinism (/Brepro), source-commit epoch for built_utc/ZIP metadata, stable relative notice ordering and deterministic packaged text/Cargo.lock checkout bytes. Preserve static CRT/unsigned/provenance checks. Compare two real EXE/ZIP rebuilds and record source/toolchain/environment/residual cross-host limits. Existing PE/ZIP timestamps use current time; bit-identical output is unproved. [Rust remapping scope/limits](https://doc.rust-lang.org/rustc/remap-source-paths.html), [official LLVM /Brepro usage](https://github.com/llvm/llvm-project/blob/main/llvm/cmake/modules/HandleLLVMOptions.cmake).
6. Independently re-review material repairs; integrate; run current hosted fmt/lint/all tests/build/package and real new-package scan/copy/encrypted-recovery smoke. Parent reconciles task checklists after exact evidence, commits/pushes main and verifies clean status/remote equality. Task24 completion follows all mandatory gates.

## Environment/reference constraints

Effective offline/no-Rust/another-Windows-user recovery is unproved. Exact-app firewall attempt returned CategoryDisabled and rules were removed. No ready VM/user session. Parent read-only DISM found Windows Sandbox Disabled/RestartRequiredPossible and executable absent; no feature enable/restart. A suitable isolated environment is needed. Source inspection/network sampling cannot replace disconnected execution.

Actual OS session lock/suspend/logon delivery is unverified on this shared interactive host; no session credential/reboot/logon changes were made. App Ctrl+L/autolock do not prove WTS/suspend/logon. Exact installed-reference Windows version/pixels and lawful disposable private-format samples were unavailable. Mandatory native backup remains; unsupported private format is accepted discovery, not a waived core requirement.

Physical DPI, contrast and Narrator are no longer globally unavailable: Windows Settings became accessible, three monitors were available and scoped real checks succeeded. Remaining mixed capture/software matrices must not be mislabeled external blockers.

## Safe resume boundary

All three monitors100%(Recommended), Contrast None, Narrator OFF, restoration visually verified. All owned final_matrix/qr_fixture windows and Settings265152 closed normally; inventory confirmed absent. Older unrelated Task21 workflow fixture untouched. No temporary OS state or native exclusivity outstanding. Latest F12 helper compiled but not launched. Never reuse historical handles/PIDs without fresh identity. Encrypted fixtures/backups remain ignored under target; curated evidence contains no raw QR, live code, recovery value or filled phrase.

Parent should retain Task21/23/24 incomplete statuses and original mandatory language. This author did not change task/index statuses or commit/push.
