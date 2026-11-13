# Authenticator user guide

Authenticator stores TOTP credentials and service-issued recovery codes locally.
It has no app account, sign-in, cloud sync, telemetry or required runtime network
service. A service username is only a label in your encrypted vault.

## Install, run and update

Extract the Windows x64 ZIP into a local folder you control and run
`Authenticator.exe`. Keep the documentation and license notices with the app.
Rust, Python and Node are build tools, not end-user runtime requirements. This
unsigned preview targets Windows 10/11 x64; see the build manifest and validation
report for the environments actually tested. Windows may display an unsigned-app
warning. Verify the ZIP's SHA256 against its adjacent `.zip.sha256` file.

To update, export an encrypted backup first, disable Auto Start, lock and exit,
then extract the new version into a separate folder. Run the new copy and unlock
the existing vault. Never run two copies against the same vault simultaneously.
Enable Auto Start again only from the local copy you intend to keep.

The ZIP is portable application distribution; it does not make the local vault
follow the executable. Windows resolves the per-user data folder, normally
`%LOCALAPPDATA%\Authenticator`. All vault records and preferences live inside
its encrypted `vault.bin`. Do not move an open vault or use it as a portable
backup. Use the encrypted export workflow below.

When another packaged application launches the executable with inherited Windows
package identity, Windows may redirect that Known Folder. Use the same normal
launch method after updates; export/import is the supported way to transfer
between different profile or package contexts. A missing vault in a different
context does not mean the original was deleted.

To uninstall, disable Auto Start, lock, exit and delete the extracted app folder.
This leaves the user vault and independently saved backups intact. Vault removal
is a separate deliberate action; export and verify recovery first. Deleting files
does not promise secure erasure from SSDs, OS copies or old backups.

## Create and unlock a local vault

On first launch choose a long, unique vault passphrase and confirm it. Spaces are
kept exactly; the accepted length is 12–1024 UTF-8 bytes. This creates a local vault,
not an online account. The app cannot reset a forgotten passphrase. Keep a
separately encrypted backup and its separate passphrase somewhere recoverable.

Relaunch starts locked. Enter the same vault passphrase to unlock. Incorrect
phrases and damaged files fail without replacing the vault. Do not delete a file
because an unlock attempt failed. Default idle lock is five minutes; Settings
offers other durations. The header Lock button and Ctrl+L lock immediately.
Windows lock/suspend notification handling is implemented; actual hardware
delivery and additional acceptance limits are listed in Security Limits.

## Add a credential

**Screenshot to Scan** is the primary acquisition action on 2FA Codes. Display
the service's TOTP QR in another window, choose Screenshot to Scan and drag a
region around the complete QR. Escape or Cancel abandons selection. Selected
pixels are decoded locally in memory; the app does not save a screenshot file.
Protected/empty/invalid regions may fail; retry, use an image or enter manually.

For multiple supported QR codes, select the intended credentials. Review service,
website/account label and TOTP parameters before explicit Save. The secret stays
concealed unless you intentionally reveal it. A duplicate warning does not
silently overwrite another entry; a separately approved account has its own ID.

**Import from Photos** reads a local PNG/JPEG. Limits are 8 MiB encoded input,
4096 pixels per axis, 4,194,304 total pixels, a 32 MiB decode budget, 2048 bytes per QR
payload and at most 7 candidates. Unsupported content is rejected. No source
image/QR payload becomes a stored vault record.

Use **Add account** for manual service name, optional website/identifier and a
Base32 secret. Defaults are SHA-1, 6 digits, 30 seconds. Advanced options also support
SHA-256/SHA-512, 8 digits and 15/60/120-second periods. Preserve the service's actual
settings. HOTP, Steam codes, passkeys and proprietary migration QR formats are
not supported. Account editing preserves the seed unless you deliberately choose
to change it. Validate a fresh code with the service before relying on an entry.

## Find and copy codes

