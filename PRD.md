# Product Requirements Document: Authenticator

**Status:** Implementation-ready scope and ordered task plan; implementation not started  
**Basis:** Direct inspection of the running Windows app “Authenticator App · 2FA” on 12 November 2026.  
**Purpose:** Define Authenticator, a Rust desktop app matching the inspected reference UI, with screenshot-based 2FA import as its defining feature and encrypted local-only storage.  
**Repository:** https://github.com/ranjbar-dev/authenticator  
**Branch:** Commit and push project changes to `main`.  
**Interface language:** English only.

## 1. Product overview

**Authenticator** is a Windows desktop app written in Rust. It stores OTP credentials, generates current codes and keeps associated emergency backup codes. Its UI must closely match the inspected **Authenticator App · 2FA** reference: a narrow, dark, mobile-like window with **2FA Codes**, **Backup Codes**, and **Settings** tabs. The highest-priority workflow is **Screenshot to Scan**: capture a selected screen region, scan its QR code locally, review the credential and import it into the encrypted vault. No Authenticator account, Google sign-in or cloud service is offered.

The reference inspection was exploratory and non-destructive. It did not create, edit, delete, export, or import real credentials, activate sync, or change security settings. The requirements below distinguish **Observed** interface evidence from **Proposed** rewrite decisions. A visible option is not proof of its implementation, interoperability, or security properties.

## 2. Goals and success criteria

### Goals

1. Make it fast to locate an account and obtain a valid code on a Windows desktop.
2. Make screenshot capture, local QR scanning and 2FA import the primary workflow, with image import and manual entry as alternatives.
3. Preserve access through deliberate, secure backup, import, and recovery workflows.
4. Protect credential seeds and recovery codes at rest and avoid accidental disclosure.
5. Deliver a maintainable Rust implementation, reference-matching UI and English-only interface without app accounts or cloud integrations.

### Success criteria

- A user can add a test TOTP account, find it, and obtain the same code as an independent standards-compliant implementation at the same time.
- Codes refresh at the configured boundary and remain correct for the supported algorithms, digit lengths, and periods.
- A user can export an encrypted test vault and restore it on a clean installation without loss of supported data.
- Secrets, OTP values, and backup codes never appear in logs, analytics, crash reports, or the PRD.
- First launch, vault creation, screenshot scanning, code generation and export/import work offline without registration or sign-in.
- A user can start Screenshot to Scan from the codes screen, capture a QR in another window, review and save the credential without typing its secret.
- Visual comparison confirms reference-matching navigation, cards, typography, colors, timers and add-account screens, with excluded controls omitted.
- Keyboard-only use covers primary navigation, search, creation, copy, and confirmation dialogs.

## 3. Scope and priorities

| Priority | Capability | Evidence / intent |
|---|---|---|
| P0 — highest | Screenshot → local QR scan → review → 2FA import | Defining product feature; Screenshot to Scan observed |
| P0 | Rust implementation and reference-matching UI | Required product foundation |
| P0 | Local TOTP vault, list, search, add, edit, copy | Core offline workflows |
| P0 | Manual entry, QR image import and validation | Alternative entry workflows |
| P0 | Encrypted local storage, locking, easy encrypted export/import | All persisted user data stays encrypted on the system |
| P0 | Associated backup-code storage | Reference flow observed |
| P1 | Folders, sorting, list/grid view, trash | Reference controls observed |
| P1 | Startup, local clock handling and help | English-only settings |

**Excluded product features:** Google sign-in, any other app login/registration, Authenticator accounts/profiles, cloud storage/backup, cross-device sync, multilingual UI and language selection. Browser extensions and mobile/watch companion integrations are outside this desktop app's scope. Service usernames stored in vault entries are supported; they do not create an Authenticator account.

**Non-goals for initial release:** Creating third-party service accounts, generating recovery codes on behalf of services, enterprise administration, hosted infrastructure, or claiming compatibility with the reference app's private vault format without testing. HOTP, Steam-style codes, passkeys, push approvals, and password management are not established by this inspection and are not assumed in P0.

## 4. Personas and jobs

- **Personal user:** “Find my service, copy its current code, and sign in before it expires.”
- **User onboarding a service:** “Scan the service’s QR code or enter its secret and verify the resulting code.”
- **User preparing for device loss:** “Keep recoverable, protected backups of my vault and emergency codes.”
- **User managing many accounts:** “Search, sort, and group entries without losing track of which account is which.”

