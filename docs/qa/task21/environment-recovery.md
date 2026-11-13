# Native runtime disk-space recovery

Parent integration, 13 November 2026. The fresh Task21 documented `node_repl` initialization/import of `@oai/sky` failed before any inventory/app/input/screenshot: `failed to write kernel assets: There is not enough space on the disk. (os error112)`.

Read-only drive inventory found C: at0free bytes and D: at96,810,950,656free bytes. Project build targets are on D:, so deleting them would not solve the C: shortage. The three known project processes were unrelated lifecycle preview44740 and retained old test processes30688/32032; none was stopped or modified.

The parent recovered space only from regenerable public crates.io download archives. No user Temp/history, vault, source cache, toolchain, Cargo configuration, credential file, Node runtime file or unrelated build output was deleted. No Cargo/rustc process was active, and Task21 held Cargo while the cache operation ran.

1142 `.crate` archives totaling194,962,999bytes were copied to a fresh ignored D: backup. Each original/copy SHA256 was compared before the corresponding literal C: archive file was removed. All source/destination absolute paths, non-link roots and containment were checked; no recursive C: deletion occurred. Afterward C: had174,727,168free bytes. Node/Sky initialization and fresh window inventory then succeeded; no LockApp desktop was reported. That recovers the new disk-space failure, not proof of the cause or resolution of older GetCursorPos denials.

The subsequent offline fixture build showed Cargo still requires compressed archives even with expanded sources available. To preserve offline builds without refilling C:, the verified backup was renamed on D: to:

`D:/CodexCaches/authenticator-public-cargo-74576573-d97a-4062-9d53-5d693f2e3008/index.crates.io-1949cf8c6b5b557f`

The original public archive path `C:/Users/root/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f` is now a directory junction to that exact D: cache. Only the verified empty C: archive directory was removed to create the link. Inventory1142 and the required accesskit archive were verified through the original path; C: had183,037,952free bytes. The destination is outside project `target`, so ordinary project cleaning does not remove it. Expanded source paths, toolchains and configuration are unchanged. All archive bytes are retained; this is cache placement, not an app/runtime or security-setting change.

Reproducible recovery scripts remain ignored at `target/recover-cargo-archives.ps1` and `target/link-cargo-archives.ps1`. Task21 was told to retry the now-satisfied offline build once, then one normal documented own-fixture native launch. App launch, actual workflow, capture and clipboard outcomes must be recorded separately; this recovery alone establishes none of those gates.

## Later linker disk exhaustion and public source-cache relocation

After the remaining native journeys finished, a test-only compiler-compatibility
change could not link: LNK1108 at a C: temporary linker file; C: again had0free
bytes while D: had about96GB. No Cargo/rustc/link process remained. This is an
observed disk/link failure, not a functional test failure or an explanation for
earlier cursor/clipboard behaviors.

Parent copied the explicitly named public expanded crates.io source cache,
refused reparse points, compared file counts/lengths and every file's SHA256,
then replaced that exact original non-link directory with a junction. All
35,127files/778,635,483bytes are retained at the permanent location:

`D:/CodexCaches/authenticator-public-source-55e53314-1fc7-4774-857e-2c04d5d308fa/index.crates.io-1949cf8c6b5b557f`

The original path is
`C:/Users/root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f`.
The destination is outside project target/clean operations. Toolchains, Cargo
configuration, credentials, user Temp/history and vaults were untouched. Unlike
the earlier archive-only operation, expanded source **placement** now changes,
with original paths and all verified bytes preserved. C: recovered830,119,936bytes.

The first verification invocation lacked `Get-FileHash`; it stopped before any
original removal. The ignored recovery script `target/relocate-public-cargo-source.ps1`
resumed the existing copy with .NET SHA256 and completed all verification before
the literal recursive cache removal/junction creation. Permanent `relocation.json`
records paths/count/bytes. Restoring placement requires preserving/verifying this
cache before replacing the junction; do not delete its target via project cleanup.

Build/test subprocesses now set TEMP/TMP to fresh owned project D: scratch paths,
without changing global environment settings. Retried current-source clipboard
tests:6passed,1direct helper ignored/exercised by parent,204filtered,4.43s.
Scratch: `target/task21-build-temp-e4be99af-99a4-4e3c-89b7-c4d090845359`.

This recovery does not close hardware/offline/native QA gates by itself.
