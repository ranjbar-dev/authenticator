# Task 21 remaining native workflows

Author execution, 2026-11-13. This supplements the prior screenshot-import/HMAC and one-account export evidence; it does not mark Task 21 complete. Independent review remains required.

## Scope and ownership

Used the computer-use skill with `@oai/sky`: real pointer clicks, actual Windows Save/Open dialogs, keyboard Tab/Shift+Tab/Space/Ctrl+L, and documented accessible field entry. No synthetic DOM activation. The current-source `workflow_qa` example calls the production UI, repository and encrypted storage paths. Both vaults began clean through the native Create screen, with different disposable local passphrases and a third separate backup passphrase.

| Owned resource | Identity |
| --- | --- |
| Source | window 20973702, PID 26396, `target/task21-owned-AopmHo` |
| Fresh destination | window 62327118, PID 5336, `target/task21-owned-qm4ORE` |
| Encrypted full backup | `target/task21-full-626a58c7-c659-4d13-b6fd-a6a590f0d6d7.authpbk` |
| Generated UTF-8 input | `target/task21-input-7cfcd397-e477-4d26-b578-6d6c0dfda303.txt` |

Ownership manifests matched the executable and roots before filesystem checks. Source and copy-verification preview closed normally after evidence preservation. Destination was left unlocked at Settings for independent review; its five-minute auto-lock remains enabled. The encrypted backup and original generated input remain under ignored `target`; neither is a Git artifact. No default/user/reference-app vault, Windows Run registration, network configuration, display settings or system security setting was changed.

Build: `rtk cargo build --offline --locked --example workflow_qa --example qr_fixture` passed. The initial link attempt encountered the old owned workflow example holding the executable. Its exact process/path/start and locked window were identified before closing that fixture normally; the subsequent build succeeded.

## Native results

1. Created a manual account using the published RFC test seed through the shared account form. Added recovery codes through the searchable existing-account selector and the actual UTF-8 file Open dialog. The generated BOM/CRLF input included surrounding whitespace, a blank line and an exact duplicate. Preview showed **2 unique valid, 1 blank discarded, 1 exact duplicate, 0 retained/2 new**. Explicit duplicate-removal consent and Save produced 2 available/0 used. The input values are omitted from this report.
2. Recovery detail: pointer Reveal/Conceal and Copy succeeded. Copy displayed Copied without changing used state. Mark used produced 1 available/1 used; Undo restored 2/0; marking used again restored 1/1. Shift+Tab reached Copy, and physical Space activated it without marking used. Ctrl+L while revealed hid account/recovery content; unlocking restored the saved state. Actual detail frames were inspected at 340, 600 and 960 widths.
3. Created QA Personal (Purple) and QA Shared (Green), each containing the same account. Used both pointer selection and keyboard Tab/Space through search, sort, clear, membership and Save. Unfiled showed no matches; QA Personal showed the account. Both folders initially showed one active account.
4. Account Trash cancellation left source ciphertext unchanged. Confirmed Move to Trash hid the account and its recovery set from active Settings counts. Restore preview retained **1 account, 2 memberships, 1 set, 2 codes**. Explicit Restore returned active counts to **1 account, 1 set, 1 available/1 used** and preserved folder links.
5. Moved QA Shared to Trash while retaining the account, its links and used history. Moved the unused individual recovery code to Trash. The export dataset then contained **1 account, 2 folders, 2 memberships, 1 recovery set, 2 codes, 1 used code, 2 records with their own Trash state**. The two trashed records were the Shared folder and the unused code.
6. Native encrypted export: chose an exact local destination in Save. Mismatched backup-phrase confirmation showed a readable error, created no output and left vault ciphertext unchanged. Correct confirmation produced export success. Checks returned `nativeBackupExists=true`, `nativeMagic=true`, `containsKnownPlaintext=false`, `sourceCipherUnchanged=true`. The plaintext check tested known generated account/recovery/seed sentinels in encrypted bytes; it is not a general cryptographic proof.
7. After export, exercised actual irreversible cleanup only on source disposable records. Single-code permanent-delete Cancel left ciphertext unchanged; explicit reviewed confirmation removed exactly that code. Empty Trash Cancel likewise preserved ciphertext; explicit confirmation removed the Shared folder and its one membership. Trash became empty while the active account/used recovery set remained. The already-exported encrypted backup stayed byte-for-byte unchanged.
8. Fresh destination native import: selected that backup through Open. Wrong backup phrase produced a safe error, cleared the phrase field, and left destination ciphertext unchanged. Correct phrase authenticated the full graph counts above; preview did not mutate destination ciphertext. Explicitly selected **Use backup display settings** (QA Personal filter), retained current security timeouts, reviewed exact resulting records/links/used and Trash timestamps, then applied Merge. Native success showed the account under QA Personal. Detail showed **0 available/1 used**, and Trash displayed the original unused code and Shared folder with their original deletion dates.
9. Independently computed SHA-1 HMAC/dynamic truncation in Node using the published RFC fixture and current 30-second counter. The restored native code matched: `independentHmacMatch=true`, `counterStable=true`, sampled **2026-11-13T14:08:52.284Z**. No code or seed is stored in this report.
10. Deliberately marked the destination's active code available. Reimported the same native backup through Open and authenticated it again. Default **Keep current / skip incoming bundle** review showed 1 bundle skipped, no remapped IDs, still **1 account/2 folders/2 memberships/1 set/2 codes/0 used/2 trashed**. Applied Merge; Settings confirmed **1 available/0 used** and Trash 2. Existing records were not duplicated.
11. Reimported again and explicitly chose Replace. Exact review showed current 0-used collections removed and the incoming 1-used graph restored. At the final destructive dialog, keyboard Space activated the default Keep current action; `replaceCancelCipherUnchanged=true`. Reopened the dialog, scrolled to inspect the explicit Replace action and clicked it. Native success and Settings confirmed **1 active account, 1 active set, 0 available/1 used, Trash 2**.

