# Portable backup format, envelope 1

This document defines Authenticator's native portable backup. It is independent of
`AUTHLOC1` local vault files, Windows SID, DPAPI, local passphrase, registry and
device keys. File extensions are not used for recognition. Reference-app `.vault`
compatibility is not implemented by this format.

All integer fields are unsigned little-endian. The entire user payload is
encrypted. There is no plaintext export operation.

| Offset | Bytes | Meaning |
|---|---:|---|
| 0 | 8 | Exact ASCII `AUTHPBK1` |
| 8 | 4 | Envelope version, 1 |
| 12 | 4 | Argon2id algorithm/version selector, `0x13` (Argon2 v1.3) |
| 16 | 4 | Argon2 memory in KiB, 65,536–131,072 |
| 20 | 4 | Argon2 iterations, 3–6 |
| 24 | 4 | Argon2 parallel lanes, 1–4 |
| 28 | 16 | Independently random KDF salt |
| 44 | 24 | Independently random key-wrapping XChaCha20 nonce |
| 68 | 24 | Independently random payload XChaCha20 nonce |
| 92 | 4 | Exact payload ciphertext length, including 16-byte tag |
| 96 | 48 | Random 32-byte DEK encrypted under the derived KEK, plus tag |
| 144 | variable | Encrypted payload plus 16-byte tag |

The backup passphrase is separately supplied, exactly 12–1024 UTF-8 bytes, without
trimming or normalization. Export UI (Task12) must collect confirmation before
calling the engine. Argon2id derives a 32-byte KEK using the stored salt/parameters.
Production exports use the existing approximately 500ms calibration and its
64MiB/three-round floor. Tests use that same validated floor, never a weaker KDF.
The DEK, salt and both nonces are freshly generated for each export. No local
wrapping bytes, DEK or local passphrase are reused by the engine.

Both AEAD operations use pinned XChaCha20Poly1305. The wrapping associated data
is ASCII `Authenticator portable backup v1` followed by one NUL byte and bytes
0–95 of this envelope. Payload associated data is that same domain string and
bytes 0–143. Thus every header field and wrapped key are authenticated; this also
separates portable encryption from the local envelope. A local file cannot be
opened as a portable backup, or vice versa.

Payload is the existing bounded binary vault codec specified in
[storage-format.md](storage-format.md): current schema2, with every account's
seed, all OTP parameters/icon/metadata/identity/dates/deletion, all folders and
many-to-many memberships, recovery sets and individual recovery IDs/values/used
dates/deletion, aggregate identity/dates and all preferences. Schema1 has the
existing explicit lazy migration to schema2 defaults for query/filter. Unknown
envelopes, KDF selectors and payload schemas reject without mutation. Future
format migrations must be explicit; there is no guessed or best-effort decoding.

Input must be a bounded regular file: maximum 144 + 64MiB + 16 bytes; ciphertext
length must exactly equal the file remainder, with no trailing bytes. Parameters
are checked before the KDF and plaintext allocation. Codec lengths, enum tags,
UTF-8, field caps, each 10,000-entity collection, total 10,000 recovery codes and
all references/timestamps are checked before preview. Portable input additionally
requires globally unique nonnil IDs across all entity types. Authentication,
wrong passphrase or validation failure never returns an empty successful vault.

## Import decisions and identity

`preview` consumes an owned zeroizing passphrase and authenticates a selected
file. Its opaque owner holds an immutable incoming/local snapshot, exact source
ciphertext SHA256 and local committed-ciphertext SHA256 revision. These are
ciphertext digests, not seed or passphrase fingerprints. Sensitive read-only
`incoming_vault` supports the unlocked review UI. Debug for these owners and
decisions is completely redacted.

Conflicts use the existing `AccountDraft::duplicate_criteria`: same decoded
seed/settings or Unicode-lowercased service/account identity, including trash.
Stable same-ID account conflicts are independently reported. Ephemeral ordered
indices borrow seed bytes and hold zeroizing lowercase metadata; no seed digest
or external index persists. Incoming-incoming duplicates are reported too.
Each incoming account has at most one bounded conflict summary containing exact
local/prior-incoming match counts, representative indices and aggregate reason
flags (reasons need not all describe the same representative). A combined
seed/settings-and-identity index subtracts overlaps, so counts do not double-count
one record matching both criteria. Representatives are checked against the shared
domain policy. At most 10,000 account summaries plus one aggregate folder/global
collision warning exist. Both unique and dense 10,000-account snapshots support
explicit Replace; conflict pairs are never enumerated or silently truncated.
Exact folder-name equality is unchanged by this credential policy.

`plan(options)` consumes the preview and builds/validates a concrete final
candidate; `candidate_for_review` and `summary` let the UI present final changes
before explicit apply. Decisions cannot be forged into or applied to another
snapshot. A plan is consumed by apply. Apply rechecks the local encrypted revision
and exact current source file bytes, then calls the existing sole-owner atomic
local transaction. Changing either file invalidates the plan; applying a second
old plan after a commit fails. The candidate never depends on rereading a changed
source's plaintext.

| Decision | Exact behavior |
|---|---|
| Merge, default KeepLocal | Preserve every current record. Skip conflicting incoming accounts together with their incoming memberships and recovery sets/codes. Reuse existing same-stable-ID folders with their current metadata/trash; copy folders whose stable IDs are new. Repeated identical default Merge is domain-idempotent. |
| Merge, per incoming account KeepBoth | Add that complete incoming bundle. Remap every globally colliding incoming record ID and every associated account/folder/membership/recovery reference. Preserve all used/trash dates. Existing records remain byte-equivalent in the domain codec. |
| Replace | Explicitly replace all account/folder/membership/recovery collections with incoming collections, including trash/used state. Preview reports all removed local counts. Per-account merge choices are rejected in this mode. |

