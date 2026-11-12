# Local vault format, version 1

Task09 review correction changes only cancellation/admission, not this format.
The former shared cancellation/rename mutex could block UI Lock during native
I/O. Current `OperationCancellation::cancel` atomically requests cancellation
without waiting. An atomic admission transition competes with that request;
separate background serialization preserves sequential shared-token use. Cancel
winning admission prevents final replace; once admitted, the complete encrypted
replace may finish or fail normally. Admission is not durable commit: native
rename success remains the commit point. Lifecycle/idle guard checks precede
admission, and UI generation/late-result checks prevent cancelled session adoption.
Active I/O/crypto worker keys/plaintext are not forcibly erased while in use;
their owners lock/drop when that work returns. UI/AX/form cleanup does not wait.

Task08 leaves the envelope/schema unchanged. `transaction_cancellable` extends
the existing detached transaction with the creation-style cancellation admission
and optional fail-closed lifecycle/idle guard before final replace. An entire
approved QR batch commits once. Fault/validation/cancellation before admission
preserves disk and committed memory; cancellation after admission can leave the
complete ciphertext batch and the UI locks/discards its late result. No fallible
step follows rename, no false rollback is reported and no committed file is
deleted on cancel. Ordinary `transaction` delegates with an uncancelled token.
The import UI retains reviewed edits on recoverable write errors.

Task05 keeps the envelope/schema unchanged and adds a cooperative cancellation
gate for creation. `create_cancellable` checks cancellation before expensive
creation and uses final-action admission (the former cancellation mutex was
superseded by Task09 review). If cancellation wins admission, pending ciphertext
is removed best effort and no vault is created; if admission wins, a complete ciphertext file can remain and the
UI discards/locks the late session. No cancellation path deletes a committed
vault. Existing `create` uses an uncancelled token, preserving its API semantics;
transactions/rewrap keep their existing commit boundary. Passphrase work remains
bounded and off the UI thread, not forcibly interruptible inside Argon2.

Implemented by task 03 on 12 November 2026. This is the local format only. Task 11 must use a distinct discriminator and independently generated DEK/salt/nonces and backup passphrase; copying the local file is not the portable backup workflow. No device key or recovery backdoor is involved.

## Envelope

All integers are unsigned little endian, with no padding. Exact byte offsets:

| Offset | Bytes | Meaning |
|---|---:|---|
| 0 | 8 | ASCII `AUTHLOC1`, local format domain |
| 8 | 4 | Envelope version, exactly 1 |
| 12 | 4 | Argon2 version, exactly 0x13; algorithm fixed Argon2id |
| 16 | 4 | Memory cost in KiB |
| 20 | 4 | Iterations |
| 24 | 4 | Lanes |
| 28 | 16 | Random salt |
| 44 | 24 | Random wrapping nonce |
| 68 | 48 | Encrypted 32-byte random DEK and 16-byte AEAD tag |
| 116 | 24 | Random payload nonce |
| 140 | 4 | Payload ciphertext length, including tag |
| 144 | variable | Encrypted complete schema and 16-byte tag |

Both operations use RustCrypto XChaCha20-Poly1305. Wrapping AAD is exactly bytes 0..68 (exclusive end). Payload AAD is exactly bytes 0..12. This deliberately allows rewrapping the same random DEK without reencrypting the payload. Each field is either checked against an exact constant/length, covered by wrapping AEAD, used as an AEAD nonce or authenticated payload. Nonces use independent OS randomness for every new wrap/payload. Payload length must equal the exact remaining file length and remain within the global cap. An old complete encrypted revision can be replayed by someone able to replace files; there is no trusted monotonic anti-rollback counter. Passphrase changes cannot revoke old ciphertext copies or backups.

Keys are 32 bytes, salts 16, nonces 24, tags 16. `getrandom 0.4.3` uses Windows ProcessPrng; errors fail closed. Passphrases are exact UTF-8 bytes (12..1024 bytes), without trimming/normalization. Creation and rewrap calibrate Argon2id with a real random probe at 65,536 KiB/3 iterations/1 lane, targeting 500 ms by selecting `ceil(1500 / measured_probe_ms)` iterations, clamped to 3..6. Slow devices keep the floor. Stored accepted bounds are 65,536..131,072 KiB, 3..6 iterations, 1..4 lanes; check all before KDF allocation. These are application policy limits, not the crate's permissive maxima. One operation can still consume up to 128 MiB and several seconds on a slow device. UI must keep KDF work off its paint/event loop in task 05, with session generation handling for late results. Measured debug calibration on this machine took about 2 seconds at the floor; optimized packaging timing remains task 23 evidence.

