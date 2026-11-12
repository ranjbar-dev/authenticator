# Task11 implementation evidence

Author implementation evidence for the independent security/data-loss reviewer;
this report does not certify acceptance. No UI, task status, coverage, PRD scope,
commits or pushes are owned by this implementation child.

Implemented `src/backup/{mod,envelope,files,merge,tests}.rs`, public module export,
minimal crate-private reuse of reviewed local codec/crypto primitives and an
opaque encrypted local revision accessor. Existing Vault aggregate mutation
methods now clamp logical time after a future-dated import; the explicit domain
backwards-clock regression was changed to assert successful edit with unchanged
creation date/nondecreasing update time. Direct recovery-code mutation still
requires the containing vault logical time, documented for Task16 integration.

Normative format and exact merge/replace/folder/security/identity policies:
[portable-backup-format.md](../../portable-backup-format.md). No format relies on
the local DEK, original passphrase, Windows account SID or device key. All APIs are
synchronous for future execution on the sole mutable-owner worker. Tasks12/13
still own passphrase confirmation, native pickers, user review/confirmation,
worker/session wiring and cached UI invalidation.

Only generated disposable temporary directories and published RFC seeds were
used. There were no real/default/reference vault, backup, credential, clipboard
or production application reads. Tests remove owned temporary directories through
their scoped tempfile owners. No test backup or secret fixture is committed.

Checks during implementation:

- `rtk cargo test --locked backup:: --lib`: initial2 passed; expanded8 passed;
  expanded10 passed plus1 ignored subprocess entry point (actually exercised by
  its parent). Tests use the real validated 64MiB/three-round Argon2id floor.
- `rtk cargo clippy --locked --all-targets --all-features -- -D warnings`: passed
  before and after replacing the quadratic duplicate scan with ordered indices.
- `rtk cargo test --locked --all-targets --all-features`: first full run114 passed,
  four ignored direct subprocess helpers,14 suites/74.55s. The fourth helper is
  the new portable process-interruption probe and is exercised by its parent.
- Intermediate final indexed full run115 passed/four ignored/14 suites/74.31s;
  authorized local cleanup repair and grouped dense-preview run116 passed/four
  ignored/14 suites/72.11s. These prompted changes were followed by the final
  folder reuse/idempotence revision; definitive final results follow below.

Definitive final validation, after folder reuse/copy/trash/idempotence and local
writer ownership fixes:

- `rtk cargo test --locked --all-targets --all-features`: **117 passed**, four
  directly ignored subprocess helpers exercised by parents,14 suites/71.85s.
- `rtk cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
- `rtk cargo fmt --all --check` and `rtk git diff --check`: passed.
- `rtk cargo build --locked --all-targets --all-features`: failed with MSVC
  LNK1104 opening `target/debug/examples/vault_lifecycle.exe`. The parent had
  already documented an unrelated running lifecycle preview; the author did not
  stop or inspect it. The observed build error is the target-file open failure;
  no stronger diagnosis is claimed. All-target tests/Clippy above passed.
- `rtk cargo build --locked --bin Authenticator --lib --all-features`: passed.
  No production application was launched. The all-target standalone build remains
  an explicit environmental gate until the owner releases that example artifact.

Owned cleanup: no preview process, UI profile, screen capture, helper script or
temporary fixture was retained. Scoped temporary directories remove the generated
local stores, portable backups and interruption leftovers. The unrelated lifecycle
preview was left alone. Parent-owned task/checkpoint edits were not changed by
this child.

New meaningful tests cover independently keyed/repassphrased destination local
storage and encrypted reopen; all24 OTP combinations; service/website/account
metadata; active/trashed folders and memberships; trashed account, recovery set
and individual code; used_at; IDs/timestamps; query/filter/sort/view preferences;
explicit security adoption and startup preservation. Two exports differ with
fresh independent randomness. Wrong passphrase, each header/authenticated region,
truncations, unknown version/KDF, length/trailing mismatch, schema/codec invalid
input and oversized regular files reject. Existing ciphertext is compared after
failed/stale/cancelled imports and failed/changed/cancelled exports.

Merge tests cover local/incoming duplicates, shared exact policy, same-ID and
cross-type collisions, KeepLocal bundle skip, KeepBoth graph remapping, used/trash
preservation, explicit folder reuse/copy/name decisions and both trash states,
repeated identical default Merge idempotence, invalid choices, retained
filters, explicit Replace fallback and removal counts, aggregate overflow, orphan
references, future-date logical edits and 10,000 unique indexed accounts.
Actual token-held commit tests prove nonblocking cancellation after admission
can complete whole import/export. Deterministic preexisting pending-name collision
proves cleanup never activates for an unowned file. Actual subprocess termination
before rename leaves the original backup unchanged and only authenticated
ciphertext in the leftover pending file; no plaintext sentinel appears.

The parent identified the temporary-path RAII-before-create ownership issue during
implementation; it was corrected before final validation, with the deterministic
regression above. The same historical Task03 local writer bug was then confirmed
and explicitly authorized for repair because backup import uses that writer.
Both guards now own/close their file handles before cleanup, activate only after
successful exclusive creation, and disarm after a consuming native move. The local
writer has its own deterministic preexisting pending-path preservation test; local
ACL and commit semantics are retained. This is not an independent final review.

Open genuine gates/limits: no separate Windows user/device, removable filesystem,
physical interruption/power-loss or final Tasks12/13 GUI recovery was available or
performed. The independent destination owner/passphrase/DEK roundtrip demonstrates
format independence without claiming a second Windows identity. Filesystem hostile
check-to-rename/directory-substitution races, full erase and instantaneous in-flight
operation cancellation are explicitly limited in the normative document. Final
conflict indices aggregate exact local/prior-incoming union counts and reasons per
incoming account, with shared-policy-verified representatives, supporting both
unique10,000 and dense10,000-by10,000 snapshots without enumerating pairs. Explicit
Replace of the dense snapshot is tested. Distinct reviewer must assess acceptance.

## Parent correction after independent review

Distinct review_11 found a P2 repeated-import defect when an incoming folder ID
collided with another local entity type: every default import remapped it again
and accumulated folders after account bundles were skipped. Parent added an
ephemeral borrowed exact-content folder index and deterministic lowest-ID reuse
only for an occupied original ID without a same-ID folder. Equal names alone
do not trigger this automatic reuse. Explicit KeepBoth still copies; KeepLocal
without an existing folder rejects. New summary flags identify global-ID collision
and content-equivalent reuse so Task13 can show the concrete resolution.

The regression covers the first remap, repeated codec-identical default import,
membership/recovery links under explicit account KeepBoth, deliberate folder
copies, deterministic reuse with multiple identical copies, invalid KeepLocal
and a changed-color folder that must remain separate. The initial focused test
passed1/1.00s; final checks and independent focused re-review follow in the parent
review report. No source provenance or hostile filesystem race guarantee is added.
