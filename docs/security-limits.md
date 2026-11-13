# Security and validation limits

The entire local vault payload—seeds, recovery values, metadata, relationships,
Trash and preferences—is authenticated-encrypted. Only public envelope/version/KDF
metadata remains outside it. Argon2id wraps a random data key; XChaCha20-Poly1305
protects the payload. Backups use independent random keys/nonces and a separate
passphrase. Windows startup registration necessarily contains the public quoted
executable path, not credential data. Do not infer safety from a filename.

The implementation has had separate agent source/security/data-loss reviews and
published/disposable-vector tests. That is not external cryptographic certification.
Use independent service recovery methods, a strong passphrase and verified backups.

A compromised unlocked endpoint, administrator, clipboard observer, OS dump,
pagefile/hibernation file, GPU/framework/allocator copy or old backup can retain
sensitive information. Owned secret buffers are cleared where feasible; complete
memory or SSD erasure is not promised. Clipboard history suppression is a hint,
not control over every manager. Expiry/Lock cleanup only clears exactly owned
data and may be delayed or unavailable when Windows will not grant access.

Lock hides data and invalidates UI sessions promptly. Worker crypto/native I/O
cannot safely be forcibly interrupted. Cancellation that wins final-action
admission prevents writing; an already admitted complete encrypted operation may
finish. The app does not claim rollback after a committed save. Imports are whole
validated transactions, with late unlocked UI adoption blocked after Lock.

Atomic replacement and checked file sharing are not a physical power-loss
durability guarantee or protection against every malicious same-user filesystem
race. ACLs do not defend against the same user, administrators or compromised
drivers. Existing encrypted files can be replayed; a passphrase change cannot
revoke old copies or independently encrypted backups.

The release is unsigned. It is a portable Windows x64 build, not a Store/MSIX
installer or a cloud service. No paid signing identity is implied. Build checks,
native disposable journeys and package-specific checks are separately recorded;
do not equate a successful build with complete UX/security acceptance.

At this checkpoint, actual OS lock/suspend/logon delivery, physical mixed DPI,
Narrator/high contrast, networking-disabled clean-machine execution, complete
state/scale matrices and exact installed reference-app pixel parity remain open.
The public reference marketing images depict phones and excluded cloud/widgets;
they do not establish native Windows equivalence. Another Windows-user/VM recovery
is verified only if the accompanying task23 evidence says it was actually run.

No credentials, screenshots, QR payloads or backups are uploaded by the app.
Development dependency downloads, source hosting and CI artifact distribution are
build-time services. The first-release migration decision supports native
`AUTHPBK1` backups only; private reference `.vault` compatibility is unproven and
unsupported. Keep irreplaceable accounts out of acceptance testing.
