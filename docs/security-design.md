# Security design and limits

Review09 corrected a responsive-lock defect: the old cancellation mutex could
make UI Lock wait for an admitted native replace. Current cancellation is an
atomic request competing with atomic action admission; it does not take the
background action-serialization mutex. Generation/query/intent and UI-owned
review/widget state clear without waiting for that action. Completion polling/
cleanup also avoid waiting on publication, and stale results cannot reopen the
UI. Cancellation winning admission prevents the replace; an already admitted
whole encrypted operation may complete, with native rename success remaining
the durable commit point. A deterministic actual-save gate test proves UI/AX
cleanup before release, then locked late completion and whole-batch unlock.

This is a UI responsiveness/adoption boundary, not an immediate worker-memory
erasure guarantee. Active Argon2/native filesystem work cannot be safely forced
to stop; its worker may still own the vault/DEK/candidate/scratch until it returns.
Cancellation records intent immediately; worker owners then lock/drop on return.
No promise is made to erase all dependency, allocator, pagefile or OS copies.

Task09 extends the existing cancellable save worker with explicit Import/Create/
Edit intent. Stable-ID edits validate an ephemeral original snapshot at the actual
transaction boundary (including same-second changes), reject missing/trashed
targets, and preserve the live seed unless replacement was deliberate. Account
creation and optional recovery codes commit as one encrypted aggregate. The
schema/envelope is unchanged; existing rename/lifecycle/cancellation semantics
still govern, including the possibility of a complete committed ciphertext file
when lock loses the rename gate. No postcommit rollback/deletion is promised.

New raw/normalized recovery and edit snapshot buffers use zeroizing owners;
bounded text normalization never embeds values in errors. Concealed fields keep
raw values out of rendered/AX text, while an explicit Reveal intentionally permits
visible layout and AccessKit text-run exposure. A password parent role alone does
not prove that a custom visible galley's AX children are masked. Tests inspect
full node values and Copy/Cut output: clipboard plaintext output is blocked in
both modes, but Cut can delete selected input. Conceal/lock drops editor histories
and owners, not every possible framework/font/allocator/OS copy. Task09 evidence
records actual isolated create/edit/write-failure/Cancel/process-relaunch/HMAC
checks and the outstanding independent/manual gates. Standalone recovery file/
used-state workflows remain Task15/16.

Task08 connects the existing capture/decoder to ephemeral masked review and
explicit separate-entry batch import. Form strings zeroize; capture pixels/raw
payloads never enter persistence. Lock/cancel/exit drop review and image jobs and
clear editor histories; native picker cancellation closes its own dialog and
generation/watch checks discard late results. Native/allocator/driver/OS copies
remain outside guaranteed erasure. Decode is bounded cooperative work.

The sole repository moves to one save worker. Every approved draft is revalidated
before the complete detached transaction is encrypted and atomically renamed.
Recoverable write failures retain edits and the unchanged committed session.
Cancellation and lifecycle/idle guards run at final-action admission; a
cancel that loses admission can leave the **complete encrypted batch**, then a
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
Lock immediately hides all current UI state, clears UI-owned form/query/pending
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
## App-owned clipboard leases (Task10)

Explicit code Copy recomputes current digits and sends no seed to the dedicated
clipboard actor. Counter/period validity, session generation and native lifecycle
watch are checked at mutation admission and immediately before text publication.
Expiry/clock jumps while queued reject the old digits. A call already in the OS
cannot be forcibly revoked; an already admitted late copy is cleared best effort
and cannot publish stale success after Lock. The UI confirms actual actor success
in a visible shared footer. Framework text-copy commands are routed through the
same guard; revealed secret editors still block framework CopyText themselves.

GlobalAlloc uses MOVEABLE|ZEROINIT. Owned allocation guards free only on failed
SetClipboardData and attempt wiping such failed buffers; successful publication
transfers memory to Windows. Clipboard close is RAII. Registered marker and zero
DWORD CanIncludeInClipboardHistory/CanUploadToCloudClipboard formats are published
before UTF-16 text. A final sequence is established after the observed close-time
sequence change by verifying the owned HWND, random marker and exact salted
content digest under a fresh exclusive open. Provisional evidence on Busy is never
rebased to a later sequence. Cleanup checks sequence and HWND before reading the
marker, then checks the marker before a bounded read of the app's own Unicode
format. It allocates no buffer from an unrelated clipboard's declared size.
Ownership verification and EmptyClipboard share one OpenClipboard interval.