## 5. Observed reference inventory

This inventory records the reference app, not additional product scope. Observed profile, sync, language and companion controls are excluded from Authenticator.

### 5.1 2FA Codes

- Main screen shows searchable account cards with a service/name, optional username, icon, six-digit code and a progress bar that changes as time passes. A floating plus opens account creation.
- Toolbar contains a folder-creation control, a grid-shaped view control, and a sort menu. Sort options observed: **Latest First**, **Earliest First**, **Service A > Z**, **Service Z > A**, **Account A > Z**, and **Account Z > A**.
- A help icon and profile/avatar control appear at the top. Their full flows were not inspected.
- Folder creation has a name, six color choices, searchable account selection, sorting and Save.

### 5.2 Adding an account

- The plus opens **Scan QR Code**, with **Screenshot to Scan**, a QR help link, **Manually Input**, and **Import from Photos**. Further options may exist below the visible area; the list was not fully inspected.
- Manual **Account** form contains website name, website address, username, secret key, advanced options and a backup-codes toggle.
- Advanced options show SHA1, SHA256 and SHA512; 6 or 8 digits; and periods of 15, 30, 60 or 120 seconds. The displayed defaults were SHA1, 6 digits and 30 seconds.
- Save is present. Validation, duplicate handling, QR formats, and saving behavior were not tested.

### 5.3 Backup Codes

- Separate tab has an empty-state explanation, **Add Backup Codes**, and a learn-more link.
- Adding starts by selecting an existing 2FA account from a searchable, sortable list. The next screen displays its website name and username, a multiline backup-code field, **Read Backup Codes from File**, and Save.
- Existing-code management, file formats, and used-code behavior were not tested.

### 5.4 Settings and recovery

- Settings displays counts for 2FA codes, backup codes and trash; **Sync & Backup** with an off status; **Export & Import**; **Auto Lock**; **Sync Time with Internet**; **Language**; **Auto Start**; **Browser Extensions**; **Developer Mode**; and rate, share and contact options.
- Export & Import describes local backup. It offers **Export All Keys**, an **Enable Vault Password** switch, and **Select Vault File** for importing a compatible `.vault` file. Encryption, file format and behavior were not tested; the switch alone does not establish that an export is encrypted.
- Trash shows deleted accounts, a remaining-days label, instructions to swipe left to restore, and **Empty Trash**. Retention policy and actual restore/deletion behavior were not tested.
- A banner advertises access from phone and watch. Its platform support and synchronization implementation were not inspected.

## 6. Functional requirements for the rewrite

### FR-1: Vault and code list — P0

1. Show accounts with recognizable icon, service name, account identifier, current code and time remaining. Support empty, loading, error and locked states.
2. Generate codes locally; update at the exact configured time-step boundary, including when the app resumes from sleep. Use system time as the baseline. Clearly report a detected clock problem without logging codes or secrets.
3. Provide an explicit copy action, accessible by mouse and keyboard, with unobtrusive confirmation. Warn when a code is near expiry or display its remaining life. Clear clipboard only when the clipboard still contains the app's copied value, after a configurable timeout (default 30 seconds); do not overwrite a user's subsequent copy.
4. Search service and account names as the user types. Include a clear no-results state. Support the six observed sort modes and persist the chosen sort. Provide both list and grid views and persist the chosen view.
5. Editing and deleting entries must be explicit; deleting moves an entry to trash after confirmation. Define a predictable permanent-deletion policy and communicate it before deletion.

**Acceptance:** With a test account, searching narrows the list, copy places the current code on the clipboard, expiry updates without restarting, and edits persist after relaunch. No production credential is required for testing.

### FR-2: Create and edit account — P0

1. Support manual entry for service name, optional website address and account identifier, and a required secret. Validate and normalize Base32 secrets; never expose the secret in validation messages or logs.
2. Make **Screenshot to Scan** the primary add-account action. Decode QR codes from a selected screen region or local PNG/JPEG image. Enforce bounded file size, pixel allocation and payload counts; document exact limits in implementation. Parse supported `otpauth://totp` URIs locally and show an editable review before saving. Capture only after an explicit user action.
3. Advanced TOTP options: SHA-1, SHA-256 or SHA-512; 6 or 8 digits; 15, 30, 60 or 120-second periods. Default to SHA-1, six digits and 30 seconds. Preserve values supplied in supported QR URIs.
4. Explain invalid or unsupported QR/URI, invalid secret, missing name, malformed period and incompatible algorithm; retain unsaved form input. Offer a non-blocking duplicate warning based on meaningful matching criteria without revealing the secret.
5. Allow an optional backup-code set to be added during account creation. Require user intent before overwriting an existing entry.

