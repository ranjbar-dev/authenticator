# Authenticator completeness audit — incomplete

Task 24 follow-up checkpoint, 14 November 2026 local. The implementation now includes the material focus repairs, measured same-environment reproducible packaging and a real final-package scan/copy/encrypted-recovery journey. Mandatory final acceptance remains open. The distinct follow-up review approved the bounded focus and packaging changes. Five required native state cases remain; the parent owns their completion/re-review, task statuses, integration, current hosted CI and the clean committed artifact.

The [follow-up handoff](qa/task24/follow-up/README.md), [exact state coverage](qa/task24/follow-up/state-coverage.md), [59 additional safe frames](qa/task24/follow-up/frame-manifest.json), [full rebuild proof](qa/task24/follow-up/reproducibility.md) and [production runtime result](qa/task24/follow-up/production-smoke.json) supersede stale active-work claims in earlier checkpoints. The [48 coverage rows](../tasks/coverage.md) and [verbatim open checklists](qa/task24/task-checklist-audit.json) remain requirements, not waived gates.

## Implemented behavior and material repairs

The Rust desktop app implements local TOTP, selected-region and image/manual entry, masked editable review and explicit encrypted import, search/sort/list/grid, clipboard leases, recovery codes/used/Undo, folders, Trash/restore/confirmed permanent deletion, locking/settings/startup/help and portable encrypted export/import. AUTHLOC1 storage and AUTHPBK1 backups encrypt metadata and preferences. Defaults are 300-second idle lock and 30-second clipboard expiry. Backup keys are independent of local vault keys. No app sign-in, cloud storage, telemetry, translation infrastructure or required runtime network service was added. Private reference .vault remains unsupported after accepted discovery.

Earlier scoped repairs corrected initial heading/scroll state, notice checkout bytes and hostile Windows archive paths. A [distinct reviewer](qa/task24/independent-review.md) approved that limited checkpoint, verified all 195 earlier frame integrity records and actually inspected 68 current plus 17 historical images. The other 110 historical images were not independently viewed. The [follow-up reviewer](qa/task24/follow-up/independent-review.md) independently opened all 59 new author frames and two separate safe dialog frames, verified the source/artifact manifests and repeated a real native import/dialog/Lock path. Neither total is a full-matrix acceptance claim.

Hosted run 43515853116 then exposed a real focused Review-button clipping defect at 600 × 800. The original whole-button visibility oracle reproduced locally and remains intact. Native testing found a second clipped initial Cancel action when focus was requested after widget construction. The final shared helper reveals focused controls minimally for three actual frames after gained or first-observed focus; wheel, pointer input or lost focus cancels the temporary reveal. Two old-red/new-green component regressions cover late geometry and ordinary wheel preservation. The original failing test and ten focus-filter tests now pass, including 45 page/width/zoom initial-heading cases. [Detailed evidence and exact checks](qa/task24/follow-up/README.md).

Packaging now uses /Brepro, source-epoch metadata, fixed ZIP member order/timestamps/attributes, relative notice identities, deterministic JSON and narrow checkout-byte rules. Static CRT, unsigned status and exact vendored provenance guards are preserved. Two builds from empty host and target release outputs each compiled 183 crates and produced identical final EXE and ZIP bytes. The distinct reviewer accepted this measured fixed-toolchain, same-environment/path scope for the original reproducibility requirement. Cross-host/path/toolchain identity is unproved.

## All 24 task outcomes

Existing separate task reviews retain their original scope. The parent must reconcile checkboxes against precise evidence after reviewing this follow-up.

