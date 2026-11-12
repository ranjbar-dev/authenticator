# Security design and limits

Task08 connects the existing capture/decoder to ephemeral masked review and
explicit separate-entry batch import. Form strings zeroize; capture pixels/raw
payloads never enter persistence. Lock/cancel/exit drop review and image jobs and
clear editor histories; native picker cancellation closes its own dialog and
generation/watch checks discard late results. Native/allocator/driver/OS copies
remain outside guaranteed erasure. Decode is bounded cooperative work.

The sole repository moves to one save worker. Every approved draft is revalidated
before the complete detached transaction is encrypted and atomically renamed.
Recoverable write failures retain edits and the unchanged committed session.
Cancellation checks and lifecycle/idle guard run at the actual rename gate; a
cancel that loses that gate can leave the **complete encrypted batch**, then a
locked session. No partial batch or claimed rollback after success is allowed.
There is no overwrite import mode: duplicates require an explicit Save-as-new
action and preserve all existing records. Errors contain fixed safe text only.
Current codes derive from real committed records and remain ordinary non-live
accessibility text. Task08's source/transaction/UI evidence and independent-code
comparison do not replace task20's wider security audit or hardware lock tests.

Task07 adds explicit selected-region-only native capture and local decoding.
Preparing ROOT publishes a nonsensitive visual/AccessKit frame before hiding;
busy ROOT omits query/count/account/review widgets. The independent native timer
observes task cancellation, monotonic lifecycle epoch/health, the existing idle
deadline and owner shutdown. Publication/cancellation share a result mutex;
queued results also recheck the native watch and AppState generation/unlocked
state. Lock clears preparation, pending capture/review/alternative intent and
owned widget memory. Restoring ROOT processes lock/result first; it does not
force unlock or expose a previous unlocked framebuffer from the worker.

Only the approved checked RECT reaches BitBlt, after overlay removal and
DwmFlush. Limits before allocation are 4096 per axis and 4,194,304 total pixels;
top-down 32-bit BGRA normalizes reserved alpha 255. Owned pixels zeroize on drop;
mapped DIB bytes are flushed/cleared before deleting the bitmap and RAII releases
GDI/native/pointer resources. No screenshot/QR payload writer, clipboard or
network path exists. GDI/driver/compositor, dependency, allocator/compiler and OS
copies remain outside complete erasure guarantees. Bounded decoding and native
calls are cooperative, not a hard interruption/time/memory-erasure guarantee.
GDI success with black/protected pixels can yield NoQr; there is no secure-desktop
or DRM bypass. See task07 QA evidence for actual private-notification and pointer
checks; actual OS lock/suspend and mixed-DPI hardware gates remain open, along
with task08 editable review/Save and task20's wider security audit.

Task05 adds local create/unlock/Lock, encrypted idle settings and passive Windows
session-lock/suspend/resume notifications. The single-owner worker, generation
and cooperative cancellation boundary are described in architecture.md. A stale
result is locked before UI adoption, including completed-but-unpolled results;
no duplicate store/key cache exists. KDF scratch and its borrowed passphrase
cannot be interrupted mid-call and are released when bounded Argon2 returns.
Lock immediately hides all current UI state, clears owned form/query/pending
state and drops native editor histories. Egui undo storage is released rather
than zeroized internally; allocator/framework/OS copies remain outside complete
erasure claims. Listener registration failure blocks unlock instead of silently
running without OS locking. Actual OS lock/suspend and fully populated future
capture/recovery screens still require their later end-to-end checks.

Task 03 implements the storage boundary described below; the original task 01 framework spike remains inert and has no usable credential flow. Independent task 03 review is pending, and task 20 owns the wider audit. The agent review required by the task plan is not external cryptographic certification.

## Threats

Protect an offline vault copied from disk, copied encrypted backups, accidental plaintext artifacts, malicious QR/images/backups, and partial writes/imports. A fully compromised unlocked endpoint, malicious administrator, process-memory access, pagefile/hibernation, GPU/UI copies and OS crash dumps are outside an app's reliable protection. An unlocked UI intentionally reveals current codes. Local passphrase strength matters; no password reset or backdoor exists in the design.

## Cryptography and file lifecycle

- Random 256-bit DEK per vault, wrapped by a passphrase-derived KEK. Argon2id with unique OS-random salt and calibrated versioned parameters; choose supported parameter floor/ceiling before any KDF allocation. Passphrase changes rewrap, not silently rotate or lose encrypted data.
- Authenticated encryption using a maintained library (proposed XChaCha20-Poly1305); independent random nonces for key wrapping and data. Authenticate type/version/KDF fields as AAD. No handwritten cipher or OTP primitive.
- Encrypted content includes seeds, recovery codes, service/account identifiers, folders, timestamps, trash and preferences. Only public envelope metadata may be outside ciphertext. Generic filenames cannot contain account names. Native backups use independent passphrases/device-independent format and no plaintext export.
- All untrusted lengths, schema references, versions, KDF work factors, image allocations and candidate counts must be bounded before allocation/commit. Validate entire import, preview conflicts, require explicit merge/replace, then commit atomically. Authentication failure cannot be treated as an empty vault.
- Write only ciphertext to same-directory temporary files, flush, atomically replace, preserve original on failures, and serialize writers with concurrent-instance protection. Current-user ACLs; no plaintext cache/database journal, screenshot or QR payload files. Plan migrations on copies and commit only after validation; reject future schema versions.
- Keep decrypted state within an unlock session and discard pending asynchronous results by session generation on lock. Release/zeroize owned secrets, keys, passphrases and capture buffers where feasible. Rust clone, allocator, driver, clipboard and OS copies cannot be guaranteed erased. Never promise secure SSD/filesystem/old-backup erasure.

