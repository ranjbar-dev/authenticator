# Task23 implementation evidence

13 November 2026. Ready for independent package/security/UI review; this is an
author report, not task acceptance. No task statuses, commits or pushes by author.

## Changes and artifact

`build.rs`, locked `winresource 0.1.34`, original SVG/PNG/multi-size ICO and Windows
manifest embed Authenticator identity, version 0.1.0, English resources,
asInvoker/uiAccess=false and PerMonitorV2. The production viewport has the same
icon. No acquisition, credential, crypto, navigation or persistence behavior was
changed. Added project license texts, dependency/font/runtime notice provenance,
the stdlib Python build/validation scripts and English user/build/security docs.
CI adds release ZIP build, regressions, validation and public artifact upload;
no GitHub release publication or user-data upload. README/architecture describe
the actual choices. `.gitignore` excludes Python bytecode and native backups.

- Artifact: `dist/Authenticator-0.1.0-windows-x86_64.zip` (6,023,310 bytes).
- ZIP SHA256: `56f6786cb72a1c7f53f294667dc071334a3c97c47dfbd90674b73b2f2d963f9b`.
- Adjacent `.zip.sha256` agrees. 364 files, including 355 actual notice texts and
  184 dependency/runtime/font inventory records. Only the production executable,
  user docs, licenses/notices and build manifest are present.
- `build-info.json` inside the ZIP records source
  `c18ebad584e129a0d6ff02cb08bb2f458f221826`, **source_dirty=true**, lockfile hash,
  relative file hashes/sizes, Cargo/rustc 1.98.0, Python 3.13.3, target, static CRT
  flag and PE linker version 14.44. It contains no developer cache/user paths.
- Final ZIP executable SHA256:
  `99a2df81e1792bd6eda93ae2be0b16c757402e9b6661aa1aea102e382e6ec9df`.
  Its bytes equal the executable used for capture/copy/restore/startup/restart.
  Rebuild after the parent's reviewed commit for clean-HEAD attribution. These
  repeatable build steps do not establish bit-identical reproducibility.

## Build, validation and notices

All shell commands used RTK. Every Cargo build/test subprocess used fresh owned
project-target TEMP/TMP on D:, process-local build jobs=2; global Cargo/cache/
toolchain configuration and cache junctions were preserved. The packager fixes
its own CARGO_TARGET_DIR to avoid consuming an ambient target-directory binary.

| Check | Actual result |
|---|---|
| `rtk proxy python scripts/package_windows.py --replace` | Final default release build and ZIP passed; matching prior checksum/manifest verified before replacement. |
| `rtk proxy python scripts/package_windows.py --validate dist/Authenticator-0.1.0-windows-x86_64.zip` | Safe Windows paths, bounds, allowlist, mandatory files, CRCs and exact manifest hashes passed. |
| `rtk proxy python scripts/test_package_windows.py` | 10 passed: tampering, duplicate manifest, traversal, Windows path alias, non-allowlist/user-data payloads, AND license coverage, notice-only rejection and changed discrepancy hash. |
| `cargo fmt --all -- --check` under RTK/process-local D: TEMP | Passed after formatting build.rs. |
| `cargo clippy --locked --bin Authenticator -- -D warnings` under RTK/process-local D: TEMP | Passed; new build dependency/resources and production icon loader compiled. |
| `cargo test --locked --bin Authenticator` under RTK/process-local D: TEMP | 1 passed; the ignored subprocess entry was exercised by that test. |
| PE/resource and Windows FileVersionInfo | x64 PE32+ GUI; icon/group-icon/version/manifest types 3/14/16/24; product/description/original filename Authenticator/Authenticator/Authenticator.exe; file/product version 0.1.0. Embedded privilege/DPI XML parsed and checked. |
| Static CRT/imports | An initial ordinary release imported VCRUNTIME140.dll. The packaged explicit-target static CRT build imports Windows system DLLs only; no external vcruntime/msvcp/concrt, Rust, Python or Node DLL. Actual launched bytes checked. |
| Source attribution/privacy | Dirty flag true as expected; executable/ZIP/checksum agreement and no developer-cache paths in build manifest checked. No examples, test records, vaults, backups, PDBs, logs or QA images in ZIP. |

The pre-package parent-reported hosted run 43446743225 passed at
3b5185187894d8b1be1d6b0a6abe01dd33671ce1 (226 ordinary tests and other checks).
It does **not** cover these new changes. New hosted package CI is pending the
parent's commit/push; no hosted success is invented here.

Notices retain actual full texts and declared expressions, checking every AND
clause/one supplied OR branch. Epaint fonts include code MIT/Apache alternatives
plus actual OFL and Ubuntu font license texts; dpi includes Apache AND MIT and
Unicode dependencies include Unicode text. Rust's installed standard-library
copyright inventory and bundled Noto OFL are included. Missing monorepo licenses
are vendored with hashes. Egui crates whose recorded commits were unavailable
use the official 0.37.0 tag resolved to immutable
`841a328ba2dd8f6996beb7189ebb7fc00e668a2a`; eframe's own recorded commit was
retrievable. **Zero current-branch fallbacks remain**. Provenance never asserts
published-code equality from a license fetch.

Quircs 0.10.3 declares MIT in Cargo metadata, but its exact published LICENSE is
the ISC grant. Both facts are recorded; the bundled LICENSE SHA256 is pinned to
`166760d656f6bfc29dfc76a7f763e529bde0ed22dde2b20cf6b6c4b14a138a9a`.
No substitute MIT grant is invented. The maintenance tool is not run by package
builds or CI. See the linked primary sources in windows-build.md and provenance
records. This is attribution evidence, not a legal certification.