| Task | Evidenced implementation / remaining acceptance |
|---|---|
| 01 | Foundation, framework spike, architecture/reference inventory and Windows CI reviewed. Exact installed-reference parity remains unproved. |
| 02 | Bounded domain/URI/TOTP and standards/configuration vectors reviewed. Actual OS resume remains a lifecycle gate. |
| 03 | Authenticated local storage/KDF/ACL/atomic-fault probes independently reviewed. No physical power-loss or total memory-erasure guarantee. |
| 04 | Shared semantic shell, contrast, pointer/keyboard and width/application-zoom evidence reviewed. Exact reference parity is separate. |
| 05 | Setup/unlock/relaunch, app Lock, controlled overlay idle and actual overlay Ctrl+L evidenced. Actual WTS/suspend and disconnected execution remain open. |
| 06 | Bounded local QR/image decoding and strict parsing reviewed. Decoder tests alone do not prove runtime isolation. |
| 07 | Primary 96/144/192 DPI capture, monitor movement and measured mixed primary-144/left-96 negative capture now evidenced. The follow-up mixed crop and measured monitor/DPI categories are independently accepted. |
| 08 | Screenshot → concealed review → explicit Save → independent current HMAC, including final package, evidenced. Required disconnected-machine workflow remains open. |
| 09 | Manual create/edit/validation/advanced controls reviewed; blank manual and current account-editor nine-cell grids are now specifically attributed. |
| 10 | Sort/view/query/filter and clipboard behavior reviewed; final real Copy matches independent HMAC. Actual sleep/resume/clock delivery and uncovered state cells remain open. |
| 11 | Complete portable graph, independent keys/authentication/merge/atomic engine reviewed. Another-user/device recovery remains separate. |
| 12 | Actual native full-graph export and final one-account package export evidenced. Exact error/state coverage remains source-attributed; offline/isolated-user gates open. |
| 13 | Merge/Keep Current/Keep Both/Replace evidence retained; final Replace dialog fix and 257-record candidate/pagination added. Final package independently keyed restore/restart passed. |
| 14 | Recovery list/selector/count/search/privacy implementation and native journeys reviewed. No blanket all-state current native claim. |
| 15 | Text/file normalization, duplicate/used-state editor and atomic behavior reviewed; native file/preview/Save and grids retained. Unrendered error combinations remain explicit. |
| 16 | Conceal/reveal/copy/used/Undo/Trash behavior and native graph round-trip reviewed. Final affected focus/privacy regressions pass; OS lifecycle limits remain. |
| 17 | Folder/membership/filter/Trash/backup implementation and native grids retained; current folder focus regression passes. No invented empty/error/menu coverage. |
| 18 | Five-kind Trash/restore/permanent/cascades/faults reviewed; actual Restore/Cancel/Confirm and populated grid retained. Unrendered empty/error combinations remain unverified. |
| 19 | Counts/preferences/help/startup and lock privacy reviewed; real default restoration and production restart added. Actual logon/offline and speech announcements remain open. |
| 20 | Independent security/data-loss/fault/sentinel review and dated advisory audit retained. Effective disconnected runtime remains unverified. |
| 21 | Prior separately reviewed full-graph native journeys plus precise Task 24 grids/state evidence retained. Full checklist, speech/environment/reference acceptance and distinct final review remain open. |
| 22 | Discovery independently accepted; unsupported unknown private formats safely rejected. No private converter or exact Windows-reference claim. |
| 23 | Final deterministic EXE/ZIP pair and actual new-binary scan/Copy/export/separate-key restore/restart now evidenced. Clean committed package/current hosted run and isolated offline/no-Rust/user/logon gates remain. |
| 24 | Follow-up source/checks/native/package evidence is ready for distinct review. Final task reconciliation, full CI, committed artifact and remote/local clean SHA verification are parent integration work. |

## Native and accessibility boundaries

The measured mixed crop compared independent QR bounds [-1824,411,-1392,843] with actual selected bounds [-1825,411,433,432]. It decoded without a backend change, preserved ciphertext through review and saved exactly one matching record. Earlier failed NoQR mappings remain failures with unresolved causes. Actual overlay Ctrl+L canceled capture, locked and cleared private AX without mutating ciphertext. Controlled one-minute idle was still capturing at 55.134 seconds and locked by 79.697 seconds; the measured bound is reported exactly. The disposable preference was restored and saved to 300 seconds.

Application zoom 100/150/200 percent is distinguished from physical Windows DPI. Blank manual, final account editor and initial Replace Cancel have their specific nine-cell grids. Other grids retain their earlier exact source attribution. Real pointer dropdowns, keyboard dialogs, wheel-with-focus, large candidate page changes, Unicode search, empty/error/busy and locked states are identified in the [state table](qa/task24/follow-up/state-coverage.md). Unrendered software combinations remain unverified; they are not called external blockers or inferred passes.

On the populated live Codes page, a 31.049-second observation crossed a real counter rollover while Screenshot to Scan remained focused. Official Narrator Speech Recap exposed only window/title-bar chrome and failed a static-label history positive control, including a foreground retry. Therefore automatic OTP announcements remain **unverified**, not silent/passed. Earlier Help/Close speech observations remain historical attribution and require independent substantiation if they used the same inaccessible history. Actual AX labels/focus facts remain valid separately. Narrator was exited through its normal visible Exit action and process absence was verified. Earlier limited Aquatic/readability evidence remains separately scoped.

