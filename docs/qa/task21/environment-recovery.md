# Native runtime disk-space recovery

Parent integration, 13 November 2026. The fresh Task21 documented `node_repl` initialization/import of `@oai/sky` failed before any inventory/app/input/screenshot: `failed to write kernel assets: There is not enough space on the disk. (os error112)`.

Read-only drive inventory found C: at0free bytes and D: at96,810,950,656free bytes. Project build targets are on D:, so deleting them would not solve the C: shortage. The three known project processes were unrelated lifecycle preview44740 and retained old test processes30688/32032; none was stopped or modified.

The parent recovered space only from regenerable public crates.io download archives. No user Temp/history, vault, source cache, toolchain, Cargo configuration, credential file, Node runtime file or unrelated build output was deleted. No Cargo/rustc process was active, and Task21 held Cargo while the cache operation ran.

1142 `.crate` archives totaling194,962,999bytes were copied to a fresh ignored D: backup. Each original/copy SHA256 was compared before the corresponding literal C: archive file was removed. All source/destination absolute paths, non-link roots and containment were checked; no recursive C: deletion occurred. Afterward C: had174,727,168free bytes. Node/Sky initialization and fresh window inventory then succeeded; no LockApp desktop was reported. That recovers the new disk-space failure, not proof of the cause or resolution of older GetCursorPos denials.

The subsequent offline fixture build showed Cargo still requires compressed archives even with expanded sources available. To preserve offline builds without refilling C:, the verified backup was renamed on D: to:

`D:/CodexCaches/authenticator-public-cargo-74576573-d97a-4062-9d53-5d693f2e3008/index.crates.io-1949cf8c6b5b557f`

The original public archive path `C:/Users/root/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f` is now a directory junction to that exact D: cache. Only the verified empty C: archive directory was removed to create the link. Inventory1142 and the required accesskit archive were verified through the original path; C: had183,037,952free bytes. The destination is outside project `target`, so ordinary project cleaning does not remove it. Expanded source paths, toolchains and configuration are unchanged. All archive bytes are retained; this is cache placement, not an app/runtime or security-setting change.

Reproducible recovery scripts remain ignored at `target/recover-cargo-archives.ps1` and `target/link-cargo-archives.ps1`. Task21 was told to retry the now-satisfied offline build once, then one normal documented own-fixture native launch. App launch, actual workflow, capture and clipboard outcomes must be recorded separately; this recovery alone establishes none of those gates.