Pinned maintained primitives: argon2 0.6.0 (zeroize, explicit owned scratch), chacha20poly1305 0.11.0 (zeroize), zeroize 1.9.1 and getrandom 0.4.3. Checked current official [Argon2 API](https://docs.rs/argon2/0.6.0/argon2/), [XChaCha API](https://docs.rs/chacha20poly1305/0.11.0/chacha20poly1305/) and [OS randomness API](https://docs.rs/getrandom/0.4.3/getrandom/). [RFC 9106](https://www.rfc-editor.org/rfc/rfc9106.html) describes the algorithm and a memory-constrained 64 MiB profile; our single-lane bounded calibration is an explicit application choice, not a claim to implement an exact RFC recommended profile.

## Authenticated plaintext schema

Only decrypted in zeroizing memory. No serde/general-purpose deserializer is used. `codec.rs` is the normative field order; all enums have fixed explicit match tables on decode. Current payload schema is **2**; envelope version remains **1**. The original version 1 layout is:

1. u32 schema (exactly 1), vault UUID bytes and record state.
2. Preferences UUID/state, sort/view/startup u8s and timeout u32s.
3. u32 account count; each account UUID/state, service, optional website/identifier, seed bytes, algorithm/digits/period/icon u8s.
4. u32 folder count; each UUID/state, name and color u8.
5. u32 membership count; UUID/state and folder/account UUIDs.
6. u32 recovery-set count; each UUID/state, owning account UUID and u32 code count; each code UUID/state, value and optional used timestamp.

State is creation u64 seconds, update u64 seconds, deletion boolean u8, and deletion u64 seconds only when trashed. Optional values are boolean u8 and value when present; only 0/1 accepted. Strings and blobs are u32 byte length plus exact bytes; strings must be UTF-8. UUIDs are 16 bytes and cannot be nil. No trailing bytes are permitted. Trashed records/memberships and recovery used state retain their complete data.

Maximum plaintext is 64 MiB, maximum file is 67,109,024 bytes (144+64MiB+16). The public file reader checks metadata then uses a bounded take, covering growth races. Each count is <=10,000 and <=remaining bytes/minimum record size before loops; vectors grow from parsed records rather than reserving attacker-requested capacities. Total recovery codes <=10,000. Each field length is checked before slicing/copying: service/folder256, website2048, identifier320, seed1024, recovery value4096 bytes. The complete candidate must pass `Vault::validate`, including normalized fields, UUID uniqueness/references and timestamp ordering. This imposes additional consistency bounds after bounded parsing.

Schema 2 adds fields immediately after the existing clipboard timeout: u32 UTF-8 query byte length + query bytes (0..1024, no control characters), then folder-filter u8 (0 All, 1 Unfiled, 2 Folder) and a nonnil folder UUID only for tag2. All original fields retain their order and encoding. The selected folder must reference an active existing folder. Folder-removal operations must reset a selected removed folder to All in the same transaction. The query, selected folder, sort, view and all other preferences remain inside the authenticated encrypted payload; no framework preference file or plaintext index is used.

The bounded decoder explicitly accepts schemas1 and2. Schema1 is transformed in detached memory to schema2 with empty query and All filter; every original ID, state, field, association, used code, preference and wrapping key is retained and validated. Unlock performs **no migration write**. The next successful ordinary atomic encrypted transaction serializes schema2. Schema0/future versions, corrupt/truncated old payloads and invalid references fail without writes. The frozen independently packed public schema1 fixture in `tests/fixtures/schema1-public.bin` includes account/folder/membership/recovery IDs, used state and nondefault preferences. Tests encrypt that exact legacy payload with the unchanged real envelope, reopen/unlock without a write, inject a failed upgrade write, then commit/reopen schema2 and check retained data and encrypted query. This is native Authenticator schema migration; it makes no claim about the excluded reference application's private `.vault` format.

## Files, transaction commit and session lifetime

Production resolves FOLDERID_LocalAppData through SHGetKnownFolderPath, then uses `Authenticator/vault.bin`, empty `session.lock`, and random generic `pending-UUID.bin` files. Test/packaging adapters can explicitly supply an isolated directory; there is no UI path override. No plaintext database, journal, cache, screenshot or serialization file exists. Existing corrupt/unsupported vaults block creation and are never treated as empty.

Windows applies a protected DACL allowing only the current process-token user SID, with full access and child inheritance on the directory; vault, lock and temporary files also get that DACL. ACL failure is fatal. Administrators can take ownership; ACLs are not protection against the same user, administrators, malicious drivers or a compromised endpoint. Reparse points at the app directory and concrete file endpoints are rejected. This is not a complete adversarial directory-race defense against the same user. Non-ACL filesystems fail instead of silently weakening protection.

The repository holds `File::try_lock` for its full lifetime, including while locked. `&mut VaultStore` serializes writes; share behind a Mutex if multiple app owners need it. Transactions clone the aggregate, apply the whole operation, validate and encrypt the candidate, verify the current file matches the last loaded ciphertext, write ciphertext only to a newly created same-directory file, sync_all, close, and call same-volume MoveFileExW with REPLACE_EXISTING|WRITE_THROUGH (creation omits REPLACE_EXISTING). COPY_ALLOWED is never set. Native rename success is the commit point. There is no delete-original/rename pair, no fallible operation after success and no directory-sync error incorrectly reporting rollback after commit. Cleanup is best effort and cannot turn success into failure. Failed/interrupted precommit writes leave the previous vault; process termination can leave an encrypted orphan that is never adopted automatically. See [Microsoft MoveFileExW semantics](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw). Sudden hardware power loss, filesystem/filter-driver defects and remote filesystems are not certified by process-termination tests; there is no absolute power-loss durability guarantee.

An open repository starts locked. Unlock reads bounded ciphertext, verifies both AEAD layers, decodes/validates the whole aggregate and only then installs a session. Failed unlock discards any prior session. Lock/drop clears the session, DEK and in-memory committed ciphertext snapshot. KEK/passphrase/decoded plaintext/wrapped plaintext key use zeroizing owners; Argon2 scratch is explicitly caller-owned and zeroizing because the crate's allocating Blocks convenience method does not wipe its allocation. Cipher objects have zeroize support enabled. Serialization reserves the fixed cap plus tag space once to avoid freed plaintext copies on Vec growth; key-wrap buffer reserves tag space as well. Domain seeds/recovery values already zeroize, and metadata strings now clear on AccountDraft/Folder drop.

Best-effort clearing does not erase caller-created clones/borrowed form input, metadata-validation temporaries, register/stack/compiler copies, all dependency temporaries, allocator history, pagefile, hibernation, OS dumps, UI/clipboard/driver copies or old encrypted revisions. No complete process-memory erasure or secure SSD deletion is promised. Explicit/OS/idle/suspend UI lifecycle integration remains task 05; the storage API itself has no UI or asynchronous work.
