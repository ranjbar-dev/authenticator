# Authenticator

A Windows desktop authenticator (TOTP/HOTP) with screenshot-based import, Windows Hello unlock and a light/dark purple UI. See [PRD.md](PRD.md) for the full spec.

![icon](assets/icon/png/app-icon-128.png)

## Requirements

- Windows 10 1809+ / Windows 11 (x64 or ARM64)
- [.NET 10 SDK](https://dotnet.microsoft.com/download) (`winget install Microsoft.DotNet.SDK.10`)

## Build, run, test

```powershell
dotnet build src\Authenticator.App -p:Platform=x64
.\src\Authenticator.App\bin\x64\Debug\net10.0-windows10.0.26100.0\win-x64\Authenticator.exe

dotnet test --project tests\Authenticator.Core.Tests
```

Production build: one self-contained `Authenticator.exe` (~41 MB, no .NET or Windows App SDK install needed):

```powershell
dotnet publish src\Authenticator.App -c Release -r win-x64 -p:Platform=x64 `
  -p:PublishSingleFile=true -p:IncludeNativeLibrariesForSelfExtract=true `
  -p:IncludeAllContentForSelfExtract=true -p:EnableCompressionInSingleFile=true -o dist
```

MSIX package: add `-p:WindowsPackageType=MSIX` and sign it (`AppxPackageSigningEnabled`, or `winapp sign`).

### Development data folder

The vault and settings live in `%LOCALAPPDATA%\Authenticator`. To test without touching your real accounts, point the app at another folder:

```powershell
$env:AUTHENTICATOR_DATA_DIR = "$env:TEMP\auth-dev"; .\...\Authenticator.exe
```

Screen-capture protection is on by default, so screenshots of the window come out black. Turn it off in Settings or set `"screenCaptureProtection": false` in that folder's `settings.json`.

## Project layout

| Project | What's in it |
|---|---|
| `src/Authenticator.Core` | Pure .NET: TOTP/HOTP, Base32, `otpauth://` and Google `otpauth-migration://` parsing/building, AES-GCM + Argon2id, vault and backup file formats |
| `src/Authenticator.Infra` | Windows platform: Windows Hello + DPAPI key wrapping (`VaultSession`), atomic file writes, settings, screen capture, QR decode/encode (ZXing.Net), SNTP clock check, start-with-Windows |
| `src/Authenticator.App` | WinUI 3 app: pages, dialogs, snip overlay, tray icon, global hotkey, auto-lock |
| `tests/Authenticator.Core.Tests` | RFC 4226/6238 vectors, parsers, crypto round-trips, QR decoding (tight crops, 6400×1440 desktops) |

## Releases and auto-update

- Every push to `main` runs [`.github/workflows/release.yml`](.github/workflows/release.yml): tests, a single-file build stamped `1.1.<run number>`, and a GitHub release `v1.1.<run number>` with `Authenticator.exe` and `Authenticator.exe.sha256`.
- The released app checks the [latest release](https://github.com/ranjbar-dev/authenticator/releases/latest) 20 s after start and every 6 hours. It downloads the new exe next to itself as `Authenticator.exe.update`, verifies its SHA-256 against the published checksum, and offers **Restart now**. If you just quit, the update installs on exit.
- The swap renames the running exe to `Authenticator.exe.old` (deleted on next start) and moves the new one into place, so shortcuts keep working.
- Self-update only runs in the published single-file exe, not in `dotnet build` output. Settings → Updates shows the version and status, and has a "Check now" button.

## Design

- **Type:** Geist for the interface, Geist Mono for codes (bundled, SIL OFL 1.1; see `Assets/Fonts`).
- **Color:** one purple accent on purple-tinted neutrals. Codes are ink; the accent marks what you can act on.
- **Main screen:** one shared time strip for all 30-second codes instead of a ring per row. Rows are borderless tiles with hover, press and inline "Copied" feedback.
- Design tokens live in `src/Authenticator.App/Styles/Theme.xaml` (light, dark and high-contrast).

## How secrets are protected

- Accounts are stored in `vault.dat`, encrypted with AES-256-GCM using a random vault key.
- **Windows Hello mode:** the vault key is wrapped by a key derived from a Hello signature (the TPM-backed private key signs a stored challenge), and that is wrapped again with DPAPI for the current user. Unlocking requires a Hello prompt.
- **Password mode:** the vault key is wrapped by an Argon2id key (64 MiB, 3 iterations).
- Screenshots used for import exist only in memory and are zeroed after decoding. Copied codes are excluded from clipboard history and cloud clipboard, and are cleared after 30 s.
- Network access is limited to update checks against this repository's GitHub releases (can be turned off in Settings → Updates) and the opt-in clock check against `time.windows.com`. Account data never leaves the PC.

If Windows Hello is reset, its key is gone and the vault can't be opened. The unlock screen then offers a reset, after which you can restore from an `.authbackup` file. **Make a backup.**
