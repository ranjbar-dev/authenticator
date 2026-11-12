# PRD coverage and evidence

Status of every row at plan generation: **planned, not implemented or verified**. Task IDs identify owners; tests/reports/commits are filled in only during execution. Use PRD.md for the full normative wording.

| Requirement | Owning tasks | Required evidence | State |
|---|---|---|---|
| App name, Rust UI/core, Windows-first architecture | 01, 04, 23 | Build, framework spike, executable/package identity | task01 foundation verified; remaining owning tasks pending |
| Reference inventory and honest visual evidence | 01, 04, 21 | docs/reference-ui.md and comparison report | task01 foundation verified; remaining owning tasks pending |
| Screenshot-first primary workflow and local import | 06–08, 21 | Complete capture/review/save/code-verification journey offline | task08 functional flow independently approved with live HMAC matches; disconnected-machine acceptance08/21 open |
| FR-1.1 real cards, icons, code, time, empty/loading/error/locked | 04, 05, 10, 21 | State screenshots and real-vault UI checks | task04 shell and task08 committed real codes/lock verified; full task10 list and final QA pending |
| FR-1.2 exact boundaries, sleep/resume, clock problems | 02, 10, 19, 20 | Clock-injected tests and Windows lifecycle checks | task02 domain verified; integration/remaining owners pending |
| FR-1.3 copy confirmation, near-expiry, safe clipboard cleanup | 10, 16, 20 | Real clipboard ownership/race tests | planned |
| FR-1.4 search, no results, all six sorts, list/grid persistence | 10, 21 | UI interactions and restart tests | planned |
| FR-1.5 explicit edit/delete and trash | 09, 18 | Edit/delete/cancel/restore journey | task09 stable-ID edit verified; task10 delete entry point and task18 trash pending |
| FR-2.1 fields and Base32 validation/normalization | 02, 09 | Parser/unit and editor tests | verified functional02/09; real manual validation/edit/relaunch and strict domain tests passed |
| FR-2.2 image/region local QR and editable review | 06–09 | PNG/JPEG and screenshot import tests | tasks06–09 functional decoder/capture/image/manual/editable import reviewed; full offline/hardware QA21 open |
| FR-2.3 algorithms/digits/periods/defaults/URI preservation | 02, 08, 09 | Standard vectors and all parameter combinations | verified functional02/08/09; all24 exact settings persisted through create/edit/reopen, live SHA1/256/512 HMAC matches |
| FR-2.4 safe invalid-input errors, retained edits, duplicate warning | 02, 06, 08, 09 | Negative/duplicate/failure UI tests | verified functional02/06/08/09; independent invalid/stale/duplicate/failure/retention checks and P3 correction passed |
| FR-2.5 creation with recovery codes and overwrite intent | 09, 15 | Atomic account-plus-codes transaction tests | task09 atomic account+normalized codes and explicit targeted edit verified; standalone recovery editor15 pending |
| FR-2A.1 discoverable capture without login/network/file save | 04, 07, 08 | Main-screen action and offline journey | task08 real user-initiated capture without secret typing/file save passed; disconnected-machine run open |
| FR-2A.2 region overlay, monitors/DPI, Escape restore | 07, 21 | Pointer and scaling/monitor evidence | task07 primary/negative monitor pointer and Escape/Cancel evidence; hardware DPI/manual QA pending |
| FR-2A.3 memory-only QR, masked editable review, multiple codes | 06, 08 | Multi-candidate/parameter review tests | tasks06/08 functional scope verified; explicit mixed selection and masked editable batch passed |
| FR-2A.4 explicit import, duplicates, current code on success | 08 | Atomic save and independent TOTP comparison | task08 functional requirement verified by independent real batch import/duplicate preservation/HMAC matches; see docs/qa/task08/independent-review.md |
| FR-2A.5 every capture/QR failure with retry/image/manual options | 06–08 | Failure matrix and UI evidence | tasks06–08 safe failure/retry and working image/manual fallback reviewed; hardware failure delivery QA open |
| FR-2A.6 no pixel/payload persistence, cleanup and lock gating | 03, 05–08, 20 | Artifact/log inspection and lock-race tests | task03/05/07/08 implementation reviewed, pending/locked cleanup and commit races tested; full hardware/runtime audit20 pending |
| FR-3.1 sets, account selection, line/file entry and review | 14, 15 | Empty/populated/select/file/text paths | planned |
| FR-3.2 encrypted concealed codes, reveal/copy/used/undo | 03, 16 | Used-state persistence, intentional reveal, copy does not mark used | task03 storage verified; UI/platform/remaining owners pending |
| FR-3.3 edit/delete confirmations and recovery guidance | 15, 16, 19 | Destructive/cancel paths and help copy | planned |
| FR-3.4 blanks/duplicates/large/encoding/source/deletion handling | 15, 16, 18 | Normalization/limits and association tests | task09 shared bounded normalization/exact duplicate review verified; file/used/deletion workflows15/16/18 pending |
| FR-4 folder name/color/search/membership/navigation/unfiled | 17 | Many-to-many, six colors and keyboard checks | planned |
| FR-4 persistence/backup and remove-folder-preserves-accounts | 11, 17 | Restart/round-trip/delete tests | planned |
| FR-5 retention policy, restore, single/all permanent delete | 18 | No-expiry copy, restore/conflict/confirmation tests | planned |
| FR-5 account/metadata/codes restored atomically and no swipe-only UI | 18, 20, 21 | Transaction faults and real button/keyboard tests | planned |
| FR-6.1 easy encrypted portable export of every supported entity | 11, 12 | Format docs, complete encrypted round-trip | planned |
| FR-6.2 version/integrity/passphrase/preview/conflict/merge/replace | 11, 13, 20 | Negative, transactional and UI tests | planned |
| FR-6.3 atomic export, destination, explicit overwrite, safe errors | 11, 12, 20 | Interrupted write/overwrite/file-picker checks | planned |
| FR-6.4 reference .vault discovery, no unsupported claim | 22 | Evidence-based compatibility report | planned |
| FR-7.1 user/idle lock, unlock/recovery, optional OS protection | 03, 05, 19, 20 | Key design, lifecycle and lock-race checks | task03 storage verified; UI/platform/remaining owners pending |
| FR-7.2 counts/status/version/startup/preferences/English UI | 19, 21 | Settings/restart/failure screenshots | planned |
| FR-7.3 system time/resume, local guidance, no remote clock mutation | 02, 10, 19 | Injected clock and OS behavior evidence | planned |
| FR-7.4 explicit optional startup and failures | 19, 23 | Registration toggle, denial, package smoke test | planned |
| FR-8 no Google/app accounts/cloud/sync/multilingual/translation | 01, 04, 19–21 | Source/UI and offline network audit | task04 shared shell verified; later workflows/full QA pending |
| FR-8 all persisted user data encrypted, no uploads | 03, 11, 20 | Ciphertext/artifact/traffic evidence | task03 storage verified; UI/platform/remaining owners pending |
| PRD §7 compact reference design across all pages/states | 04–05, 07–10, 12–19, 21 | Per-task checklist and independent comparison | task04 shared shell verified; later workflows/full QA pending |
| PRD §7 resize/scale/contrast/focus/labels/keyboard/Narrator | 04, 21 | Multi-size/DPI and accessibility report | task04 shared shell verified; later workflows/full QA pending |
| PRD §8 threat model, crypto/KDF/versioning and key lifetime | 01, 03, 11, 20 | Security design and separate-agent review | task01 foundation verified; remaining owning tasks pending |
| PRD §8 no secrets/PII in diagnostics, no telemetry or raw captures | 03, 06–08, 20 | Sentinel file/log/crash/network audit | task03 storage verified; UI/platform/remaining owners pending |
| PRD §8 untrusted parsing bounds, schema checks, transactions | 02, 03, 06, 11, 20 | Adversarial/fault-injection test results | task03 storage verified; UI/platform/remaining owners pending |
| PRD §8 locked UI/accessibility, clipboard and endpoint limits | 05, 10, 16, 20 | Lock checks and accurate limitations docs | task05/08/09 locked/held-save AX and nonblocking cancel reviewed; clipboard10/16 and full audit20 pending |
| PRD §9 entities/relations/migrations/rollback/no secret filenames | 02, 03, 11, 18, 20 | Migration/referential integrity/filename tests | task02 domain verified; integration/remaining owners pending |
| PRD §10 deterministic clock, separate layers/adapters, offline | 01–03, 06–07, 19–20 | Architecture and adapter tests | task01 foundation verified; remaining owning tasks pending |
| PRD §11 full correctness/workflow/recovery/privacy/UI/reliability | 02–21, 23–24 | Consolidated regression and QA report | planned |
| Clean Windows package, English usage/developer/recovery docs | 23 | Package build, clean-profile smoke/recovery | planned |
| Commit/push main, final PRD and task/evidence reconciliation | 24 | Clean status and remote/local SHA equality | planned |

