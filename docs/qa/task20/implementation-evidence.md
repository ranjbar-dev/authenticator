# Task20 implementation and audit evidence

13 November 2026, sole `implement_20`. Source and tests are ready for distinct review, not self-accepted. Baseline `6264c54d438f0dfe546a4e2b761b77f1ea5a2aed` was clean at handoff. Parent alone edits task/coverage/index statuses and commits. No child delegation, staging, commits or pushes.

## Scope and source changes

Read full Task20 and relevant PRD7–12, checkpoint/coverage and existing security/format evidence. Focused source audit inspected local crypto/codec/files/transactions, portable envelope/preview/merge/writer, strict OTP/URI/QR boundaries, lifecycle/capture publication and selected-only capture path, clipboard actor/native ownership, startup boundary, Root lock/review/worker adoption and diagnostics. No claim of exhaustive formal verification or external cryptographic certification. Report: [security-review.md](../../security-review.md).

Changed paths:

- `src/main.rs`: debug-only legacy framework-spike application route; production panic-hook helper and an actual subprocess suppression test, with a deliberately panicking ignored helper reached by its parent.
- `src/storage/{files,test_gate,tests}.rs`, `src/backup/{files,tests}.rs`: explicit test-only StorageFull writer after a bounded real ciphertext prefix; existing local rollback/portable overwrite oracles include it. No production writer/commit semantics changed.
- `src/domain/otp.rs`: bounded Base32 normalized buffer reserves before filtered extension.
- `src/platform/clipboard.rs`: bounded UTF-16 owner reserves before conversion; existing sequence/HWND/marker/digest checks unchanged.
- `src/platform/recovery_file.rs`: fallible MAX+1 allocation, initialized fixed buffer, bounded reads into remaining slice, no growth or read_to_end EOF allocation assumption; exact bytes/UTF-8/limit/path behavior retained.
- `examples/security_audit.rs`: generated published-vector QR and disposable isolated CLI core probe; no GUI/default vault or real data.
- `docs/security-review.md`, `docs/security-design.md`, this evidence and two sanitized dependency JSON artifacts. Existing design notes are marked historical and the current screenshot-batch versus portable Replace policy clarified.

Three project-owned growth sites could abandon earlier plaintext allocations before the zeroizing owner dropped. The new source reduces those avoidable copies, without promising complete erasure of opaque framework/dependency/compiler/allocator/OS copies. Legacy demo release reachability was hardening, not a discovered vault bypass. No confirmed unresolved material crypto/data-loss defect was identified in this focused author audit; independent review and mandatory native/manual gates remain required.

## Commands and test ownership

All shell commands start RTK. Fresh ignored output `target/task20-isolated`; old retained root/task18 library images and unrelated lifecycle preview were not overwritten or touched. `rtk cargo test --locked --all-targets --no-run --target-dir target/task20-isolated` passed; it compiled library209 entries plus main2 entries (one parent test/one directly ignored panic helper). This was compilation, not a full-suite test run.

Actual tests used `rtk proxy python -X utf8 target/task20-run.py <lib|main|domain> <filter> [--exact] --nocapture --test-threads=1`. Wrapper created fresh known TEMP/TMP under `target/task20-runtime-<UUID>`, recorded exact executable/PID/arguments/start/root, wrote stdout directly to file and used90s bounds. No run timed out or required termination. A sequential18-case harness was `rtk proxy python -X utf8 target/task20-checks.py`; it stopped on failure and completed successfully.

Executables: `target/task20-isolated/debug/deps/authenticator-8fbe62e92022c983.exe`, `Authenticator-92fe69a4fec7f682.exe`, `totp_domain-460af3df6337a9e9.exe`. Parent tests wait for their own helper processes; helper PIDs are not separately persisted. Final fresh executable-path inventory must find no remaining Task20 test/probe process before cleanup. No broad process-name termination or user Temp scan.

## Passing focused checks

