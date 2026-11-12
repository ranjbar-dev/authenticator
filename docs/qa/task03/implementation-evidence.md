# Task 03 implementation evidence

12 November 2026. Ready for a distinct focused security/data-loss review; not self-certified. Task/status/coverage acceptance changes and commits belong to the parent. Task 01/02 dependency implementations were present and their 20 existing tests remain passing. No real vault or user AppData directory was opened. No UI work or lifecycle integration is included here.

## Changes

- `src/storage/mod.rs`: locked-by-default repository, scoped session, owned zeroizing passphrase input, creation/unlock/lock, immutable read, detached validated transactions, atomic commit and DEK-only passphrase rewrap. All writes require exclusive mutable ownership and a lifetime OS instance lock.
- `src/storage/crypto.rs`: fixed authenticated envelope, OS CSPRNG, pinned current Argon2id/XChaCha primitives, calibrated stored bounded KDF parameters, explicit zeroizing KDF scratch and safe fixed errors.
- `src/storage/codec.rs`: explicit complete binary schema for all domain records/preferences with byte/count limits before copies/loop growth, exact enum handling and whole-aggregate validation. No serde or plaintext file serialization.
- `src/storage/files.rs`: known-folder LocalAppData resolution, current-user protected DACLs, bounded reads, generic filenames, ciphertext-only same-directory temp/sync/native rename, reparse-point endpoint checks and test-only failure injection.
- `src/storage/tests.rs`: twelve storage acceptance tests plus two subprocess helpers. Constants are disposable/published fixtures; every generated file is under tempfile ownership.
- `src/domain/records.rs`: zeroize account metadata/folder strings on drop, preserving task 02's validation/identity API.
- `src/error.rs`, Cargo.toml/lock and .gitignore: closed storage errors, pinned dependencies/native features and ignored concrete vault/temp/lock names.
- `docs/storage-format.md`, architecture.md and security-design.md: exact offsets/AAD/schema, limits, calibration, commit point, memory/ACL/filesystem limits and remaining task boundaries.

## Exact final verification

Windows x64 MSVC, `rustc 1.98.0 (88d9e12ae 2026-08-18)`, development/test profile **unoptimized + debuginfo**. Reused task 02's isolated build cache; no app process was launched and no user vault was involved.

1. `rtk proxy cargo test --all-targets --target-dir target/task02-checks -- --test-threads=2 --nocapture` — exit0. Library17 passed/0failed/2ignored, foundation3 passed, domain12 passed; binary/example0 tests. **32 main-process tests pass**. Library elapsed63.63s after3.79s compilation. The concurrent-instance helper additionally passed1 test in a child. The interrupted-writer helper deliberately exited73, as asserted by its parent. The two helpers are ignored in normal discovery because they require a parent-supplied disposable directory; both are actually invoked by passing acceptance tests. RTK's aggregated display can count the helper and show33 rather than32.
2. `rtk cargo fmt --all -- --check` — exit0, no differences.
3. `rtk cargo clippy --all-targets --target-dir target/task02-checks -- -D warnings` — exit0, no issues.
4. `rtk git diff --check` — exit0, no whitespace errors.

The final expanded tamper matrix includes magic/envelope/KDF version, iteration/lane fields, salt, wrapping nonce/key, payload nonce/length/ciphertext. Wrong passphrase and representative byte flips run real AEAD authentication; structural/KDF-limit failures occur before expensive derivation. The maximum-size-file case uses a sparse isolated file. At every failure the original test bytes remain exact, no unlocked session is installed, and original ciphertext can still unlock when restored by the test harness.

## Acceptance mapping

