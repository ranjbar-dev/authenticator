# PRD: Authenticator for Windows

| | |
|---|---|
| **Name** | Authenticator |
| **Platform** | Windows 10 (1809+) / Windows 11, x64 + ARM64 |
| **Stack** | C# / .NET 10 (LTS), WinUI 3 (Windows App SDK) |
| **Status** | Draft v1 · 2026-11-16 |

---

## 1. Summary

A native Windows desktop authenticator app that generates TOTP and HOTP codes, much like Google Authenticator. Users can bring their existing accounts over by **capturing a QR code from the screen or an image**. This includes Google Authenticator's multi-account "Transfer accounts" QR. They can also **enter secrets manually**. The UI is modern and minimal, follows the system light/dark theme, and uses **purple** as the brand color. Secrets are encrypted at rest and the app is locked behind **Windows Hello**.

## 2. Goals & Non-Goals

### Goals
1. Generate correct TOTP (RFC 6238) and HOTP (RFC 4226) codes for all common providers.
2. Import accounts quickly from screenshots: snip a region of the screen, paste from the clipboard, or drop/open an image file.
3. Import a whole Google Authenticator export (`otpauth-migration://`) in one step.
4. Add accounts manually when no QR is available.
5. Keep secrets safe: encrypted vault, Windows Hello unlock, auto-lock, no network access.
6. Provide a modern, minimal UI with light and dark mode and a purple accent.
7. Support encrypted local backup and restore.

### Non-Goals (v1)
- Cloud sync or multi-device sync.
- macOS, Linux, or mobile versions.
- Push-based 2FA, Steam Guard, or other proprietary schemes.
- Browser extension or autofill.

## 3. Target User

One person on a personal Windows PC who already uses Google Authenticator on a phone. They want their codes on the desktop as well, without retyping dozens of secrets. They care about security but don't want friction.

## 4. User Stories

| # | As a user, I want to… | So that… |
|---|---|---|
| U1 | export from Google Authenticator on my phone, show the QR, and snip it on my PC | all my accounts move over in one go |
| U2 | snip a website's 2FA setup QR shown in my browser | I can enroll a new account without a phone |
| U3 | paste a screenshot (Ctrl+V) or drag an image onto the window | I can import from images I already have |
| U4 | type in an issuer, account name, and secret key | I can add accounts that only show a text key |
| U5 | see every code with a countdown and copy it with one click | logging in is fast |
| U6 | search my accounts by typing | I find the right one among many |
| U7 | unlock with my face, fingerprint, or PIN | my codes stay protected if someone uses my PC |
| U8 | export an encrypted backup file and restore it later | I don't lose access if my PC dies |
| U9 | switch between light, dark, or system theme | the app matches my setup |
| U10 | rename, reorder, pin, and delete accounts | my list stays tidy and my most-used codes stay on top |
| U11 | use counter-based (HOTP) accounts too | older services and hardware-token-style accounts work as well |
| U12 | close the window and still use the scan hotkey | the app is always ready without cluttering my taskbar |

## 5. Functional Requirements

### 5.1 Code generation
- **FR-1** Implement TOTP per RFC 6238 / RFC 4226 truncation.
- **FR-2** Support algorithms SHA1 (default), SHA256, and SHA512.
- **FR-3** Support 6 (default), 7, and 8 digit codes.
- **FR-4** Support periods of 30 s (default) and 60 s, or any positive integer.
- **FR-5** Base32 secrets: case-insensitive. Ignore spaces and hyphens. Accept missing padding.
- **FR-6** Format codes for readability: `123 456` / `1234 5678`.
- **FR-7** Show a per-account countdown ring. Change the ring color in the last 5 s.
- **FR-8** Optionally show the *next* code during the last 5 s (setting, off by default).
- **FR-9** Use the system clock. Warn if the clock looks wrong (optional NTP check, off by default, see §8).
- **FR-9a HOTP:** implement HOTP per RFC 4226 with a stored counter per account (same algorithm/digit options as TOTP).
  - HOTP rows hide the code (`••• •••`) and show a **"Get code"** (↻) button instead of a countdown ring.
  - Pressing it increments the counter, **persists the new counter to the vault before showing the code**, then displays and copies the code.
  - The code stays visible for 30 s, then is hidden again.
  - "Edit details" lets the user view and adjust the counter to resync with a server (with a warning).