| Filter (full prefix where relevant) | Result and scope |
| --- | --- |
| main `tests::production_panic_hook_does_not_emit_payload_in_child_output` | Final1 passed0.01s. Child attests it reached the installed hook, exits101 from the actual test panic, disposable payload absent from stdout/stderr. One ignored helper is exercised by this parent. |
| `storage::tests::failed_cancelled_invalid_and_panicking_transactions_are_rollback_safe` |1 passed8.09s, now including explicit StorageFull partial file write; unchanged encrypted file/in-memory aggregate, cleanup, invalid/cancel/panic rollback and later valid transaction. The test harness's expected public cancellation panic output is not a production app log. |
| `backup::tests::destination_consent_faults_and_cancellation_preserve_prior_ciphertext` |1 passed2.22s, now explicit StorageFull plus partial/pre-rename/stale overwrite/cancel/unowned pending collision. |
| `storage::tests::restart_preserves_entire_aggregate_without_plaintext_artifacts` |1 passed6.09s; encrypted reopen/full codec equality and own-directory sentinel scan. |
| `storage::tests::wrong_passphrase_tamper_truncate_and_oversize_never_mutate` |1 passed14.44s; actual KDF/authentication, meaningful header/wrapped-key/nonce/payload flips, lengths/caps, retained file and eventual valid reopen. |
| `storage::tests::malicious_lengths_enums_versions_and_noncanonical_dtos_are_bounded` |1 passed0.60s; every byte truncation of a complete aggregate, trailing/count/schema/KDF invalid cases. |
| `storage::tests::authenticated_future_schema_and_failed_migration_preserve_original` |1 passed5.28s; actual authenticated future payload and failed candidate do not replace prior data. |
| `storage::tests::schema1_frozen_public_fixture_migrates_without_unlock_write_then_atomic_schema2` |1 passed8.03s; lazy migration and failed/successful commit behavior. |
| `storage::tests::concurrent_instances_are_rejected_in_process_and_across_processes` |1 passed2.21s; own subprocess lock probe exercised. |
| `storage::tests::terminated_writer_keeps_last_commit_and_leaves_only_ciphertext` |1 passed6.35s; child exits73 before rename, prior file readable, generic ciphertext orphan only. |
| `storage::tests::native_windows_replace_failure_preserves_memory_and_file` |1 passed8.30s; real Windows sharing denial. |
| `storage::tests::windows_dacl_is_protected_and_only_current_user_has_access` |1 passed4.07s; native current-token SID/protected DACL checks. |
| `backup::tests::interrupted_process_leaves_only_ciphertext_and_preserves_backup` |1 passed10.13s; actual child exits74, prior backup survives, encrypted pending bytes decrypt as expected. |
| `backup::tests::stale_source_revision_cancel_and_failed_import_never_mutate` |1 passed19.94s; exact revision/source binding, early cancellation and encrypted import write failure. |
| `backup::tests::portable_envelope_roundtrip_and_randomization` |1 passed28.06s; independent randomized portable envelopes, wrong phrase/tamper/parameter limits and complete restore. |
| `platform::clipboard::tests` | Final5 passed3.71s after UTF-16 allocation hardening: isolated native station process, actor expiry/hidden/lifecycle/shutdown/newer identical external copies, busy retry, held admission/Lock and queue/code-window/clock cases. Three cases also passed individually in the initial18-case batch; not double-counted below. No WinSta0/user clipboard access. |
| `qr::tests` |15 passed2.14s; strict payloads, hostile dimensions/segments/ancillary chunks, deterministic adversarial bytes/noise, file preservation, format/rotation/alpha/parameters/count caps and redacted Debug/errors. |
| `platform::capture::tests` |6 passed0.17s; pure geometry/overflow/cancellation/queued-publication/idle/epoch checks, no physical capture. |
| `ui::state::tests::cancellation_does_not_wait_for_completion_publication` |1 passed0.00s; cancellation does not block on held completion publication. |
| `platform::recovery_file::tests` | Final3 passed0.06s after fixed-buffer reader: UTF-8/BOM/source unchanged, invalid/size/nonfile errors, links/ancestor/ADS/device/traversal refusal. Earlier reserve-only version3 also passed but is not the final read implementation. |
| integration `totp_domain` |12 passed0.02s after Base32 hardening: RFC6238/RFC4226 vectors, all24 parameters/boundaries, strict Base32/URI/validation and sensitive Debug redaction. |

These are57 unique focused ordinary cases across the task, with affected cases rerun after later allocation/panic-test changes. Repeated runs are not added to that total. Five subprocess helper entrypoints are exercised by their parents (four existing library helpers and the new main panic helper), not silently skipped acceptance. No current-source full-suite result is claimed. Parent's hosted success for6264c54 predates Task20 source.

## Disposable runtime/artifact/network probe

Built with `rtk cargo build --locked --example security_audit --target-dir target/task20-isolated`; executed through `rtk proxy python -X utf8 target/task20-probe.py`. The example has no path override or default-vault access: it owns a TempDir and generated public data. It performs in-memory QR decode, explicit encrypted account+folder+used-recovery save, encrypted export, independently created destination, reviewed engine import, encrypted reopen, published RFC code comparison and own-tree plaintext sentinel scan. No source QR pixels/payload file or raw recovery source is written. Only a fixed result line is printed.