| Requirement | Evidence |
|---|---|
| Restart every entity/preferences, no plaintext disk artifacts | `restart_preserves_entire_aggregate_without_plaintext_artifacts`: compares complete encoded aggregate including IDs, metadata, folder membership, account trash, used recovery code, non-default preferences and timestamps after a committed change/reopen. Scans vault/temp files for service/user/folder/recovery/passphrase/Base32 and decoded seed markers. Empty mandatory-locked lockfile verified by length. |
| Wrong passphrase/tampering/truncation/oversized KDF or file | `wrong_passphrase_tamper_truncate_and_oversize_never_mutate`; each failed unlock retains exact bytes and remains locked. |
| Bounded malformed schema | `malicious_lengths_enums_versions_and_noncanonical_dtos_are_bounded`: every truncation point of the whole fixture, trailing bytes, malicious count, unsupported schema and KDF policy bounds. Reader code checks field maxima before copying; whole vault validates after decode. |
| Future schema/failed migration candidate/existing corrupt file | `authenticated_future_schema_and_failed_migration_preserve_original` and `external_file_changes_and_existing_corruption_are_never_overwritten`: a real authenticated future-schema ciphertext fails without writes; invalid schema transaction preserves old memory/file; existing corrupt/future file blocks create. Schema1 has no supported older release to migrate. |
| Failed/interrupted writes and rollback | `failed_cancelled_invalid_and_panicking_transactions_are_rollback_safe`: partial write, flushed pre-rename failure, explicit cancellation, invalid graph, panic; exact previous memory/file and normal temp cleanup. `native_windows_replace_failure_preserves_memory_and_file`: real Windows sharing violation, then successful retry after releasing handle. `terminated_writer_keeps_last_commit_and_leaves_only_ciphertext`: actual child-process exit after sync before rename, OS releases lock, previous vault reopens, encrypted orphan contains no sentinels. |
| Concurrent-instance protection | `concurrent_instances_are_rejected_in_process_and_across_processes`: separate repository and child process refused while lifetime lock held; next instance succeeds after drop. |
| Key rewrap and lock cleanup | `rewrap_changes_only_wrapping_and_failure_preserves_old_unlock`: failed rewrap retains old unlock, successful rewrap changes bytes0..116 only and preserves complete nonce/payload tail, old passphrase fails/new succeeds, lock removes session/snapshot and blocks reads. Zeroizing owners/scratch and metadata Drop verified by source, with no claim of complete memory erasure. |
| Current-user ACL | `windows_dacl_is_protected_and_only_current_user_has_access`: reads native directory/vault/lock descriptors, verifies protected DACL with exactly one allow ACE, full file access and SID equal to the process-token user. |
| Calibration/randomness | `calibration_and_randomness_use_bounded_production_policy`: final debug probe measured **1,990 ms**, selecting **65,536 KiB / 3 iterations / 1 lane**; floor retained. Random outputs differ and production implementation uses getrandom/ProcessPrng, not a seeded test PRNG. |

## Corrections during implementation

The first storage run had8pass/2fail because its artifact scanner attempted to read Windows' mandatory-locked empty `session.lock`. The scanner now verifies that file is empty and scans every data/temp file; the product lock behavior was correct. Added native ACL/sharing tests initially had a Windows constant/name and unnecessary-cast compile/lint issue, corrected before final checks. One exploratory relink returned LNK1104 while this same test executable was still running; final checks ran after the prior process exited. The expected disposable panic text in `--nocapture` is from a caught rollback test, not a product logger or unhandled failure.

Inspection of Argon2 0.6's allocating convenience scratch revealed its Blocks Drop deallocates without wiping; the implementation therefore uses a bounded caller-owned Zeroizing<Vec<Block>> with `hash_password_into_with_memory`. XChaCha's zeroize feature is explicitly enabled. Serialization reserves its fixed cap plus tag space before writing plaintext, avoiding abandoned plaintext on buffer growth.

## Review focus and honest limits

Review the binary offset/length/AAD policy, rewrap binding, untrusted allocation bounds, complete restoration validation, Windows DACL/unsafe-call cleanup, lock lifetime and exact native rename commit point. No fallible action follows rename success. Backup key/format independence is specified but backup engine remains task11. Schema1 rejects unknown schemas; there is no fictitious legacy transform. OS/idle/suspend/UI locking remains task05. Optimized release KDF timing and full clean-profile recovery remain packaging/backup evidence. Debug timing is not a release performance claim.

Tests cover process interruption and real sharing failure; they do not certify physical power-loss durability or filesystem/filter-driver behavior. Same-user compromise, administrator takeover, pagefile/hibernation/dumps, register/compiler/dependency/caller copies and old encrypted copies are outside guaranteed clearing/protection. No secure deletion or anti-replay counter is promised. The repository's ciphertext equality check protects against unexpected external edits while unlocked, not a malicious trusted-process race. A process crash may leave an encrypted orphan; it is never silently adopted or treated as a recovery source.

## Parent acceptance and independent review

Separate review_03 independently passed locked all-target tests (32main; helpers actually invoked), fmt, locked Clippy and whitespace checks and inspected primitives/format/codec/ACL/transactions. Sole P3 finding: SHGetKnownFolderPath failure needed CoTaskMemFree even on failure. Parent corrected the failure branch, reran fmt/locked Clippy/diff checks (pass); review_03 independently rechecked and approved task03 scope. No full KDF rerun for cleanup-only correction. This is an agent source/test review, not external cryptographic certification. Implementation SHA follows in next checkpoint.