**Acceptance:** RFC test vectors and representative QR URIs produce expected values for all supported parameters; invalid input cannot create a broken entry. Screen/image data remains local.

### FR-2A: Screenshot to Scan — P0, highest priority

1. Expose **Screenshot to Scan** prominently from the codes screen/add-account flow. Require no sign-in, internet connection or screenshot-file save step.
2. On user initiation, hide the main app as needed and show a region-selection overlay. Support multiple monitors and Windows display scaling. Escape cancels and restores the app without changing vault data.
3. Capture only the selected region and decode locally in memory. Show service, account identifier, algorithm, digits and period for review; conceal the secret by default. If multiple supported QR codes are found, let the user select which credentials to review/import.
4. Validate before saving, warn about duplicates and require explicit Save/Import confirmation. On success, show the imported account and current 2FA code.
5. Provide actionable retry, image-import and manual-entry paths for missing QR, unsupported content, malformed credentials, capture permission failure and decoding failure. Preserve reviewed edits where applicable.
6. Do not persist screenshot pixels or raw QR payloads to disk, clipboard, logs or telemetry. Release buffers after success, failure or cancellation. Block capture/import while locked and discard pending capture data when the vault locks.

**Acceptance:** Capture a disposable TOTP QR displayed in another window and verify the imported code against an independent implementation. Test offline operation, real pointer selection, multiple monitors/scales, Escape cancellation, no/invalid/unsupported QR, multiple QR codes, duplicate entries and lock during capture. No screenshot file or plaintext payload remains afterward.

### FR-3: Backup codes — P0

1. List backup-code sets by associated account. Add a set by selecting an existing account and entering one code per line or choosing a local UTF-8 text file (optional BOM; 1 MiB maximum). Trim surrounding whitespace, ignore blank lines, detect exact duplicates and show a review before saving. Reject invalid encoding and excessive input without changing existing data.
2. Encrypt backup codes at rest, conceal them by default, and require an intentional action to reveal or copy. Permit marking a code used and undoing that action; do not assume the service has accepted a code merely because it was copied.
3. Support editing and deleting a set with a clear confirmation. Explain that backup codes are issued by the service, may be single-use, and should be stored separately as appropriate.
4. Define behavior for duplicate or blank lines, large files, invalid encodings, and deletion of the associated account. Avoid storing raw source files.

**Acceptance:** A set can be added from text or a test file, marked used, restored after an encrypted vault round trip, and is not disclosed while locked.

### FR-4: Folders and organization — P1

1. Create and rename a folder with a name and color; select accounts through a searchable list. Display membership clearly and persist changes.
2. Accounts may belong to multiple folders (many-to-many). Removing a folder must not silently delete accounts.
3. Provide visible, keyboard-operable folder navigation and an unfiled/all-accounts view.

**Acceptance:** A folder and its membership survive restart and backup/restore; deleting a folder preserves accounts.

### FR-5: Trash — P1

1. Present deleted entries, deletion time and a clear no-automatic-expiry policy: retain entries until the user explicitly permanently deletes them. Allow restoring an entry and permanently deleting one or all entries, with confirmation for irreversible deletion.
2. Restore the secret, metadata and associated backup codes together where possible; report conflicts and partial failure clearly. Do not silently remove existing entries.
3. Use buttons/context actions as well as gestures so desktop and keyboard users can restore items.

**Acceptance:** Delete and restore a test account without altering its OTP settings; permanent deletion is preceded by a clear confirmation. Trash has no automatic expiry; associated backup codes and recoverable folder relationships follow the account.

### FR-6: Backup and migration — P0

