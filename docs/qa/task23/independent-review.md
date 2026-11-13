# Task23 independent review

13 November 2026. Distinct read-only reviewer; no implementation, task-status,
commit or push changes. **Scoped source/package review approved after the
follow-up fixes recorded below.** Initial archive-validation findings are
retained as history and are now closed.
The actual submitted archive passes its checks and contains none of the rejected
examples; these findings concern the validator's promised Windows path boundary.

## Findings

1. **P2 — reserved Windows aliases accepted**, `scripts/package_windows.py:376`.
   Manifest-consistent `licenses/COM¹.txt`, `licenses/LPT².txt`,
   `licenses/CONIN$` and `licenses/CONOUT$` were accepted. Python 3.13's
   `os.path.isreserved` returned true for all four. Extend device-name checks to
   superscript 1/2/3 and console aliases while preserving Python 3.11 support.
2. **P2 — file/ancestor collisions accepted**, archive-name validation.
   Both `licenses/collision` and `licenses/collision/child.txt` were accepted as
   files in one manifest. They cannot both be extracted faithfully on Windows.
   Reject any case-insensitive file path that is another file's ancestor,
   independent of archive ordering.
3. **P2 — raw NUL name hidden by zipfile normalization**, archive-name validation.
   A bounded test ZIP with matching local/central raw name
   `licenses/LICENSE\0suffix` and a manifest naming `licenses/LICENSE` passed.
   `ZipInfo.orig_filename` retained NUL while `filename` was truncated. Check raw
   names before trusting normalized names; reject normalization differences and
   control characters. The ordinary per-component control check sees only the
   truncated name. No extraction or user-data operation was performed.

These are small validator fixes with meaningful regression cases. Re-review was
required after implementation. They do not justify describing the current
package as a malicious/user-data archive or changing vault/backend behavior.

## Independently checked

- `rtk proxy python scripts/test_package_windows.py`: **10 passed**.
- `rtk proxy python scripts/package_windows.py --validate dist/Authenticator-0.1.0-windows-x86_64.zip`:
  passed archive bounds, paths, CRCs and manifest hashes. Additional manifest-
  consistent probes above reproduced the three gaps in owned ignored target
  scratch; no existing output was replaced.
- Python stdlib hash/ZIP/PE inspection confirmed ZIP **6,023,310 bytes**, SHA256
  `56f6786cb72a1c7f53f294667dc071334a3c97c47dfbd90674b73b2f2d963f9b`,
  adjacent checksum agreement and executable SHA256
  `99a2df81e1792bd6eda93ae2be0b16c757402e9b6661aa1aea102e382e6ec9df`.
  Packaged executable bytes equal the explicit-target release executable.
- Cargo's current filtered metadata/tree independently matched the inventory:
  **183 graph packages including this application; 182 dependency records plus
  font/runtime records = 184 records**, 355 notice texts, 364 archive files.
  Metadata subprocesses used fresh owned D: target TEMP/TMP and build jobs=2.
  All notice hashes and the Cargo.lock hash agree. No full Rust suite was rerun.
- PE inspection confirmed x64 PE32+, Windows GUI, icon/group-icon/version/manifest
  types 3/14/16/24, asInvoker/uiAccess=false, PerMonitorV2/PerMonitor and an empty
  Authenticode certificate directory. Imported DLL names are Windows platform
  DLLs, with no external VC redistributable, Rust, Python or Node DLL import.
  Independent Windows FileVersionInfo reports ProductName/FileDescription
  Authenticator, OriginalFilename Authenticator.exe and file/product version
  0.1.0. `rtk git diff --check` passed.
  This is not a clean-machine graphics/OS compatibility test.
- Source review covered Cargo/lock changes, build.rs, the three-line viewport
  icon integration, original SVG/PNG/ICO generation, manifest, complete packaging
  and regression scripts, maintenance notice-fetch script, provenance, CI and
  user/build/security/package docs. `winresource = "=0.1.34"` is build-only; four
  lock packages were added. No acquisition, crypto, persistence or account/cloud
  runtime behavior was added by this task.
- The packager fixes its target directory and static CRT flags, stages only
  explicit public payloads, verifies prior checksum/manifest before replacement
  and performs no recursive deletion. CI uploads public ZIP/checksum artifacts
  after checks, with no Release publication or runtime vault upload.