## Actual packaged pointer/keyboard journeys

Used the Computer Use skill and medium-integrity helper, uniquely selecting the
owned extracted production executable under
`target/task23-extracted-b9b3b896-65ea-4457-9c50-3d845f990809/`.
Pre-existing debug QA processes were preserved. Only disposable/public fixtures
were used. No QR/secret/code/recovery screenshot was written to disk. HMAC and
clipboard comparisons returned booleans; test phrases/seed/code values are not
in repository evidence.

1. Helper-launched production UI showed Create vault. Create, exit/relaunch
   locked, unlock and F10-with-no-debug-gallery checks passed. Windows redirected
   its Known Folder into the Codex MSIX host's `LocalCache/Local/Authenticator`.
   The shell's ordinary Known Folder had been inspected read-only and was absent;
   that shell result alone did not establish the inherited package context.
   Actual new vault creation/file times and paths were subsequently recorded.
2. Real Screenshot to Scan drag over the separate public QR fixture, editable
   concealed-secret review and explicit Save succeeded. Initial mis-mapped
   selections returned No QR and left the vault empty. The helper normalizes
   the 6400-pixel virtual desktop overlay to 2560 pixels (factor 2.5); selecting
   the fixture at physical -1824,411 through -1392,843 with normalized x38.4–211.2
   fixed the automation mapping. No capture/backend source change was needed.
3. The saved current code matched an independent HMAC calculation in memory.
   Real Copy displayed success and Windows clipboard matched the current or
   boundary code, boolean only. The app-owned clipboard copy was absent after
   exit. This does not claim retention control over Windows history/managers.
4. Production encrypted export wrote a 357-byte AUTHPBK1 file into owned ignored
   target. Its SHA256 is
   `839c2cead8bb78234760d52ba394bcc0b8e0202ac803c2bc77af52c5837175d5`.
   After closing, only the proven new vault.bin was preserved with checked hash.
   A second UI-created vault used a different local passphrase/key. The backup
   was independently authenticated, previewed (0 current/1 incoming/1 resulting
   account), reviewed and applied; restored code matched. Envelope/ciphertext
   differed from the first vault. Ctrl+L concealed account/code immediately.
5. Ordinary File Explorer navigated only to the owned extracted folder and
   double-clicked its visible Authenticator.exe filename. Process 3648's
   GetPackageFullName returned APPMODEL_ERROR_NO_PACKAGE (15700). Its Create UI
   and ordinary `%LOCALAPPDATA%/Authenticator/session.lock` confirmed the normal
   launch context; no vault pre-existed. The original icon was visible in Explorer
   and the native title bar. A third disposable vault was created there, and the
   same exported backup restored successfully across the two data contexts.
6. With the real Run value `Authenticator.ranjbar-dev` observed absent, actual
   Settings Enable Auto Start registered exactly the quoted owned executable
   path in current-user Run. Disable restored absence, independently read from
   the registry. No unknown registration was replaced. Actual sign-in delivery
   was not attempted.
7. Ordinary exit and Explorer relaunch opened locked; the normal-context phrase
   unlocked the persisted restored account, whose code again matched independent
   HMAC. Real Ctrl+L concealed it. This host has development tools installed, so
   this is not a clean-machine/no-Rust-installed claim.

Clean locked frames only:
`packaged-locked.jpg`, `packaged-restored-locked.jpg`,
`packaged-normal-locked.jpg`. Their capture purpose is review evidence, not UI
reference substitution. No new page layout was introduced in this task.

## Offline attempt, ownership and remaining gates

**Offline execution remains unverified.** A bounded alternative created two
unique exact-executable inbound/outbound Block rules. BFE/MpsSvc ran, all
ActiveStore profiles were enabled with local rules allowed, and the rules were
present/enabled for the exact owned program. Deeper inspection, however, returned
`EnforcementStatus=CategoryDisabled` for both. Thus presence was not effective
blocking proof. No global category/profile/network/policy was changed. Both
owned GUID rules were removed and ActiveStore absence verified. The host remained
online throughout; these UI journeys are not described as offline passes.

After backup/restart checks, app and owned QR fixture were closed and absence
verified. Current ciphertext hashes, exact source/destination paths, directory
members and session creation times guarded preservation/cleanup. First and
second MSIX vaults and the normal-context vault are preserved in ignored target,
as is the portable backup. Only proved own empty session.lock files were removed,
non-recursively. The ordinary data directory, observed absent before launch,
was removed only after verified empty; ordinary baseline is again **absent**.
The redirected Codex directory is left empty; no Codex package parent/unknown
directory was removed. Existing debug QA processes, unrelated outputs and all
global Cargo/cache settings were preserved. Exact non-secret ownership records
and cleanup script remain `target/task23-ui-owned.json` and
`target/task23-cleanup-owned.ps1`. Startup is absent; own firewall rules absent.

Read-only `Get-VM` returned 0 VMs; VBoxManage/vmrun/qemu commands and running VM
processes were absent. No ready separate clean Windows-user session was available
through enabled tooling; no global users or VMs were created. Another Windows
user/VM recovery, truly networking-disabled runtime, a no-development-tools
machine, actual logon/OS lock/suspend, physical mixed DPI/Narrator/high contrast
and the broader full state/scale matrix remain open. Existing owning-task
limitations are preserved. Parent-owned independent review and new hosted CI
must follow; no completion or compatibility/security certification is claimed.
