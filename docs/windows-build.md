# Windows build and package

The supported build target is `x86_64-pc-windows-msvc`. Install Rust stable
(MSRV 1.95), Visual Studio C++ Build Tools and the Windows SDK, including `rc.exe`.
The locked build dependency `winresource 0.1.34` locates SDK resource tools and
embeds the original icon, English version information, asInvoker privileges and
PerMonitorV2 manifest. Numeric manifest version derives from Cargo's version.
See Microsoft's [application manifest specification](https://learn.microsoft.com/windows/win32/sbscs/application-manifests).

Python 3.11+ is required for packaging, not for running the app. Cargo.lock pins
dependencies. From a Windows checkout:

```powershell
python scripts/package_windows.py
python scripts/package_windows.py --validate dist/Authenticator-0.1.0-windows-x86_64.zip
```

This workspace requires RTK, so prefix the commands here with `rtk proxy`.
Elsewhere the underlying commands do not require RTK. `--replace` replaces only
a matching prior package whose checksum still agrees; an unknown or modified
output is preserved. `--skip-build` exists for local archive checks only. A normal
release/package validation should build afresh through the default command.

The script creates fresh owned project-target scratch and sets TEMP/TMP only
for its own process/children. It fixes packaging RUSTFLAGS to
`-C target-feature=+crt-static` and builds the explicit Windows target, following
[Rust's CRT linkage guidance](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes).
This keeps host build tools separate and avoids shipping an unbundled VC runtime.
PE import/resource checks verify that result instead of relying on the flag alone.
Custom ambient encoded Rust flags are removed only in that subprocess.

Output is `dist/Authenticator-<version>-windows-x86_64.zip` plus `.zip.sha256`.
No recursive deletion is used. The script copies an explicit production binary,
three short user documents, project licenses and collected notice texts into an
owned staging directory. It never packages target fixtures, vaults/backups, QA
captures, example executables, debug symbols or arbitrary source directories.
Archive validation checks bounded sizes, safe/case-unique paths, CRCs and every
manifest file hash. Existing output replacement verifies its previous manifest
and checksum; source/build metadata clearly records a dirty tree.

The executable has a Windows GUI subsystem, x64 machine type, version/icon and
manifest resources. Windows system DLLs and a working desktop graphics driver
remain prerequisites. The package is unsigned, not Store/MSIX signed. The build
records exact compiler, source commit, dirty flag and Cargo.lock digest. These
steps are repeatable; timestamped toolchains/linkers/ZIPs are not promised to yield
bit-identical bytes across machines or runs.

## Notices and source provenance

The Windows normal/build graph is enumerated with Cargo's filtered metadata/tree.
Every package needs a declared license and supplied text; missing texts fail the
package build. The script copies all discovered license/copyright/notice files,
including epaint default font notices and bundled Noto OFL text. The installed
Rust standard-library copyright notice is included (part of rustc's component,
also present with the minimal toolchain profile).

Some published crates omit their monorepo root licenses. `assets/notices/`
contains public upstream texts and SHA256 provenance for those exact crate names
and versions. `scripts/vendor_missing_notices.py` is a development maintenance
tool, not a runtime/build-network dependency. It first uses the published crate's
recorded Git revision. If that revision is not publicly retrievable, the matching
release tag is resolved to an immutable commit for the license texts. Only if
neither source is available is an explicit current upstream revision recorded as
a fallback. Every provenance record preserves the crate's declared SPDX
expression and **does not** assert code-version identity. The current bundled
egui 0.37.0 notices use its pinned release-tag commit, not the current branch.
Review that provenance on dependency updates. The inventory is evidence and
attribution, not a legal compliance certificate or a claim that every dependency
is MIT-only. The app's original icon is covered by the project dual license.

The declared SPDX expression and the text families selected to satisfy each AND
clause/one OR branch are both recorded. Quircs 0.10.3 is an explicit discrepancy:
Cargo metadata says MIT, but its published LICENSE contains the ISC grant.
The exact text and hash are retained with both labels and a discrepancy note;
the script fails if that reviewed version/text changes. See the
[published license](https://docs.rs/crate/quircs/0.10.3/source/LICENSE) and
[ISC text](https://spdx.org/licenses/ISC.html).

Icon regeneration uses the committed SVG geometry and optional development
Pillow: `python scripts/generate-icon.py`. PNG/ICO outputs are committed, so users
and CI package builds need no image-generation tool or Pillow.

## Validation and CI

Run formatting, strict Clippy and the unit/integration suite as documented in the
repository README. CI separately builds/validates the release ZIP and uploads
only `dist/*.zip` and checksums as workflow artifacts. It does not publish a
GitHub release or upload test profiles. A hosted pass and a developer-machine
launch are different evidence from a clean Windows-user/VM test without build
tools; unperformed environment checks remain explicitly unverified.