- Actual license texts, AND/OR selection and font/runtime notices are retained.
  In particular dpi selects Apache AND MIT; default fonts retain code-license
  alternatives plus OFL and Ubuntu terms. All 15 vendored provenance records
  were opened; none records a current-upstream fallback. Egui notices use the
  pinned release-tag commit where the published commit was unavailable. The
  maintenance tool can explicitly record future fallbacks; docs disclose that.
  Quircs' Cargo MIT versus exact published ISC-text discrepancy is explicit and
  hash-pinned, with no invented MIT grant. This is attribution/source review,
  not legal certification.
- Build metadata honestly identifies source `c18ebad584e129a0d6ff02cb08bb2f458f221826`
  with `source_dirty=true`, and contains no developer/cache absolute paths. A
  parent-owned clean-HEAD rebuild and new hosted CI remain necessary. Repeatable
  steps are documented without a bit-identical reproducibility claim.
- Update/uninstall instructions keep vault storage independent of the extracted
  app directory, explain startup disable/re-enable and independent encrypted
  backup recovery, and document redirected package contexts. Supported native
  AUTHPBK1 versus unsupported reference `.vault` is clear. Security limitations
  and remaining acceptance are explicit rather than certified away.

## UI and native evidence scope

Applied `ui-consistency:verifying` against the Task23 identity-only change and
the existing shared-shell foundation. Opened all three actual production images:
`packaged-locked.jpg`, `packaged-restored-locked.jpg` and
`packaged-normal-locked.jpg`, plus the original icon. Compared the whole set:
three matching locked shells; no task-introduced visual deviation found. Native
title/icon, holder order, selected tab, readable copy, concealed empty field,
visible focus and primary/secondary action contrast are consistent. The cursor
glow in two images is capture tooling, not product styling. Build/CI/scripts are
not pages of this kind.

Reused the dispatcher's Task04 source calibration: an outside-worktree TITLE
weight 400/600 plant was detected by a different agent. Its copied native launch
failed, so that calibration never proved pixel or interaction sensitivity. No
new plant was run by this checker. This review opened rendered locked evidence;
it did not independently operate the native app, measure every pixel/theme pair
or verify pointer/keyboard, assistive technology, other states, widths or DPI.
The author's real packaged capture/edit/save, HMAC/copy, export/fresh-key restore,
ordinary Explorer launch, startup toggle and restart records remain attributed
author evidence. The bounded cleanup script preserves owned ciphertext using
exact path/hash/time/member guards and contains no recursive deletion.

Offline runtime is **unverified**: the attempted exact-app firewall rules had
CategoryDisabled enforcement and were removed. Another Windows user/VM,
no-development-tools host, effective network-disabled execution, actual OS
lock/suspend/logon, physical mixed DPI, Narrator/high contrast, the full state/
scale matrix and precise Windows reference parity remain open. No source/artifact
check or same-user fresh-key restore closes those gates. Do not mark full Task23
or v1 release acceptance complete on this review alone.

Review cost estimate: 3 rendered locked states plus 1 icon; approximately 45
project/source/provenance files opened, 3 discovery/diff searches, two required
script checks and targeted hash/graph/PE/path probes. Counts are estimates where
not maintained individually. Only this review report was added by the reviewer.

## Follow-up re-review: all three findings closed

13 November 2026. Independently read the bounded validator and regression changes
after the parent implemented the fixes. Raw `orig_filename` differences and
control characters are rejected before `testzip()` or any payload/manifest read.
The reserved-name regex covers case-insensitive CONIN$/CONOUT$ and COM/LPT
superscript 1/2/3 aliases, including extensions, without a Python 3.13-only API;
it remains compatible with the documented Python 3.11 minimum. Ancestor checks
use the complete casefolded filename set, so file/directory collisions are
rejected in either archive ordering and across casing differences.

Independently reran `rtk proxy python scripts/test_package_windows.py`:
**13 passed**, including all device aliases, both ancestor ordering/case cases
and a real raw central/local NUL-name fixture. Reran
`rtk proxy python scripts/package_windows.py --validate dist/Authenticator-0.1.0-windows-x86_64.zip`:
**passed**. No remaining material finding in this bounded change. The existing
dirty-source archive and previously named environment/manual gates are unchanged;
this approval is not full Task23/v1 acceptance or clean-HEAD/hosted-CI evidence.
Follow-up cost: 2 source files and the existing report opened, one diff inquiry,
2 script runs; no additional native UI operation.