### 5.2 Import from screen / image
- **FR-10 Snip region:** a "Scan from screen" button and a global hotkey (default `Ctrl+Shift+Q`, configurable) open a dimmed full-screen overlay on all monitors. The user drags a rectangle, and the app decodes the QR in that region.
  - Use the Windows.Graphics.Capture API or a GDI/DXGI screen grab. Handle per-monitor DPI correctly.
  - Hide the app window during capture so it doesn't cover the QR.
- **FR-11 Clipboard:** `Ctrl+V` in the main window, or a "Paste image" button, decodes the image on the clipboard.
- **FR-12 File:** drag & drop or an "Open image…" picker accepts PNG, JPG, BMP, GIF, and WEBP.
- **FR-13 Decoder:** ZXing.Net (QR). Try the original image first, then grayscale, inverted, and upscaled versions to handle small or low-contrast QRs. If an image contains multiple QR codes, decode all of them.
- **FR-14 Supported payloads:**
  - `otpauth://totp/...`: standard Key URI format (issuer, label, secret, algorithm, digits, period).
  - `otpauth://hotp/...`: same format, with the required `counter` parameter.
  - `otpauth-migration://offline?data=...`: Google Authenticator export. Base64 → protobuf `MigrationPayload` → list of `OtpParameters` (both TOTP and HOTP entries, including HOTP counters). Handle multi-batch exports (batch_index/batch_size). Tell the user "batch 1 of 3 imported, scan the next".
- **FR-15 Review screen:** before saving, show a list of the parsed accounts with checkboxes, editable issuer/name, and a flag on duplicates (same secret, or same issuer + name). The user confirms with "Import N accounts".
- **FR-16 Errors:** show clear messages for "No QR found", "QR is not an authenticator code", and "Unsupported or malformed parameters".
- **FR-17 Hygiene:** screenshots and decoded images exist **only in memory**. Never write them to disk or log them. Clear buffers after use.

### 5.3 Manual entry
- **FR-18** Form fields: Issuer (optional), Account name, Secret key (required), Type (Time-based / Counter-based, default Time-based), plus an "Advanced" expander for Algorithm, Digits, and Period (TOTP) or initial Counter (HOTP, default 0).
- **FR-19** Validate Base32 as the user types and show inline errors. Show a live preview of the current code so the user can compare it with the website.
- **FR-20** Accept a pasted `otpauth://` URI in the secret field and auto-fill all fields from it.