Search service/account labels, choose one of six sort orders and select list or
grid view. Sort, view, Codes query and folder filter are encrypted preferences.
Code cards show remaining lifetime; use a fresh code near expiry. Copy has its
own button and confirmation. Default clipboard expiry is 30 seconds, configurable
in Settings. Cleanup checks ownership and contents and preserves a subsequent
copy from another app. Clipboard managers and other processes may retain copies.

Tab/Shift+Tab move focus; Space or Enter activates supported controls. Ctrl+L
locks. Escape closes a dialog or cancels an active flow as its UI explains.
Cancel does not undo an already admitted/completed atomic save; unlock and check
stored records after interrupting busy work.

## Recovery codes

In Backup Codes choose Add Backup Codes, select an existing account and enter
one service-issued code per line or use **Read Backup Codes from File**. Input
is UTF-8 with optional BOM, CRLF/LF, up to 1 MiB. Surrounding whitespace is trimmed,
blank lines omitted and exact duplicates require explicit reviewed deduplication.
Invalid encoding/oversized input cannot replace an existing set.

Values are concealed by default. Reveal and Copy are separate actions. Copy
never marks a code used: choose Mark used only after the service accepts it.
The view offers one-step Undo; navigation or locking clears that undo action.
Editing retains exact matching codes' identity/used history; omitted active codes
move to Trash. New values start unused. Deleting a set preserves its account.
These codes come from the service; Authenticator cannot issue replacements.

## Folders and Trash

Use Manage folders to create/rename a folder, choose one of six colors and select
accounts through search/sort. An account may belong to several folders. All and
Unfiled are distinct views. Removing a folder preserves accounts and recovery
codes; its recoverable links remain until permanent deletion.

Account deletion moves it to Trash. Open Trash in Settings to review dates and
restore the account, associated recovery data and surviving relationships.
Records separately deleted remain separately in Trash; restore their required
account/set first. Trash has **no automatic expiry**. Single permanent deletion
and Empty Trash require explicit reviewed confirmation and may include dependent
records shown in the scope. The last permanently removed code can remove its
empty owning set. Existing exported backups are not revoked or rewritten.

## Encrypted backup and restore

Open Settings, Export & Import, **Export encrypted backup**. Choose a local file,
enter and confirm a separate backup passphrase, then explicitly export. Encryption
is mandatory; no plaintext export exists. Existing output requires deliberate
overwrite approval. The `.authpbk` contains accounts/settings, metadata, folders,
memberships, recovery-code used states, Trash and preferences.
The format accepts up to 64 MiB of encoded vault payload plus its 160-byte
authenticated envelope overhead; oversized files fail before full-file loading.

To restore, choose **Import backup**, select the native file and enter its backup
passphrase. Review authenticated counts, conflicts and exact resulting records.
Default Merge keeps current conflicting account bundles. Choose alternatives only
when you understand the reviewed result. Replace removes current collections and
requires a final separate confirmation. Display/security preference adoption is
explicit; startup is never enabled by import. Local vault identity/passphrase/key
stay local. Wrong passphrases, corruption and unsupported formats do not become
an empty successful backup. A stale preview must be reviewed again.

Native portable backups are independent of the original local keys. Keep a
second copy somewhere you control and verify a restore before depending on it.
The reference app's private `.vault` format is unsupported. Renaming a file or
selecting local `vault.bin` does not convert it into a portable backup.

## Clock, startup and help

An incorrect system clock can produce rejected codes. Check Date & time under
Time & language in Windows Settings, including the time zone. Authenticator uses
local system time; it does not contact a time server or change Windows time.

Auto Start is an explicit optional Settings action. Enable/disable it from the
local `Authenticator.exe` copy you intend to keep. The app reports saved desired
state and observed Windows registration/failures separately. A registered entry
does not override Windows Startup Apps or organizational policy. Do not move an
enabled copy without disabling its registration first.

The header Help button gives contextual English guidance without a network
connection. Current validation gaps are documented in Security Limits; this build
does not claim external cryptographic certification or full release acceptance.