1. Provide clearly labeled **Export encrypted backup** and **Import backup** actions in Settings. Export a versioned, authenticated-encrypted vault to a user-selected local file, including accounts, OTP parameters, folders, backup-code states, trash, preferences and metadata. Require a backup passphrase with confirmation and explain that the app cannot recover it. Never offer plaintext export.
2. Import through file selection, validate version and integrity, unlock with the backup passphrase, preview counts and conflicts, and choose merge or replace explicitly. A failed or cancelled import must leave the current vault unchanged.
3. Write exports atomically; never silently overwrite an existing backup. Communicate destination and success or failure without displaying secret contents.
4. The reference `.vault` import is a discovery target, not a promised format. Implement compatibility only after lawful format analysis and round-trip tests with disposable data.

**Acceptance:** Restore a test export into an empty vault; reject an incorrect password, corrupt file and unsupported version without partial mutation. Verify a merge conflict path and backup portability. A new user can complete either flow with labeled actions, file selection, passphrase entry and a concise review, without an app account or external tools.

### FR-7: Locking and settings — P0/P1

1. Lock vault on user request and after a configurable idle period; a locked screen must conceal codes and require the configured unlock method. Use a local vault passphrase, with optional explicitly enabled OS-backed protection only where it preserves the recovery design. Default idle timeout is five minutes; lock on OS session lock, suspend and relaunch. Explain that a lost passphrase cannot be reset by the app and recovery requires an independently encrypted backup and its passphrase.
2. Settings should show vault counts, lock status, backup/import actions, version and startup preference. Persist changes and show failures. Use English for all UI, help, errors and dialogs; omit language selection and sign-in/profile controls.
3. Use local system time and explain that an incorrect clock can cause rejected codes. Recompute on clock changes and sleep/resume. Offer OS clock-correction guidance; do not implement remote time synchronization or change the OS clock.
4. Keep startup registration optional with enable/disable controls. Do not change security settings without user intent. Browser integration is outside scope.

### FR-8: Local-only operation and English UI — P0

1. Do not implement Google sign-in, OAuth, app login/registration, Authenticator accounts, hosted vaults, cloud backup or cross-device synchronization. A local vault unlock prompt is a security control, not an app account.
2. Keep all persisted user data encrypted on the local system, including metadata and preferences. Export only encrypted files to a user-selected destination; transferring backups is user-controlled.
3. First use, unlock, screenshot scan/import, code generation, organization and export/import must work offline. Do not upload screenshots, secrets, metadata or backups.
4. Ship English-only UI and documentation; do not build translation infrastructure or a language preference.

**Acceptance:** With networking disabled on a clean installation, create a vault, scan/import a credential, generate a code, export and restore. No registration, sign-in, language selection or sync step appears.

## 7. Interaction and visual requirements

- Match the reference UI closely: three tabs (2FA Codes, Backup Codes and Settings), compact dark theme, blue accent, typography, spacing, card hierarchy, timers, navigation and add-account screens. Compare against reference screenshots during implementation. Remove excluded controls and extend the same design language to review, lock, empty and error states. Keep readable contrast and visible focus indicators.
- At narrow and resizable desktop window widths, keep account names, controls and code values readable without overlapping. Do not rely on swipe-only interactions.
- Show confirmation, error and progress states for long-running or destructive work. Do not reveal secrets in toast messages, notifications, persisted screenshots, or diagnostics. User-initiated QR capture stays in memory as specified in FR-2A.
- Provide accessible names for icon-only controls, logical tab order, keyboard shortcuts with discoverable labels, and screen-reader announcements for meaningful status changes. Avoid announcing changing OTP values repeatedly.
- Explain QR acquisition, clock problems, recovery codes, export passphrases and irreversible deletion in concise contextual help.

## 8. Security and privacy requirements

These are **proposed requirements**, not claims about the reference app.

- Threat-model local credential theft, clipboard exposure, malicious imports, lost backup passphrases, stolen backups, compromised devices and malicious QR content. A local app cannot protect secrets from a fully compromised unlocked endpoint.
- Use established, maintained cryptographic libraries; authenticated encryption and a modern memory-hard passphrase KDF with unique salt and calibrated parameters for portable backups. Store format and KDF version for migration. Never invent cryptography.
- Protect local keys with OS facilities and/or a user-held vault secret after deciding recovery requirements. Encrypt all persisted user data: seeds, recovery codes, account metadata, organization, trash and preferences. Prevent database journals, caches and temporary files from exposing plaintext user data. Minimize plaintext lifetime in memory and document platform limitations.
- No secrets, codes, QR payloads, passphrases, account identifiers, screenshots or raw imports in logs or crash reports. Do not include analytics, telemetry or automatic crash uploads.
- Parse untrusted QR images and backup files defensively with size limits, schema validation and safe error handling. Use transactional imports and authenticated integrity checks.
- Prevent lock bypass via notifications or accessible UI; consider screen capture privacy as a platform-dependent enhancement rather than a guarantee. Explain clipboard and backup risks.
- Have the implementation and crypto reviewed before trusting it with irreplaceable accounts. Test first with disposable accounts and retain independent recovery methods.