### 5.4 Account list
- **FR-21** Show cards/rows with: letter avatar (first letter of the issuer, or of the account name if there's no issuer, on the `AccentSubtle` tint; no logos, no network fetch), issuer, account name, code, and countdown (or "Get code" button for HOTP).
- **FR-21a Favorites:** each account has a **Pin** toggle (☆/★ on hover and in the context menu). Pinned accounts appear in a "Pinned" section at the top of the list; the rest follow under "All accounts". Drag-to-reorder works within each section. No folders or groups.
- **FR-22** Click a code (or press Enter when it's focused) to copy it. Show a toast saying "Copied". Clear the clipboard after 30 s if it still holds the code (configurable).
- **FR-23** Search box (`Ctrl+F`, or just start typing) filters by issuer or name.
- **FR-24** Drag to reorder. Context menu: Pin/Unpin, Rename, Edit details, Show QR (requires re-auth), Delete (with confirm).
- **FR-25** Empty state: friendly illustration + "Scan from screen" / "Enter manually" buttons.

### 5.5 Security & lock
- **FR-26** On launch, and after auto-lock, require **Windows Hello** (face / fingerprint / Windows PIN) via `UserConsentVerifier` / `KeyCredentialManager`.
- **FR-27** Fallback: if Windows Hello isn't available or set up, the user creates an **app password** on first run instead.
- **FR-28** Auto-lock after N minutes of inactivity (default 5; options: 1, 5, 15, 30, never) and on Windows session lock or sleep.
- **FR-29** Exclude the window from screen capture by default (`SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`), with a setting to turn this off.
- **FR-30** Require re-authentication to reveal a secret, show a QR, or export.

### 5.6 Backup & restore
- **FR-31** **Export:** create an encrypted `.authbackup` file protected by a user-chosen password (minimum 8 characters, strength meter). Format: JSON payload → AES-256-GCM, key from Argon2id (salt + params stored in a header).
- **FR-32** **Import:** pick a file, enter the password, then review and merge (same review screen as FR-15).
- **FR-33 Transfer to phone (QR only):** "Export as QR" shows Google Authenticator-compatible `otpauth-migration://` QR codes **on screen only**, so the user can scan them with a phone. Behind re-auth and a strong warning ("anyone who sees these codes can copy your accounts").
  - The user picks which accounts to include.
  - Split into multiple QRs (batches) when needed, with Next/Previous.
  - Screen-capture protection stays on while these QRs are shown, even if the user disabled it in settings.
  - No plaintext file export (no `.txt`/`.csv` of URIs).
- **FR-34** Show a reminder banner if there are more than 0 accounts and no backup has been made, or the last backup is older than 90 days (dismissible).

### 5.7 Settings
- Theme: System / Light / Dark
- Auto-lock timeout
- Clipboard clear timeout
- Show next code
- Scan hotkey
- Screen-capture protection
- Start with Windows (**off** by default; when on, starts minimized to the tray)
- Close button behavior: minimize to tray (**default**) / exit
- Backup / Restore
- About (version, licenses)

## 6. UI / UX

### 6.1 Principles
- **Minimal:** one main screen. Codes are the hero, and the chrome stays quiet.
- **Fast:** from launch to a copied code should take under 3 seconds, including Hello.
- **Native:** Fluent Design, Mica backdrop, rounded corners, and Segoe UI Variable all come from WinUI 3.
- **Keyboard-first:** every action is reachable without a mouse.

### 6.2 Layout
```
┌───────────────────────────────────────────────┐
│  Authenticator                    ─  □  ✕     │  ← custom title bar, Mica
│  [🔍 Search accounts…        ]   [+ ▾]  [⚙]   │  ← + menu: Scan screen / Paste / Open image / Manual
├───────────────────────────────────────────────┤
│  (G)  Google                                  │
│       me@gmail.com              482 913   ◔   │  ← big monospace code, countdown ring
│  ───────────────────────────────────────────  │
│  (Gh) GitHub                                  │
│       larry                     107 554   ◑   │
│  ───────────────────────────────────────────  │
│  …                                            │
└───────────────────────────────────────────────┘
```
- Default window size is 420×640 and can be resized (minimum 360×480). In wide windows the list becomes a 2-column grid.
- Codes use **Cascadia Mono / Segoe UI Variable Display** at 24–28 px with tabular figures.
- Use motion sparingly: fade in new codes, ease out the countdown ring, use a subtle press scale on copy.

### 6.3 Color & theme

The purple brand color drives the accent. Neutrals come from Fluent's system defaults.

| Token | Light | Dark |
|---|---|---|
| `Accent` (primary) | `#7C3AED` | `#A78BFA` |
| `AccentHover` | `#6D28D9` | `#C4B5FD` |
| `AccentPressed` | `#5B21B6` | `#8B5CF6` |
| `AccentSubtle` (avatar bg, selection) | `#EDE9FE` | `#2E1065` |
| `OnAccent` (text on accent) | `#FFFFFF` | `#1E1033` |
| `Background` | Mica / `#F7F5FB` fallback | Mica / `#141118` fallback |
| `Surface` (cards) | `#FFFFFF` | `#1E1A24` |
| `TextPrimary` | `#1C1626` | `#F3F0F7` |
| `TextSecondary` | `#6B6478` | `#A59EB2` |
| `Warning` (countdown < 5 s) | `#D97706` | `#FBBF24` |
| `Danger` | `#DC2626` | `#F87171` |

- Override `SystemAccentColor*` resources with the brand purple, so standard WinUI controls (buttons, toggles, focus rings) pick up the purple automatically.
- All text/background pairs must meet WCAG AA contrast (4.5:1 for body text, 3:1 for large codes).
- Follow the system theme live (`UISettings.ColorValuesChanged`) unless the user picks a theme manually.
- Support high contrast mode: drop the brand colors and use system high-contrast colors.

### 6.4 Key screens
1. **Unlock:** centered app icon, "Unlock with Windows Hello" button (prompts automatically on show).
2. **Main list:** described above.
3. **Add flyout/menu:** Scan from screen · Paste image · Open image… · Enter manually.
4. **Snip overlay:** dimmed screen, crosshair, "Drag around the QR code · Esc to cancel".
5. **Import review:** checklist of the parsed accounts, with editable fields and duplicate badges.
6. **Manual entry dialog:** described in §5.3.
7. **Settings page:** grouped settings cards (WinUI `SettingsCard` style from Community Toolkit).

### 6.5 System tray
- Closing the window (✕) hides it to the notification area tray by default; the app keeps running so the global scan hotkey works.
- The first time this happens, show a one-time toast: "Authenticator is still running in the tray."
- Tray icon: left-click opens the window (unlock prompt if locked). Right-click menu: Open · Scan from screen · Lock now · Quit.
- The app locks when hidden to the tray for longer than the auto-lock timeout.

### 6.6 Keyboard shortcuts
| Key | Action |
|---|---|
| `Ctrl+Shift+Q` (global) | Scan from screen |
| `Ctrl+V` | Import image from clipboard |
| `Ctrl+N` | Add manually |
| `Ctrl+F` / type | Search |
| `↑ ↓` + `Enter` | Navigate list / copy code |
| `Ctrl+L` | Lock now |
| `Ctrl+,` | Settings |

## 7. Technical Architecture

### 7.1 Why this stack
**C# / .NET 10 + WinUI 3** is the best fit for a Windows-only, modern-looking app:
- It provides native Fluent controls, Mica, and light/dark theming without custom styling work.
- It gives first-class access to Windows Hello, DPAPI, Windows.Graphics.Capture, the clipboard, and global hotkeys.
- It has mature libraries: ZXing.Net (QR), Google.Protobuf (migration payload), Konscious/libsodium (Argon2id).
- It can be packaged as MSIX for clean install, auto-update, and uninstall.

Alternative we considered: Tauri 2 (Rust + web UI). It is smaller and cross-platform, but feels less native on Windows and needs more effort for Hello and capture. Revisit it if cross-platform becomes a goal.

### 7.2 Project structure
```
Authenticator.sln
├─ src/Authenticator.App/        WinUI 3 app (Views, ViewModels – MVVM via CommunityToolkit.Mvvm)
├─ src/Authenticator.Core/       Pure .NET: TOTP, Base32, otpauth URI parser, migration parser, models
├─ src/Authenticator.Infra/      Vault storage, crypto, Windows Hello, screen capture, QR decode, clipboard
└─ tests/Authenticator.Core.Tests/  xUnit – RFC test vectors, parser fuzz cases
```

### 7.3 Key libraries
| Concern | Library |
|---|---|
| UI | Microsoft.WindowsAppSDK, CommunityToolkit.WinUI (SettingsControls) |
| MVVM | CommunityToolkit.Mvvm |
| QR decode | ZXing.Net (+ bindings for SkiaSharp or SoftwareBitmap) |
| Google migration | Google.Protobuf (with `.proto` for `MigrationPayload`) |
| KDF | Konscious.Security.Cryptography.Argon2 (or libsodium) |
| AEAD | `System.Security.Cryptography.AesGcm` |
| TOTP | Own implementation in Core (small, auditable), using `HMACSHA1/256/512` |

### 7.4 Data model
```csharp
record Account(
    Guid Id,
    string Issuer,
    string Name,
    byte[] Secret,          // raw bytes, never Base32 at rest
    OtpType Type,           // Totp | Hotp
    OtpAlgorithm Algorithm, // SHA1 | SHA256 | SHA512
    int Digits,             // 6–8
    int Period,             // seconds (TOTP only)
    long Counter,           // HOTP only
    bool Pinned,
    int SortOrder,
    DateTimeOffset CreatedAt);
```

### 7.5 Storage & encryption
- Store the vault file at `%LOCALAPPDATA%\Authenticator\vault.dat`.
- The vault is serialized JSON, encrypted with **AES-256-GCM** using a random 256-bit **vault key**.
- Protecting the vault key:
  - **Windows Hello mode:** the vault key is wrapped with DPAPI (CurrentUser) *and* gated by a Hello `KeyCredential` signature. The app derives a wrapping key from the signature over a fixed challenge, so the vault can't be decrypted without a Hello prompt.
  - **Password fallback:** the vault key is wrapped with an Argon2id-derived key (m=64 MiB, t=3, p=1, tunable).
- Write files atomically (write temp → flush → replace). Keep one previous copy as `vault.bak`.
- Hold secrets in memory only while unlocked, and zero the byte arrays on lock where practical.

### 7.6 Privacy
- No telemetry and no icon fetching. The only network calls are update checks against the project's public GitHub releases (Settings → Updates, on by default, can be turned off) and the opt-in clock check.
- It doesn't log secrets, codes, URIs, or images, even at debug level.

## 8. Non-Functional Requirements

| Area | Requirement |
|---|---|
| Performance | Cold start to unlock prompt < 1 s. Code refresh does not cause visible jank with 200 accounts. QR decode < 500 ms for a 4K capture. |
| Accuracy | Pass all RFC 6238 Appendix B test vectors (SHA1/256/512) and RFC 4226 Appendix D HOTP vectors. |
| Reliability | No vault corruption on crash or power loss (atomic writes + backup copy). |
| Accessibility | Full keyboard navigation. Narrator reads issuer, name, and code (digit by digit). AA contrast. High contrast mode supported. Respects the "reduce animations" setting. |
| Size | MSIX < 40 MB (self-contained, trimmed). |
| Clock | Optional, opt-in "check time drift" that queries an NTP server and warns if drift > 15 s. This is the only network use, and it's off by default. |

## 9. Distribution
- MSIX package, signed. Sideload in v1; Microsoft Store is optional later.
- Optional portable unpackaged build. Without package identity, some features fall back: Hello needs the password fallback, and there is no auto-start.
- Version scheme: SemVer.

## 10. Milestones

| Milestone | Scope |
|---|---|
| **M1 – Core** | TOTP + HOTP engine, Base32, `otpauth://` parser, migration parser (read + write), unit tests |
| **M2 – Vault & lock** | Encrypted storage, Windows Hello + password fallback, auto-lock |
| **M3 – Main UI** | Account list, copy, search, pin, reorder, HOTP "Get code", light/dark/purple theming, manual entry |
| **M4 – Import** | Image file + clipboard decode, snip overlay + hotkey, review screen, Google batch flow |
| **M5 – Backup & polish** | Encrypted export/import, QR transfer export, settings, tray, accessibility pass, MSIX packaging |

## 11. Success Metrics
- Importing a 20-account Google Authenticator export takes under 60 s end to end.
- Zero code mismatches against Google Authenticator for the same accounts.
- Unlock → copy code in ≤ 3 s.
- No secret material found on disk outside `vault.dat` (verified with a disk/temp-folder scan in testing).

## 12. Decisions Log

| Topic | Decision |
|---|---|
| Platform | Windows only, C# / .NET 10 + WinUI 3 |
| App lock | Windows Hello, with app-password fallback |
| Import sources | Google Authenticator export QR, single `otpauth://` QRs, screen snip, clipboard/file image |
| Backup | Encrypted local `.authbackup` file; no cloud sync |
| App name | **Authenticator** |
| HOTP | **Supported in v1** (FR-9a) |
| Account icons | **Letter avatars** only |
| Close button | **Minimize to tray** by default (§6.5) |
| Plaintext export | **On-screen QR only**, with warning; no plaintext file (FR-33) |
| Start with Windows | **Off** by default |
| Organization | **Pin favorites** to top; no folders (FR-21a) |
| Distribution | Single-file `Authenticator.exe` built and released by GitHub Actions on every push to `main`; the app self-updates from GitHub releases (SHA-256 verified) |
| App icon | **Purple gradient tile + white shield + 3/4 countdown ring + keyhole**, source in `assets/icon/` (§14) |

## 13. Open Questions
_None at this time._

## 14. App Icon

| | |
|---|---|
| **Concept** | A white **shield** for security and a **keyhole** for the lock, surrounded by a **3/4 ring** that matches the per-code countdown ring in the UI. All of it sits on a rounded tile with a purple gradient (`#A78BFA` → `#7C3AED` → `#5B21B6`). |
| **Small-size variant** | At 16–24 px the ring is dropped and the shield and keyhole fill the tile, so the icon stays legible in the title bar and tray. |
| **Files** | `assets/icon/app-icon.svg` (master, ≥ 32 px) · `assets/icon/app-icon-small.svg` (≤ 24 px) · `assets/icon/png/app-icon-{16…1024}.png` · `assets/icon/app.ico` (16–256 px, multi-size) |
| **Usage** | `app.ico` for the window, tray, and unpackaged EXE. PNGs for MSIX visual assets (`Square44x44Logo`, `Square150x150Logo`, `StoreLogo`, etc.). |
