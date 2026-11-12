# Task11 independent security and data-loss review

Parent record, 13 November 2026. Distinct read-only `review_11` was not the author.
Original review requested a P2 merge correction. Focused re-review resolved it;
the portable backup engine is approved within the reviewed scope. Export/import
GUI and final package/offline recovery remain Tasks12/13/21/23, not established here.

## Finding and correction

**P2: repeated default Merge duplicated remapped folders.** An incoming folder ID
could equal an existing local account ID. The first merge assigned a fresh folder
ID; the next merge still searched for the original ID and created another folder.
An independent disposable probe produced2 then3 folders, with account bundles
skipped, contradicting the documented idempotence policy and risking collection
exhaustion after repeated imports.

Parent added a borrowed exact-content index for folders. If an incoming original
ID is occupied and no same-ID folder exists, default Merge reuses an exact name,
color, creation/update and complete trash-state/date match. Lowest stable ID breaks
ties. Same-ID reuse remains first, equal names alone do not automatically merge
ordinary distinct identities, and explicit KeepBoth creates a separate folder.
Pure cross-type collision now permits that explicit copy; KeepLocal without a
current same-ID/name folder rejects. Preview flags expose global-ID collision
and content-equivalent reuse. This is a documented content policy, not inferred
source provenance or a persistent external-ID alias.

The regression checks first remap, repeated codec-identical default import,
membership and retained recovery-used mapping with account KeepBoth, deliberate
folder copies, invalid KeepLocal, deterministic selection among identical copies
and changed metadata remaining separate. Focused independent re-review passed
**3 tests**: repeated cross-type collision, folder reuse/copy/trash, and conservative/
KeepBoth graph preservation. No new security or data-loss finding arose.

## Independent checks

| Check | Result |
|---|---|
| `rtk cargo test --locked --lib backup::` | 12 passed,1 direct helper ignored; interruption parent exercises it |
| `rtk cargo test --locked --lib storage::` | 21 passed,2 direct helpers ignored; parents exercise them |
| Changed timestamp integration test | 1 passed |
| Two tests in an independent external source copy | 2 passed: remapped-folder reproduction and timestamp invariant probe |
| Focused P2 re-review filters | 3 passed |

Storage checks include temporary ownership, actual process interruption,
cancellation/admission, native replace failure and ACL behavior. The external
timestamp probe confirmed that child/preference timestamps beyond the aggregate
are rejected and maximum logical timestamp edits remain usable; the suggested
future-child bypass was not present. The initial folder probe established the
P2 finding rather than being treated as a passing acceptance case.

Source review confirmed normative envelope offsets/AAD, independent backup
keys/randomness, bounded parsing and authentication failure, complete graph
references and recovery used/trash preservation, default local security/startup
preservation and pending cleanup only after successful exclusive creation.
No high-severity cryptographic or data-loss finding was established. This review
does not replace the final Task20 security audit or cryptographic certification.

Both portable and local writers were reviewed after the parent-identified
guard-before-create bug was fixed. Guards own/close handles, cannot unlink a
pre-existing failed-create path and disarm after a consuming native move. The
historical local writer correction preserves its ACL/commit semantics.

## Final parent validation and limits

After the P2 correction, `rtk cargo test --locked --all-targets --all-features`:
**118 passed,4 directly ignored subprocess helper entrypoints exercised by their
parents,14 suites,75.08s**. Strict all-target/all-feature Clippy, formatting and
diff checks passed. `rtk cargo build --locked --bin Authenticator --lib --all-features`
passed. These final tests include the explicit-copy and multiple-match additions.

The author standalone all-target build encountered MSVC LNK1104 opening the
existing `target/debug/examples/vault_lifecycle.exe` artifact. The unrelated
preview was untouched; no successful standalone all-target build is claimed.
All-target tests and linting passed. Task23 can use an isolated packaging output
directory for its complete package checks.

Clean independently keyed/passphrased local destination roundtrip is evidenced;
no actual second Windows user/device, removable filesystem, physical power loss
or GUI/session recovery was exercised. Task11 specifies separate context where
available; no second identity/device result is fabricated. Explicitly documented
hostile-directory/check-to-rename races are not atomic filesystem compare-and-swap
guarantees. No plaintext export, complete memory erasure or secure SSD deletion
claim is added.

The review changed no repository files/status or clipboard and opened no real/
default/reference data or production application. The independent source copy
`C:/Users/root/AppData/Local/Temp/auth-review11-584a62bb` was removed after exact
owned-path verification, with absence confirmed. Focused re-review created no
new fixtures; no retained probe/helper processes. Commit hash follows in the
parent checkpoint to avoid a circular self-reference.
