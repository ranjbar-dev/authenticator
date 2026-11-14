# Two full release rebuilds

The final shared-focus source produced identical unsigned EXE and ZIP bytes in two actual compiler/linker/package builds from empty host and target release outputs. This is measured same-environment reproducibility, not cross-host or cross-toolchain identity.

| Item | Build 1 | Build 2 |
|---|---|---|
| Duration including packaging | 205.288 seconds | 219.703 seconds |
| Fresh compilation log lines | 183 crates | 183 crates |
| EXE bytes | 12,179,456 | 12,179,456 |
| EXE SHA256 | `9656ed317d6b2e36a96cf17ac5c0d2729680cb6abb850941c295f7418272978e` | same |
| ZIP bytes | 6,026,388 | 6,026,388 |
| ZIP SHA256 | `60f940fd5364f999f7335500487e0e79024798aed0ab0f93686a6c21517e9b78` | same |

The default packager builds `cargo build --release --locked --target x86_64-pc-windows-msvc`, then validates PE/static CRT/resources/provenance, generates deterministic notices/metadata and validates the complete archive. RUSTFLAGS are exactly `-C target-feature=+crt-static -C link-arg=/Brepro`. The script does not use `--skip-build` for this proof.

The [exact driver](full-repro-driver.py) was executed from `target/follow24-full-repro.py` using `rtk proxy python -X utf8 target/follow24-full-repro.py`. Its archived copy preserves those original path assumptions; copy it back into an owned target script before reproducing. The [result](full-repro-result.json) includes every bounded preservation move, directory inventory and disk headroom. [Source inputs](full-repro-source-inputs.json) were checked unchanged after each build. Local logs and binaries remain under `target/task24-full-repro-5erftluo` (`compile-1.log`, `compile-2.log`, `build-1.exe/.zip`, `build-2.exe/.zip`).

Before each round, the driver recursively rejected links/junctions and possible vault/backup/evidence files, checked exact absolute source/destination containment and same volume, and preserved only `target/release` and `target/x86_64-pc-windows-msvc/release` in a fresh owned GUID root. All previous and first-round outputs remain intact. No entire target clean/deletion was performed; encrypted QA roots were untouched. Each next build started at the original empty release paths, forcing host proc macros/build scripts, dependencies, own code and link again. The first driver attempt refused unfamiliar Cargo lock marker names and `Authenticator.pdb` before any move; after actual artifact-name inspection, the narrowly extended allowlist admitted only those known build artifacts. It was not counted as a build.

The environment is Rust 1.98.0 (`88d9e12ae178fab0fb5cc050a94da85685d449ea`, LLVM 22.1.8), Cargo 1.98.0 (`797e8a9bc`), x86_64 MSVC, Python 3.13.3 and zlib 1.3.1. PE linker version is 14.44. Installed Visual Studio 2022 Build Tools has MSVC directory 14.44.35207 and link.exe file version 14.44.35208.0; the build did not log the exact linker invocation path, so that installed candidate is distinguished from observed PE metadata. The resource build output explicitly selects Windows SDK `10.0.26100.0/x64/rc.exe`, whose file version is 10.0.26100.3916. [Environment evidence](environment.json) records hashes and the actual resource output. The installed Rust standard library, cached crate source bytes and fixed original paths are environment inputs, not freshly rebuilt standard-library source.

[Build metadata](package-build-info.json) records source commit `2e2da2cba35e99d6d6d218471a12f48d67e7abc0`, **source_dirty=true**, epoch 1794607268 / 2026-11-13T22:01:08+00:00, static CRT flags and unsigned status. The epoch is a source timestamp, never described as actual wall-clock build time. ZIP member order, UTC DOS timestamps, permissions and generated JSON are stable. Narrow Git attributes preserve exact packaged/embedded text and pinned upstream licenses through Windows autocrlf checkouts. Relative license identities remove absolute-root ordering dependence without dropping exact provenance verification.

Earlier own-package forced-link pairs remain in `target/task24-repro-b0ml4ghc` (9165… EXE before focus changes), `target/task24-repro-icslvimt` (b7ac… EXE before the programmatic-focus correction) and `target/task24-repro-ass5cd4x` (final 9656… EXE, dependencies cached). They are historical supporting measurements. The final full pair supersedes the cached-dependency limitation.

After a clean parent commit, the source commit/epoch/dirty metadata changes, so the ZIP must be rebuilt and its new hash recorded. Native proof may be reused only if the final packaged EXE bytes are independently identical to 9656…; otherwise a new actual binary smoke is required. Cross-host/path/linker/SDK/compiler/Python/zlib changes have not been tested for byte identity. `/Brepro` follows [LLVM's official usage](https://github.com/llvm/llvm-project/blob/main/llvm/cmake/modules/HandleLLVMOptions.cmake); source epoch semantics follow the [SOURCE_DATE_EPOCH specification](https://reproducible-builds.org/specs/source-date-epoch/).
