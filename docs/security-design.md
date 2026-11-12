# Security design and limits

This is a design boundary recorded before persistence work. Task 01 does not implement a vault and must not be used for real credentials. The agent review required by the task plan is not external cryptographic certification.

## Threats

Protect an offline vault copied from disk, copied encrypted backups, accidental plaintext artifacts, malicious QR/images/backups, and partial writes/imports. A fully compromised unlocked endpoint, malicious administrator, process-memory access, pagefile/hibernation, GPU/UI copies and OS crash dumps are outside an app's reliable protection. An unlocked UI intentionally reveals current codes. Local passphrase strength matters; no password reset or backdoor exists in the design.

## Planned cryptography and file lifecycle

- Random 256-bit DEK per vault, wrapped by a passphrase-derived KEK. Argon2id with unique OS-random salt and calibrated versioned parameters; choose supported parameter floor/ceiling before any KDF allocation. Passphrase changes rewrap, not silently rotate or lose encrypted data.
- Authenticated encryption using a maintained library (proposed XChaCha20-Poly1305); independent random nonces for key wrapping and data. Authenticate type/version/KDF fields as AAD. No handwritten cipher or OTP primitive.
- Encrypted content includes seeds, recovery codes, service/account identifiers, folders, timestamps, trash and preferences. Only public envelope metadata may be outside ciphertext. Generic filenames cannot contain account names. Native backups use independent passphrases/device-independent format and no plaintext export.
- All untrusted lengths, schema references, versions, KDF work factors, image allocations and candidate counts must be bounded before allocation/commit. Validate entire import, preview conflicts, require explicit merge/replace, then commit atomically. Authentication failure cannot be treated as an empty vault.
- Write only ciphertext to same-directory temporary files, flush, atomically replace, preserve original on failures, and serialize writers with concurrent-instance protection. Current-user ACLs; no plaintext cache/database journal, screenshot or QR payload files. Plan migrations on copies and commit only after validation; reject future schema versions.
- Keep decrypted state within an unlock session and discard pending asynchronous results by session generation on lock. Release/zeroize owned secrets, keys, passphrases and capture buffers where feasible. Rust clone, allocator, driver, clipboard and OS copies cannot be guaranteed erased. Never promise secure SSD/filesystem/old-backup erasure.

Task 03 must select/pin actual crypto libraries, calibrate and document parameters and test corruption/wrong-password/truncation/interrupted writes/concurrency before claiming storage security. Task 11 must independently prove backup round trips. Task 20 reviews these implementations and their failure boundaries.

## UI and adapters

Lock on explicit request, five-minute default monotonic idle timeout, OS lock, suspend and relaunch. Locked UI/accessibility contain neither account metadata nor codes. Capture/import is blocked while locked, and lock discards pending review state. Screenshots/QR payloads stay in memory; no clipboard use for captures. Only an explicit user action captures an approved rectangle.

Clipboard copy has an explicit action, confirmation and default thirty-second expiry. Clear only if app ownership/sequence and contents still match; preserve subsequent user copies. Clipboard history and other apps can retain copies outside Authenticator's control. Copying recovery codes never marks them used.

No app accounts, cloud, sync, runtime network dependency, icon fetching, telemetry or automatic crash uploads. Do not log secrets, QR payloads, account identifiers, filenames/paths, passphrases, OTPs or recovery codes. Error enums must not embed untrusted text. The task 01 entrypoint suppresses Rust panic payload output; it does not control operating-system dumps. Framework persistence is disabled, and no log subscriber/network client is installed.

Tests use published or generated disposable fixtures and isolated temporary directories. Ignore vaults, backups, captures, crash files and build output in Git. Only reviewed, inert/sanitized UI evidence can be committed. A clean build or passing unit-test count alone does not establish any of these security properties.