## 9. Data model (proposed)

| Entity | Fields |
|---|---|
| Vault | schema version, encryption metadata, preferences, timestamps |
| Account | stable ID, service name, optional website and username, encrypted seed, TOTP algorithm/digits/period, icon reference, creation/update timestamps, deletion state |
| Backup code set | stable ID, account ID, encrypted codes, per-code used state and timestamps |
| Folder | stable ID, name, color, timestamps |
| Folder membership | folder ID, account ID |
| Preferences | sort, view, auto-start, lock timeout, clipboard timeout |

Keep secrets out of indexes and filenames. Define migrations and rollback behavior for each schema change. All persisted user-data entities belong inside encrypted storage. Rust is required for application and UI implementation; the Rust UI framework and storage format remain implementation choices.

## 10. Architecture and implementation guidance

- Implement the app in Rust, including its UI, domain logic, OTP generation, screenshot/QR processing, encrypted persistence and OS integrations. Separate these responsibilities and make code generation deterministic under an injectable clock.
- Use a mature OTP/QR library where available and verify against published standard vectors. The account URI parser must validate and normalize inputs before persistence.
- Encrypt portable backups independently of device-bound storage. Keep import/export versioned and test across clean installs. Use atomic writes and transactions to avoid partial vault changes.
- Put capture and startup registration behind OS adapters. Core workflows require no network services. Do not add account services, OAuth, sync, extension messaging, translation infrastructure or telemetry.
- Target Windows desktop for the initial release. Choose a Rust UI framework based on reference fidelity, accessibility, secure storage and capture integration, packaging and maintenance. Other platforms are outside the initial release.

## 11. Validation plan

1. **OTP correctness:** standard TOTP/HOTP underlying algorithm vectors for SHA-1/256/512, supported digits and boundaries; timezone independence; sleep/resume and clock drift. HOTP vectors validate primitives, not a commitment to expose HOTP accounts.
2. **Screenshot workflow:** Run the complete FR-2A acceptance matrix with disposable credentials, including actual pointer selection, Escape cancellation, scaling, local decode, review, import and code verification.
3. **Account workflows:** manual entry, supported QR URIs, image/screen scan, validation errors, duplicates, edit, search, sort, folder membership, and keyboard operation using disposable secrets.
4. **Recovery:** encrypted export/import on a clean installation; wrong passphrase, tampering, malformed/oversized files, conflicts, cancellation, interrupted writes and migration.
5. **Privacy/security:** inspect logs, crash output, clipboard expiry, locked UI, temporary files, encryption and key recovery; independent security review before real use.
6. **UI:** empty/populated/locked/error states, readable compact widths, scaling, focus order, screen reader labels and high-contrast settings. Compare desktop, intermediate and narrow-window screenshots against the reference. Verify English-only text and absence of excluded controls.
7. **Reliability:** restart persistence, concurrent update protection, no data loss on failed writes, offline operation and system sleep/resume.

## 12. Settled decisions for autonomous implementation

The user authorizes reasonable implementation decisions without follow-up questions. These defaults resolve the former discovery backlog; agents must record evidence and actual implementation choices rather than asking for routine confirmation.