The display preference was explicitly imported on the first restore. Later imports retained current display settings. Lock/clipboard timeouts stayed local (300/30 seconds), startup stayed disabled, and the UI stated local passphrase/key/identity retention. This is native record-graph and preference-choice evidence, not a claim of byte-identical vault files or independently decrypted database equality. Existing engine round-trip tests complement this UI evidence.

## Sanitized visual evidence and repair

All retained PNGs in [remaining-native](remaining-native/) are app-generated F10 frames, with masks applied before the only disk write. Reviewed every retained frame, including the intentionally revealed recovery state and restored OTP mask. No QR payload, seed, recovery value, OTP or passphrase pixels are retained. The first three filenames are chronological per owned source, rather than a new synthetic mockup sequence.

| Frames | Observed state |
| --- | --- |
| `source-002`, `source-003`, `source-005` | Duplicate preview at 600; revealed detail masked at 960; keyboard Copy focus at 340 |
| `source-007`, `source-008` | Empty folder page and selected account membership at 600 |
| `source-010`, `source-013` | Account restore scope and single permanent-delete scope at 600 |
| `restored-001`, `restored-003`, `restored-004` | Wrong-phrase error at 600; authenticated graph at 340/960 |
| `restored-005`, `restored-006`, `restored-007` | Successful restored Codes with OTP mask, used detail, original Trash records at 960 |
| `restored-008`, `restored-009`, `restored-010`, `restored-011` | Keep-current Merge review, Replace review/confirmation at 960, final Settings at 600 |
| `guidance-fixed-600` | Fresh compiled native recovery help showing readable navigation words |

Observed arrow separators in folder/help guidance rendered as missing glyphs (`source-007`). Replaced the 11 existing navigation strings in six UI files with words, keeping their shared text components and meaning. Scoped checklist: existing guidance only, readable navigation, no new roles/layout/styles/business behavior. Inspected the six affected string regions (two grouped searches); skipped unrelated typography/spacing counts. Used UI consistency finding-patterns/adjusting/implementing guidance; parent supplies the separate review. No planted-copy calibration was performed by this author, and no claim of that proof is made.

`rtk cargo build --offline --locked --example recovery_detail` passed. Launched that newly compiled isolated database fixture, opened Help with a real pointer, scrolled, and visually verified “Trash in Settings” without a missing glyph. Saved its sanitized frame, then closed it normally. `rtk cargo fmt --all -- --check` and `rtk git diff --check` passed. No tests added for the copy-only repair; no redundant full-suite rerun.

## Still open

This run does not establish reference-pixel parity (reference app unavailable), hardware/mixed-monitor DPI, disconnected-machine execution, Narrator/high-contrast behavior, actual Windows lock/sleep/resume delivery, or real Windows logon launch/Run registration. The example's Settings reported that startup was unavailable from this long example path; no production registration was attempted. Full pagination/large-vault performance, every help topic and every import conflict choice were not re-exercised here. Keep-both/remapped IDs were not selected in this native run. No task completion checkbox or PRD scope was changed.