Per-folder KeepBoth explicitly makes a separate copy instead of same-ID reuse.
Explicit per-folder KeepLocal can also reuse a same-name/different-ID current
folder; if several match, the lowest stable ID is selected deterministically.
Without that choice, different stable IDs remain distinct even with equal names;
the collision is warned and schema folder-name rules are not changed. Replace
rejects per-folder merge choices. Copied folders retain incoming names/colors/dates;
reused folders retain current metadata/trash. Incoming memberships follow the
selected folder resolution atomically. Plan `folder_copies` lists every incoming
index, original/resulting ID, copied/reused flag, existing ID/name collision and
both incoming/resulting trash flags. Names/colors/dates are available through the
sensitive candidate; UI must show these resolutions and incoming metadata before
apply. No existing folder is changed in Merge. All references change together. Aggregate overflow,
invalid/orphan references or unresolved global ID collisions fail before any transaction.

An incoming folder ID can also be occupied by a current account, recovery record,
vault or preference. When no same-ID folder exists, default Merge first looks for
an exact current folder-content match: name, color, creation/update dates and full
trash state/date. It reuses the lowest matching stable ID, or creates a remapped
copy if none exists. This makes repeating an identical backup idempotent after
such a collision; it does not infer provenance or match ordinary different-ID
folders by name alone. Explicit KeepBoth still creates a separate copy. KeepLocal
is rejected if there is no current same-ID/name folder to keep. Preview flags
`local_global_id_collision` and `matching_content_reused` distinguish this policy
from ordinary same-ID reuse, and all memberships/filter references follow the
resulting ID. Existing folders remain unchanged even if several identical folders
match. Users can explicitly request separate organization through KeepBoth.

In both modes the destination vault identity, creation/deletion metadata and
preferences identity are preserved. Imported aggregate update time can advance
the logical update time. Imported per-record creation/update/trash/used dates are
preserved. All existing Vault aggregate mutation methods clamp new logical update
time to the aggregate maximum, so future imported dates or a corrected wall clock
do not prevent subsequent edits/creation. The OS clock and OTP wall time are never
changed. Direct per-record `BackupCode::set_used` still requires a nondecreasing
time; future Task16 must pass `vault.mutation_time(actual_time)` and update its
containing set/aggregate atomically.

Local startup is always preserved; the engine never registers startup or performs
OS side effects. Local display, lock and clipboard preferences are preserved by
default even in Replace. Separate explicit preview options can import display
(query/sort/view/filter) and security (lock/clipboard timeout) fields. Local
preferences identity/creation/deletion stay local; update time becomes logical
aggregate time when imported preference fields or the filter fallback apply.
Otherwise the complete local preferences record is preserved. Imported
selected-folder identity follows its remap. A retained
local filter whose folder disappears in Replace falls back to All and sets the
visible `selected_folder_reset` summary flag. Task13 must invalidate cached
Codes/Backup Codes queries, IDs, filters and codes after successful import.

## Portable writes and limits

`Destination::create_new` is explicit create-only intent. Existing files require
`overwrite_reviewed` after UI overwrite consent; this binds consent to the exact
reviewed bytes. Task12's `Destination::review` alternative inspects the target
on a worker without granting consent, returning an opaque `DestinationReview`.
The UI consumes `approve` only after an explicit Export or named overwrite
confirmation. It retains the same inspected bytes/absence across the confirmation;
approving performs no I/O. The writer checks them again inside commit admission.
Changed/missing/replaced targets fail. Native create-only rename also refuses a
concurrent newly created destination. The writer only writes ciphertext into a
random generic `pending-UUID.bin` beside the target, flushes the file and performs
one same-volume native move/replace. Cleanup starts only after `create_new`
success proves temporary-file ownership. After a successful move cleanup is
disarmed. No delete-original/rename pair, parent ACL change or `session.lock` is
used. Local `vault.bin` and `session.lock` destination names are rejected.

Ordinary absolute drive paths, existing parents and regular files are required
on Windows. UNC/device paths, ADS, ambiguous reserved names, parent traversal,
symlinks and reparse points are rejected. Reads check both path and opened-handle
metadata and use a bounded take to detect growth. The file implementation has no
path-sensitive error text or Debug output.

Cancellation is cooperative and checked before work/commit. Existing storage
atomic admission decides the race: cancellation before admission prevents the
write; after admission the complete encrypted write can succeed and returns
success. There is no fallible stage after native commit. No forced interruption
or immediate in-flight crypto/I/O memory-erasure guarantee is made. Future UI
must discard stale worker results after Lock rather than adopt them.

Windows uses `MoveFileExW` with WRITE_THROUGH and explicit REPLACE_EXISTING only
for consented replacement. Unix fallback uses atomic rename for replace and
create-only hard link for new files. Filesystem/device durability, stalled I/O,
power loss and hostile concurrent directory/target substitutions remain OS
limits. Byte revalidation is not a filesystem compare-and-swap against a hostile
writer in the final check-to-rename interval. Parent directories are not anchored
with retained no-reparse handles. This engine protects normal reviewed concurrent
changes and rejects observed unsafe endpoints; it is not a defense against an
attacker controlling the destination directory or the unlocked process.

Interrupted exports may leave generic partial/complete ciphertext pending files,
never plaintext. They do not delete the prior backup. Pending files are not
automatically enumerated/deleted at startup. Password loss has no bypass; old
backups retain their original encrypted records and passphrases. Zeroizing owned
buffers/records/keys/KDF scratch is best effort, with existing compiler, allocator,
library, pagefile and crash-dump limits. Physical secure erasure is not promised.