Both runs exited0. Initial own PID25636:66 successful network samples, maxTCP0/maxUDP0. Final-source own PID3876:61 successful samples, maxTCP0/maxUDP0; current QR/OTP/used-state/artifact assertions passed. `netstat -ano` output was held only in process memory and filtered to that exact owned PID; only counts/tool status were retained, never other endpoints. Adapters remained enabled and sampling can miss transients. This is limited runtime observation, **not** a network-disabled-machine, actual screenshot or native GUI journey.

Core probe prints/logs and generated-artifact scans contain no sentinel values. Existing tests/fixtures intentionally contain published or generated values. Targeted tracked-source token/private-key pattern scan returned no matches (rg exit1); tracked secret-shaped/binary path inventory returned empty. This is a scoped heuristic plus source-fixture inspection, not proof that arbitrary secret encodings or every existing raster artifact are clean.

## Dependency and license evidence

`rtk cargo audit --version`: cargo-audit0.22.2. `rtk proxy cargo audit --json` exited0, no ignores:381 locked dependencies,1490 advisories, database commit `af7b3ec33460b8e2de8d422e16d628518891a519`, updated2026-11-12T15:21:59+01:00,0 known vulnerabilities. Raw safe output remains ignored under target; sanitized [summary](dependency-audit.json) records exact lock SHA/database and three informational unmaintained notices.

`rtk cargo tree --locked --target all -e normal -i zbus_names` traces the Unix AT-SPI/accesskit_unix chain. The corresponding Windows normal tree prints nothing. Filtered metadata traversal independently confirms zbus_names/zvariant/zvariant_derive absent, along with reqwest/webbrowser. No notices were suppressed and no unsafe forced transitive upgrade was made. Public primary RustSec pages were checked; docs.rs pinned pages were inaccessible through the web tool, so local pinned RustCrypto source/Cargo metadata and existing implementation sources were inspected instead. No external audit of the current crate version is claimed.

`rtk cargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc`, traversing only normal/build edges, yields185 packages including self, none missing license metadata; [inventory](windows-dependencies.json). Reviewed combined declarations and actual font notice files (epaint_default_fonts fonts/OFL.txt, UFL.txt, Hack/emoji notice texts; project Noto OFL). Task23 must package applicable texts/notices. Cargo-deny was unavailable; no license-policy pass is attributed to it. See the current report for AND/OR/legacy-expression and package-notice limits.

## Final static/build and open gates

- `rtk cargo fmt --all -- --check`: final pass. The newly added example initially needed formatting; `rtk cargo fmt --all` corrected it. No source behavior was changed by formatting.
- `rtk cargo clippy --locked --all-targets --all-features -- -D warnings`: pass, no allowances.
- `rtk cargo check --locked --release --bin Authenticator --target-dir target/task20-isolated`: final pass3.93s after production buffer changes; release entry configuration type-checks without the debug route. Not a release package or linked-binary inspection.
- `rtk cargo build --locked --all-features --bin Authenticator --lib --example security_audit`: pass11.78s; no application was launched.
- `rtk proxy git diff --check`: final result recorded at handoff.

Native screenshots/input:0. No unchanged GetCursorPos-denied launch, security/privacy/network-settings change, actual session lock/suspend/logon, production registration, user clipboard or real/default/reference vault action occurred. Actual interactive Copy→Changed cause remains unknown. Offline clean-install/full native/physical/accessibility/clipboard/package gates remain OPEN; independent review of this audit/fixes is still required. OS dump/memory/history/secure-erasure/hostile-filesystem limits are explicit in the report.

Cleanup: all28 recorded wrapper PIDs were freshly absent. Native PowerShell verified every exact root's manifest/executable and resolved containment beneath the non-link workspace target, rejected linked roots or remaining fixture directories, archived only safe logs/process/network summaries to ignored `target/task20-evidence/<runtime-UUID>/`, then removed28 exact runtime roots. No generated vault/backup/raw source was archived. The exact removed-name list is `target/task20-evidence/cleanup.json`. Helper processes were waited by their parents; no repeated kill, resume, elevation, broad Temp scan or speculative cleanup occurred. No task20 test/probe is active. Known old30688/32032 and unrelated lifecycle preview remain untouched. No task status/PRD scope/coverage/parent checkpoint was changed by this child.