## Execution updates

Task01 accepted: implementation 7502fa4, independent review_01 re-review approved, Windows CI run 43227714011 succeeded, six local tests and live inert spike evidence in docs/qa/task01/. No full multi-owner requirement row is yet verified. Replace each row's state with verified only after its acceptance checks pass; link commands, report paths and commit IDs here or in owning task evidence. Hardware or reference limitations must stay explicitly unverified. Optional reference-format compatibility may be unsupported with a completed discovery report; mandatory core features may not be silently waived.


Task02: separate review_02 approved;20 tests incl published vectors/all24settings, strict bounded parser, redacted aggregates and validated relations pass. See docs/qa/task02/implementation-evidence.md. Full multi-owner UI/persistence rows remain pending.

Task03: review_03 focused security/data-loss review approved after native error-path cleanup;32 main tests plus actual process interruption/lock probes, ACL/sharing checks pass. See docs/qa/task03/implementation-evidence.md and docs/storage-format.md. UI lifecycle, backup and full runtime audits pending.

Task04 accepted after distinct calibration + review_04/re-review. Shared role/contrast/lock-state tests,2 actual editor-metric/password tests, livepointer/key/modal/Lock,9fixedwindow/appzoom samples; source-only calibration limit explicit. See docs/qa/task04/. PhysicalWindowsDPI and complete workflow/accessibility/reference QA remain pending.