The lease retains only marker, digest/length, sequence and HWND; it does not retain
raw expected OTP text during expiry/Busy retries. The queue is bounded to8 texts,
each <=1MiB and zeroizing on drop. Lock invalidates queued generations immediately;
an OS call already running can retain its current/queued buffers until it returns.
Busy cleanup warns after2seconds and keeps retrying without raw expected text.
Native OpenClipboard failures do not reliably distinguish a competing holder from
all access failures, so persistent denial is also an honest cleanup limitation.
Shutdown's UI wait is bounded300ms and the actor's stop retry budget is2seconds.
Neither can guarantee cleanup after abrupt termination, a stalled kernel/API call,
denied station access, device failure, Windows/third-party history or clipboard
observers. No private clipboard contents are backed up/restored or logged.

Primary contracts checked during implementation: [OpenClipboard](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-openclipboard),
[SetClipboardData](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setclipboarddata),
[sequence numbers](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclipboardsequencenumber),
[clipboard formats/history hints](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats),
and [window stations](https://learn.microsoft.com/en-us/windows/win32/winstation/window-stations).
Tests use a separate process and a newly created random CREATE_ONLY noninteractive
window station/desktop for actual Win32 clipboard races. They never open WinSta0's
clipboard or switch the input desktop. This is native ownership evidence, not a
substitute for the actual user-pointer journey or hardware lock/suspend delivery.

## Portable backups (Task11)

Native `AUTHPBK1` backups are independent of local wrapping keys and Windows user
or device secrets. A fresh random DEK is wrapped with a separately entered exact
backup passphrase using the same bounded Argon2id floor and reviewed AEAD
primitives. Both wrapping and payload authenticate explicit portable domain data
and all header fields; local vault bytes are not accepted as backups. The full
payload, metadata, preferences, trash and per-code used state are encrypted.
[portable-backup-format.md](portable-backup-format.md) is normative for byte
offsets, limits, migration and decisions.

Parsing rejects unbounded lengths/unknown versions/KDF parameters before heavy
work. Wrong password/authentication/validation errors never yield an empty
successful preview. Opaque previews/plans own immutable candidates, bind to exact
ciphertext source/local revisions and expose only sensitive read-only review
accessors. Debug and error text never include paths, passphrases, IDs or metadata.
Duplicate comparison indices borrow seed data; owned lowercase metadata and
plaintext/key/KDF buffers use zeroizing owners. Dense conflicts reject rather
than silently truncate. Memory/allocator/OS erasure limits remain unchanged.

The portable writer deliberately never modifies Desktop/Documents/USB parent
ACLs or creates a local vault/session lock there. Its same-directory pending
files contain ciphertext only. Cleanup becomes active after successful exclusive
creation and is disarmed after a native move. Reviewed overwrite consent is bound
to prior bytes and revalidated inside nonblocking commit admission. Pre-admission
cancellation/faults preserve the prior file; an admitted whole encrypted commit
may finish after cancellation. Process termination before rename preserves the
prior backup, leaving at most generic ciphertext pending files.

Path/handle regular-file checks and bounds reject observed symlinks/reparse/device
endpoints. They do not anchor directory handles or provide a hostile filesystem
compare-and-swap during the final revalidation-to-rename interval. Device stalls,
power-loss durability and adversarial directory replacement remain OS limits.
All imported startup remains local with no registry action. Local security policy
stays current unless an explicit plan option adopts imported lock/clipboard
values. Full UI confirmation/session integration and actual different Windows
user/device recovery remain later gates, not inferred from engine tests.

Task11 also repairs a historical Task03 cleanup ownership error in the local
atomic writer used by import: its previous guard was constructed before exclusive
temporary-file creation succeeded, so an exact preexisting pending-name collision
could cause deletion of an unowned file. Both local and portable guards now start
only after successful creation, close owned handles before failure cleanup and
disarm when a move consumes the name. Separate deterministic collision regressions
preserve the unowned file and prior ciphertext. This correction does not change
local ACL policy, encryption or transaction admission.