1. **Platform and technology:** Windows desktop first; application and UI in Rust. Evaluate egui/eframe first with a reference-fidelity/accessibility/capture spike; select another Rust UI framework if evidence shows it is needed. Record the choice and Windows packaging model in docs/architecture.md. No JavaScript/web UI substitution.
2. **Unlock and recovery:** A local passphrase creates/unlocks the encrypted vault; it is not an Authenticator account. Use established authenticated encryption and a memory-hard KDF, versioned/calibrated during implementation. Default idle lock is five minutes; also lock on OS session lock, suspend and relaunch. OS-backed protection is optional and must not prevent passphrase-based backup portability. No forgotten-passphrase reset/backdoor; recovery uses a separately encrypted backup.
3. **Portable backups:** Native versioned encrypted export/import is mandatory and portable across Windows users/devices without the original device keys. Require a backup passphrase with confirmation for export. Reference-app private `.vault` compatibility is optional evidence-driven discovery; if no lawful documentation/disposable samples are available, document it as unsupported in v1 and continue. Never guess compatibility or require real user credentials.
4. **Account interaction:** Use explicit Copy, Edit and Delete actions, both list and grid views, all six reference sorts and local icons/fallbacks. Do not fetch icons or service metadata over the network. Default app-owned clipboard expiry is 30 seconds and must preserve subsequent user copies.
5. **Folders and trash:** Use many-to-many folder membership and six reference-style color choices. Keep trash until explicitly permanently deleted; no automatic expiry. Restore metadata, seed, associated recovery codes/used states and surviving folder relationships atomically. Folder removal preserves accounts. Do not promise secure filesystem/SSD/old-backup erasure.
6. **QR/image input:** Support user-selected screenshot regions and PNG/JPEG image files, processed locally with resource limits. Multiple supported QR codes require selection and review before atomic approved-batch import. Only supported TOTP URIs are required; HOTP/proprietary/third-party migration formats remain excluded unless separately verified and authorized.
7. **Reference evidence:** Inspect the installed app or existing screenshots non-destructively when available; record evidence paths in docs/reference-ui.md. If unavailable, implement against section 5's observed inventory and mark precise visual fidelity unverified. Extend the same reference language to lock/review/error screens; do not invent evidence or wait for uploads.
8. **Backup-code input:** UTF-8 text with optional BOM, CRLF/LF, up to 1 MiB; trim surrounding whitespace, omit blanks and review exact duplicates before saving. Preserve used state for exact retained codes when editing. Copying never marks a code used; used marking is explicit and undoable.
9. **Settings and help:** English only, no translations or language preference. Offline contextual help covers capture, clock issues, recovery codes, backups, passphrase loss and irreversible deletion. Auto Start is explicit opt-in. Imported preferences must not silently enable startup or weaken local lock policy.
10. **Execution:** Make remaining reversible technical choices autonomously and record material decisions. Never weaken mandatory scope to finish. Record genuine policy/tool/authentication/environment blockers and unperformed checks honestly; no requirement may be marked complete without evidence.

## 13. Suggested delivery sequence

1. Set up the Rust project at https://github.com/ranjbar-dev/authenticator on `main`, select the Rust UI framework and unlock/recovery design, and establish reference screenshots and deterministic TOTP tests.
2. Deliver the defining workflow first with disposable data: **Screenshot to Scan → region selection → local QR decoding → editable review → encrypted save → current 2FA code**. Validate offline operation, cancellation, multiple monitors and scaling.
3. Complete the reference-matching codes list, manual/image entry, search, copy, lock and easy encrypted export/import. Verify recovery on a clean installation before real use.
4. Add backup-code management, folders, trash, accessibility and visual polish. Investigate reference-format migration only after the local vault and recovery workflow are reliable.
5. Commit and push project changes to `main` in the designated repository. Keep secrets, real vaults, screenshots and backups out of version control.

## 14. Implementation task plan and completion evidence

The ordered implementation plan lives in [tasks/README.md](tasks/README.md), with **24 numbered task files**, one per sequential work unit. [tasks/coverage.md](tasks/coverage.md) maps all functional, security, UI, reliability and delivery requirements to tasks and evidence. [tasks/CODEX_PROMPT.md](tasks/CODEX_PROMPT.md) instructs a future Codex CLI session to execute them with one child agent at a time, independent review after each implementation, no routine questions and persistent checkpoints.

Tasks 01–07 establish the Rust/security/UI/capture foundation; task 08 completes the defining screenshot-to-2FA journey before secondary work. Tasks 09–19 complete all remaining workflows; tasks 20–24 cover independent security/visual validation, reference-format discovery, packaging/documentation and the final PRD audit/commit/push.

Plan generation does not constitute implementation. Initial task status is todo, and coverage is planned rather than verified. Completion requires actual acceptance evidence, independent review, a usable Windows package, accurate final documentation and main pushed to the designated repository. Preserve the task files and update PRD.md to the actual final behavior after implementation; do not remove mandatory requirements or fabricate successful checks to declare completion.