Task05 implementation reviewed (not fullhardwareacceptance): async single-store owner, cancellation/commit gate, secret-history clearing, monotonicidle and registerednativeWTS/poweradapter tested.45fulltests+subprocessprobes; independent5state/1native/3cancel/3editor checks plusliveUI pass. ActualOSsessionlock/suspenddelivery and futurecapture/import/revealedcode integration remain unverified; FR7.1/lockedprivacyrows mustnotbe fullyverifiedyet. See docs/qa/task05/.

Task06 accepted: review_06 independentlypassed15QR+15domain/foundation tests, lockedClippy/fmt/diff andinspectedpinnedcodecsource. BoundedPNG/JPEG/pixels,alpha,strictparser/mixedcandidate/saturation/privacy evidence docs/qa/task06/. No actualcapture/UI/save claim; FR2/2A capture/cancel/lock/import remain07/08/21.

Task07 implementation independently reviewed and focused re-review passed (1 actual native font-metric test and 12 platform tests). Actual pointer QR acquisition on primary and negative-coordinate monitors, reverse drag, Cancel/Escape/no-QR/private-WTS lock and restored focus are evidenced in docs/qa/task07/. GDI selected-only memory capture, bounds, cancellation epochs and accessibility preparation are implemented; native measured typography exception is explicit. Task07 remains open for hardware/manual coverage and working image/manual/review/save integration. Screenshot-first and FR2A rows therefore remain partially implemented, not fully verified.

Task08 functional implementation independently approved by review_08 with no findings requiring changes. Full76 author tests plus final focused regressions; independent atomic/stale/AX/contrast/duplicate/domain/build/lint checks passed. Real pointer capture, explicit multiple selection, invalid edit retained, whole-batch conflict/retry, three separate accounts preserved, Help/Cancel/pickerLock and340/600/960 layouts passed. Independent SHA1/6/30 and SHA512/8/120 HMAC matches recorded in docs/qa/task08/independent-review.md. Source/source-only evidence does not close the disconnected-machine/hardware gates; Task08 remains in progress for offline acceptance, Task05 offline checkbox corrected to open, and later09–24 owners remain required. Commit SHA follows in checkpoint.

Task09 accepted after review_09 and focused re-review resolved P2 responsive-lock and P3 stale duplicate feedback. Final full88 tests and app/examples/fmt/strictClippy pass; independent held-save1/cancellation5/feedback1/atomic1 and prior all24/association/AX/contrast checks passed. Actual owned process restart, independent HMAC and earlier340/600/960 pointer/key evidence recorded in docs/qa/task09/. Final cursor-access denial limited independent pointer repetition; author final P3 pointer evidence and independent source/tests were used transparently. Shared cancellation now admits atomically without making UI wait on active filesystem action; pending worker memory lifetime remains limited by I/O/crypto return. No global05/07/08/21 manual gate is closed. Commit SHA in following checkpoint.
