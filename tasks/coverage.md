# PRD coverage and evidence

Status of every row at plan generation: **planned, not implemented or verified**. Task IDs identify owners; tests/reports/commits are filled in only during execution. Use PRD.md for the full normative wording.

| Requirement | Owning tasks | Required evidence | State |
|---|---|---|---|
| App name, Rust UI/core, Windows-first architecture | 01, 04, 23 | Build, framework spike, executable/package identity | task01 foundation verified; remaining owning tasks pending |
| Reference inventory and honest visual evidence | 01, 04, 21 | docs/reference-ui.md and comparison report | task01 foundation verified; remaining owning tasks pending |
| Screenshot-first primary workflow and local import | 06–08, 21 | Complete capture/review/save/code-verification journey offline | planned |
| FR-1.1 real cards, icons, code, time, empty/loading/error/locked | 04, 05, 10, 21 | State screenshots and real-vault UI checks | planned |
| FR-1.2 exact boundaries, sleep/resume, clock problems | 02, 10, 19, 20 | Clock-injected tests and Windows lifecycle checks | planned |
| FR-1.3 copy confirmation, near-expiry, safe clipboard cleanup | 10, 16, 20 | Real clipboard ownership/race tests | planned |
| FR-1.4 search, no results, all six sorts, list/grid persistence | 10, 21 | UI interactions and restart tests | planned |
| FR-1.5 explicit edit/delete and trash | 09, 18 | Edit/delete/cancel/restore journey | planned |
| FR-2.1 fields and Base32 validation/normalization | 02, 09 | Parser/unit and editor tests | planned |
| FR-2.2 image/region local QR and editable review | 06–09 | PNG/JPEG and screenshot import tests | planned |
| FR-2.3 algorithms/digits/periods/defaults/URI preservation | 02, 08, 09 | Standard vectors and all parameter combinations | planned |
| FR-2.4 safe invalid-input errors, retained edits, duplicate warning | 02, 06, 08, 09 | Negative/duplicate/failure UI tests | planned |
| FR-2.5 creation with recovery codes and overwrite intent | 09, 15 | Atomic account-plus-codes transaction tests | planned |
| FR-2A.1 discoverable capture without login/network/file save | 04, 07, 08 | Main-screen action and offline journey | planned |
| FR-2A.2 region overlay, monitors/DPI, Escape restore | 07, 21 | Pointer and scaling/monitor evidence | planned |
| FR-2A.3 memory-only QR, masked editable review, multiple codes | 06, 08 | Multi-candidate/parameter review tests | planned |
| FR-2A.4 explicit import, duplicates, current code on success | 08 | Atomic save and independent TOTP comparison | planned |
| FR-2A.5 every capture/QR failure with retry/image/manual options | 06–08 | Failure matrix and UI evidence | planned |
| FR-2A.6 no pixel/payload persistence, cleanup and lock gating | 03, 05–08, 20 | Artifact/log inspection and lock-race tests | planned |
| FR-3.1 sets, account selection, line/file entry and review | 14, 15 | Empty/populated/select/file/text paths | planned |
| FR-3.2 encrypted concealed codes, reveal/copy/used/undo | 03, 16 | Used-state persistence, intentional reveal, copy does not mark used | planned |
| FR-3.3 edit/delete confirmations and recovery guidance | 15, 16, 19 | Destructive/cancel paths and help copy | planned |
| FR-3.4 blanks/duplicates/large/encoding/source/deletion handling | 15, 16, 18 | Normalization/limits and association tests | planned |
| FR-4 folder name/color/search/membership/navigation/unfiled | 17 | Many-to-many, six colors and keyboard checks | planned |
| FR-4 persistence/backup and remove-folder-preserves-accounts | 11, 17 | Restart/round-trip/delete tests | planned |
| FR-5 retention policy, restore, single/all permanent delete | 18 | No-expiry copy, restore/conflict/confirmation tests | planned |
| FR-5 account/metadata/codes restored atomically and no swipe-only UI | 18, 20, 21 | Transaction faults and real button/keyboard tests | planned |
| FR-6.1 easy encrypted portable export of every supported entity | 11, 12 | Format docs, complete encrypted round-trip | planned |
| FR-6.2 version/integrity/passphrase/preview/conflict/merge/replace | 11, 13, 20 | Negative, transactional and UI tests | planned |
| FR-6.3 atomic export, destination, explicit overwrite, safe errors | 11, 12, 20 | Interrupted write/overwrite/file-picker checks | planned |
| FR-6.4 reference .vault discovery, no unsupported claim | 22 | Evidence-based compatibility report | planned |
| FR-7.1 user/idle lock, unlock/recovery, optional OS protection | 03, 05, 19, 20 | Key design, lifecycle and lock-race checks | planned |
| FR-7.2 counts/status/version/startup/preferences/English UI | 19, 21 | Settings/restart/failure screenshots | planned |
| FR-7.3 system time/resume, local guidance, no remote clock mutation | 02, 10, 19 | Injected clock and OS behavior evidence | planned |
| FR-7.4 explicit optional startup and failures | 19, 23 | Registration toggle, denial, package smoke test | planned |
| FR-8 no Google/app accounts/cloud/sync/multilingual/translation | 01, 04, 19–21 | Source/UI and offline network audit | planned |
| FR-8 all persisted user data encrypted, no uploads | 03, 11, 20 | Ciphertext/artifact/traffic evidence | planned |
| PRD §7 compact reference design across all pages/states | 04–05, 07–10, 12–19, 21 | Per-task checklist and independent comparison | planned |
| PRD §7 resize/scale/contrast/focus/labels/keyboard/Narrator | 04, 21 | Multi-size/DPI and accessibility report | planned |
| PRD §8 threat model, crypto/KDF/versioning and key lifetime | 01, 03, 11, 20 | Security design and separate-agent review | task01 foundation verified; remaining owning tasks pending |
| PRD §8 no secrets/PII in diagnostics, no telemetry or raw captures | 03, 06–08, 20 | Sentinel file/log/crash/network audit | planned |
| PRD §8 untrusted parsing bounds, schema checks, transactions | 02, 03, 06, 11, 20 | Adversarial/fault-injection test results | planned |
| PRD §8 locked UI/accessibility, clipboard and endpoint limits | 05, 10, 16, 20 | Lock checks and accurate limitations docs | planned |
| PRD §9 entities/relations/migrations/rollback/no secret filenames | 02, 03, 11, 18, 20 | Migration/referential integrity/filename tests | planned |
| PRD §10 deterministic clock, separate layers/adapters, offline | 01–03, 06–07, 19–20 | Architecture and adapter tests | task01 foundation verified; remaining owning tasks pending |
| PRD §11 full correctness/workflow/recovery/privacy/UI/reliability | 02–21, 23–24 | Consolidated regression and QA report | planned |
| Clean Windows package, English usage/developer/recovery docs | 23 | Package build, clean-profile smoke/recovery | planned |
| Commit/push main, final PRD and task/evidence reconciliation | 24 | Clean status and remote/local SHA equality | planned |

## Execution updates

Task01 accepted: implementation 7502fa4, independent review_01 re-review approved, Windows CI run 43227714011 succeeded, six local tests and live inert spike evidence in docs/qa/task01/. No full multi-owner requirement row is yet verified. Replace each row's state with verified only after its acceptance checks pass; link commands, report paths and commit IDs here or in owning task evidence. Hardware or reference limitations must stay explicitly unverified. Optional reference-format compatibility may be unsupported with a completed discovery report; mandatory core features may not be silently waived.