Task 03 selected/pinned argon2 0.6.0 and chacha20poly1305 0.11.0 with zeroize, plus getrandom 0.4.3. Argon2id version0x13 parameters are bounded before allocation:64..128MiB,3..6iterations,1..4lanes. Creation/rewrap calibrate a64MiB/3iteration/1lane probe toward500ms without lowering the floor. Caller-owned zeroizing Argon2 scratch avoids the library convenience allocator's unwiped scratch. Local data uses a random32-byte DEK and independent24-byte random AEAD nonces; only public envelope metadata stays outside ciphertext. [storage-format.md](storage-format.md) specifies exact encoding/AAD, calibration, API/commit semantics and memory/ACL/filesystem limits. Task 11 must independently prove backup round trips. Task 20 reviews these implementations and their failure boundaries.

## UI and adapters

Lock on explicit request, five-minute default monotonic idle timeout, OS lock, suspend and relaunch. Locked UI/accessibility contain neither account metadata nor codes. Capture/import is blocked while locked, and lock discards pending review state. Screenshots/QR payloads stay in memory; no clipboard use for captures. Only an explicit user action captures an approved rectangle.

Clipboard copy has an explicit action, confirmation and default thirty-second expiry. Clear only if app ownership/sequence and contents still match; preserve subsequent user copies. Clipboard history and other apps can retain copies outside Authenticator's control. Copying recovery codes never marks them used.

No app accounts, cloud, sync, runtime network dependency, icon fetching, telemetry or automatic crash uploads. Do not log secrets, QR payloads, account identifiers, filenames/paths, passphrases, OTPs or recovery codes. Error enums must not embed untrusted text. The task 01 entrypoint suppresses Rust panic payload output; it does not control operating-system dumps. Framework persistence is disabled, and no log subscriber/network client is installed.

Tests use published or generated disposable fixtures and isolated temporary directories. Ignore vaults, backups, captures, crash files and build output in Git. Only reviewed, inert/sanitized UI evidence can be committed. A clean build or passing unit-test count alone does not establish any of these security properties.

## Task 02 implemented domain boundary

Seed/current-code/recovery-code wrappers now zeroize their owned buffers and redact Debug; containing account/draft/folder/membership/preferences/recovery/vault Debug implementations also redact all user metadata. Validation errors remain closed fixed messages and never wrap URI/library error strings. Original borrowed form/QR input remains the caller's responsibility; there is no runtime logger or file persistence in this task. UTF-8 metadata is ordinary session memory, and no complete memory-erasure claim is made.

Strict Base32/URI and field limits, enum-constrained OTP settings, checked boundary arithmetic and whole-vault consistency validation exist in `src/domain/`. URI input never triggers network access. Duplicate matching happens against decrypted session records without a persisted seed index. No serialization derives are enabled on domain records; task 03 uses a bounded explicit codec and validates the whole aggregate after authentication and before commit. Account/folder metadata strings now also zeroize on owned-record drop. Domain collection limits complement the encrypted reader's pre-copy byte/count bounds. Detailed choices and source links are in architecture.md; tests/evidence are in docs/qa/task02/ and docs/qa/task03/. Storage lock/unlock exists, while UI/OS lifecycle integration, backup recovery and wider security review remain later tasks.

## Task 03 validation and remaining boundaries

The implemented local repository starts locked, retains an OS concurrent-instance lock and exposes immutable reads plus whole-aggregate transactions. Wrong passphrase, header/wrapped-key/nonce/payload tampering, truncation, oversized file/KDF settings and authenticated future schema reject without writes. Tests inject partial/pre-rename write failure, a real Windows sharing violation, abrupt child-process termination before rename, cancellation, invalid candidate, migration rejection and panic. Failed operations retain the last committed in-memory aggregate and ciphertext. Passphrase rewrap preserves payload bytes and changes only the wrapping header; old passphrases fail against the new envelope.

Only ciphertext reaches temporary files. Ordinary failed writes remove their pending file; abrupt termination can leave an encrypted orphan, never automatically adopted. Protected DACLs are verified natively against the process-token user SID, and a second process is refused. No corruption reset, plaintext journal, decrypted cache, hidden recovery key or automatic network path exists. Lock drops zeroizing secrets/keys/plaintext buffers, but caller/dependency/compiler/OS copies cannot be guaranteed cleared. The format has no anti-replay counter; old copies/backups remain decryptable under their original passphrases. These tests establish specific behavior, not a claim of absolute durability, memory erasure or cryptographic certification.