## Artifact and check attribution

Last parent-pushed source is 2e2da2cba35e99d6d6d218471a12f48d67e7abc0. The measured final working-tree package is **dirty**, with source epoch 1794607268 (2026-11-13T22:01:08+00:00), not an actual build wall time. The current artifact at dist/Authenticator-0.1.0-windows-x86_64.zip is 6,026,388 bytes, SHA256 60f940fd5364f999f7335500487e0e79024798aed0ab0f93686a6c21517e9b78. Its 12,179,456-byte EXE SHA256 is 9656ed317d6b2e36a96cf17ac5c0d2729680cb6abb850941c295f7418272978e. Both full rebuilds matched. [Exact environment, driver, hashes and limits](qa/task24/follow-up/reproducibility.md).

That exact EXE completed actual setup, negative-monitor public QR scan, edited concealed review, explicit Save, exactly-one independent HMAC, real keyboard Copy, encrypted export with a separate random phrase, restore into a newly created independently keyed local vault, normal restart locked, correct unlock/persistence/HMAC and Ctrl+L. The process reported no package identity (15700). Separately, its fresh files appeared in the previously empty Codex LocalCache path while ordinary AppData remained absent. No causal inference or clean-other-user claim is made. Only proven fresh vault/session files were preserved as ciphertext and removed after normal exit; the preexisting directory remains empty. No production QA data-root override was added.

Formatting, strict all-target Clippy, original import visibility, ten focus-filter tests, Settings paint/triggering-frame privacy and all 17 packaging regressions passed. [Exact frozen-source checks](qa/task24/follow-up/final-check-results.json), [current source hashes](qa/task24/follow-up/current-source-sha256.json). The previous hosted run failed before these repairs; no current full hosted pass is claimed. Earlier ebd80e2/99a2… and intermediate 9165…/b7ac… binaries remain historical and cannot stand in for these final bytes. Parent clean-commit packaging must record new source metadata/ZIP hash and compare EXE bytes before reusing runtime evidence.

The [dated advisory audit](qa/task24/advisory-current.md) checked all 385 locked dependencies with cargo-audit 0.22.2, RustSec revision 4ef2df8e806a65e2f1ecc14f56f51249062b23d6: zero reported vulnerabilities, three unsuppressed Unix maintenance warnings, none in the Windows normal/build inventory. This is scoped evidence, not a security certification.

## Checker calibration and limits

The [checker proof](qa/task04/calibration.md) used a detached copy at `C:/Users/root/AppData/Local/Temp/authenticator-ui-cal-4cg1yjfn/checkout`, removed afterward with the original source unchanged. TITLE weight alone was planted as Regular 400 instead of SemiBold 600; a separate calibration agent found the deviation and traced it through the shared headings. This counts as source-level whole-style sensitivity. Copied-executable launches returned ShellExecuteW 5, so live calibration was not proved. No plant tested pixel comparison, rasterization, contrast, clipping, zoom, focus, pointer interaction or speech. Actual unmodified screenshots and interaction evidence are separate review evidence; the implementation reviewer is neither the author nor that calibration agent.

## Remaining mandatory acceptance

Five bounded required native state cases in the [review](qa/task24/follow-up/independent-review.md), re-review of their additions, current hosted full tests/build/package, precise task/state reconciliation and a clean committed artifact/remote SHA remain. Effective offline first use and runtime without Rust, recovery under another Windows user or suitable isolated installation, actual WTS/suspend/logon delivery and exact installed Windows reference comparison are unverified. No ready isolated VM/user session exists; the prior disabled firewall category did not enforce isolation and was removed. Shared-host OS lock/suspend/logon was not attempted. Reference private format compatibility is unsupported discovery, not a waived native-backup requirement.

Speech announcements and uncovered software state cells remain explicitly unverified for the reasons above. The original reproducible-build requirement is preserved and now has measured same-environment evidence awaiting review. No requirement was renamed or weakened.

At this checkpoint all three monitors are 100 percent, Contrast is None, Narrator is off and the disposable idle preference is 300 seconds. Fresh final_matrix/QR/production windows are closed normally; ordinary AppData remains absent and the redirected root is empty. Older unrelated Task 21 and Task 23 windows were left untouched. Native exclusivity is released. Encrypted disposable fixtures remain ignored under target, and curated evidence contains no raw QR/OTP/recovery/filled-phrase material. This child made no task/index status changes, commits or pushes.
