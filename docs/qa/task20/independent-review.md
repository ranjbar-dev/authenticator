# Task20 independent security and data-loss review

13 November 2026, distinct `review_20`, after implementation FINAL. Main source, task statuses, PRD, staging, commits and pushes remained unchanged. This report is the only tracked file written; bounded scripts/logs were kept under ignored target.

**Result: limited source approval; no material security or data-loss defect found in the reviewed scope.** One minor command-transcription correction is noted below. This is an agent source/test review, not external cryptographic certification or full Task20 acceptance. New-source hosted checks and the existing native/offline/interactive-clipboard gates remain open.

## Source assessment

Read the complete Task20 contract, relevant PRD FR6–8/sections7–12, author audit/evidence, dependency artifacts, full changed-source diff and selected underlying storage, backup, parser, capture, clipboard and worker boundaries.

The production changes are scoped correctly. The legacy gallery route is guarded by `cfg(debug_assertions)` at both argument selection and application construction, so release configuration has no such entry branch. The panic-hook function used by main is the same function exercised by the subprocess test. Its child attests reaching the installed hook, then actually panics; the parent requires exit101 and absent payload in both output streams. The attestation is nonsensitive and test-only.

Base32 filtering reserves the bounded input size before extending; filtered bytes cannot exceed that capacity. Decoded partial output already belongs to a zeroizing buffer. UTF-16 conversion reserves UTF-8 byte length plus the terminator, a valid upper bound for UTF-16 units; allocation is fallible before clipboard mutation. The recovery reader fallibly allocates and initializes MAX+1 once, reads only into its remaining slice, retries Interrupted, rejects over-limit/invalid UTF-8 input and returns zeroizing text. It does not depend on `read_to_end` EOF allocation behavior. These improvements reduce avoidable old allocations without promising complete erasure of allocator, compiler, framework, dependency or OS copies.

StorageFull instruments are test-only and pass actual file handles through a bounded partial ciphertext writer. They return a real StorageFull error without filling the volume. They exercise the existing error/temporary ownership and rollback paths; production writer/cancellation semantics are unchanged. Existing pending guards activate only after exclusive creation, close handles before unlink and preserve unrelated preexisting names. No fallible action follows successful native commit before memory adoption.

Selected underlying review confirmed bounded KDF validation before allocation, Argon2id scratch/key ownership, independent OS randomness, authenticated key wrapping/payload boundaries, exact envelope lengths, local/portable domain separation, strict bounded schema/URI/QR parsing, selected-region GDI capture and zeroizing owned buffers. Local transactions validate detached candidates and check the committed ciphertext before replacement. Portable plans bind authenticated snapshots and source/local revisions, then apply through that transaction boundary. Generation/lifecycle/deadline checks and nonblocking cancellation prevent stale unlocked adoption; an already admitted complete operation can still finish, as documented.

Clipboard HWND/sequence/random-marker/content checks were retained. Publication and cleanup require ownership evidence; the UTF-16 change does not relax it. Native test children create a random private noninteractive station rather than touching WinSta0/user clipboard. This review does **not** explain or resolve the earlier real interactive Copy→Changed result. The startup public-command/encrypted-preference distinction, external filesystem/registry race limits, OS policy and lack of complete memory/secure-erasure guarantees are stated honestly.

The CLI probe owns its TempDir and published/generated fixtures, has no default-vault or caller path override, scans only its own encrypted artifacts, and prints a fixed result line. Its generated QR/pixels stay in memory. It checks a real encrypted local save, independent portable restore, used state and RFC value after reopen. Network sampling filters to its owned PID and retains counts only. It is correctly described as a limited observation with adapters enabled, not a disconnected-machine or native UI journey.

## Independently executed checks

Every shell command began with RTK. Existing final executables under `target/task20-isolated` were used without source changes or relinking. Commands were `rtk proxy python -X utf8 target/task20-run.py <main|lib|domain> <filter> [--exact] --nocapture --test-threads=1`, using a fresh known TEMP/TMP root, exact executable/PID/start/root manifest, direct-file logs and90s bounds. All exited0 without termination.

| Target/filter | Result |
| --- | --- |
| main `tests::production_panic_hook_does_not_emit_payload_in_child_output` |1 passed0.01s; actual attested panic helper exercised. |
| lib `platform::recovery_file::tests` |3 passed0.06s; final fixed-buffer behavior, invalid/size/path/source preservation. |
| integration `totp_domain` |12 passed0.01s; published RFC vectors/all supported parameters, strict Base32/URI and redaction. |
| lib `storage::tests::failed_cancelled_invalid_and_panicking_transactions_are_rollback_safe` |1 passed7.00s; explicit StorageFull plus unchanged committed memory/file, failure/cancel/panic cleanup and valid retry. Expected public test panic output is not production application logging. |
| lib `backup::tests::destination_consent_faults_and_cancellation_preserve_prior_ciphertext` |1 passed2.10s; explicit StorageFull, partial/pre-rename/stale overwrite and cleanup. |
| lib `platform::clipboard::tests` |5 passed3.73s; private native station helper, actor ownership/expiry/races/lifecycle/shutdown/queue checks. No user clipboard read/write. |
| lib `storage::tests::wrong_passphrase_tamper_truncate_and_oversize_never_mutate` |1 passed13.83s; actual authenticated/header/KDF limits and retained vault. |
| lib `backup::tests::stale_source_revision_cancel_and_failed_import_never_mutate` |1 passed19.40s; source/local revision, cancellation and failed encrypted import. |

Total25 independently passing ordinary focused cases; two subprocess helper entrypoints were exercised by these parents. This is not the author's57-case set or a full suite. Other author checks are attributed in implementation-evidence.md rather than recounted as independent execution.

`rtk proxy python -X utf8 target/task20-probe.py` independently passed the core/artifact probe. Owned PID39688 produced62 successful own-PID network samples with maxTCP0/maxUDP0. Adapters remained enabled; short connections can escape sampling. No other process endpoints were emitted or retained. The probe and source inspection do not close the required network-disabled clean-install/native journey.

`rtk proxy git diff --check` passed, with ordinary LF/CRLF notices. Author formatting, lint, release check and build results were inspected but not repeated here; their exact scope remains attributed.

## Dependencies, licenses and minor evidence correction

Independent `cargo audit --json` execution through an RTK-prefixed bounded Python wrapper exited0, with0 known vulnerability matches, the same database commit `af7b3ec33460b8e2de8d422e16d628518891a519`, and the same three unsuppressed maintenance notices. Primary RustSec pages independently confirm informational unmaintained status: [zvariant](https://rustsec.org/advisories/RUSTSEC-2026-0370.html), [zbus_names](https://rustsec.org/advisories/RUSTSEC-2026-0371.html), [zvariant_derive](https://rustsec.org/advisories/RUSTSEC-2026-0372.html). This time-specific result is not proof of vulnerability absence.

Independent Windows-filtered metadata traversal over normal/build edges exactly matched all185 name/version/license inventory entries, found no missing license declarations, and excluded all three warning packages. Combined declarations were preserved: dpi Apache-2.0 AND MIT; self_cell Apache-2.0 OR GPL-2.0-only; default fonts additionally OFL-1.1 AND Ubuntu-font-1.0. Metadata validation is not package license/notice compliance; actual distribution contents remain Task23's gate.

**Minor P3 evidence typo:** the author evidence transcribes `--format-version1`; the valid reproducible Cargo option is `--format-version 1`. The independent command actually used `cargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc` and reproduced the185-entry result. Parent should correct that transcription before committing. No production/source correction or renewed security test is needed for this documentation-only fix.

The current-tree heuristic key/token/path scan and historical raster sanitation were inspected as attributed scoped evidence. This review did not independently rescan every binary/raster or prove absence of all possible secret encodings. No real user credentials were accessed to test privacy.

## Remaining gates and cleanup

Native screenshots/input inspected:0. Actual interactive clipboard success/expiry/Lock/exit remains unresolved; no ownership checks were weakened or native success invented. Physical DPI, Narrator/high contrast, real session-lock/suspend/logon, disconnected-machine full journeys and clean-package recovery are still unverified. Existing reference-pixel limitations remain. This report cannot accept full Task20/21 or production readiness from source/test counts.

Complete protection from an unlocked compromised endpoint, administrator/process-memory access, clipboard history, pagefile/hibernation/OS dumps or previous backups is not established. Cancellation does not forcibly erase already executing OS/crypto work; atomic logical replacement does not establish physical power-loss durability or secure erasure. The source/report accurately retain those limits.

All nine owned wrapper PIDs37760/45476/44776/39472/47156/11480/39688/42508/9496 exited normally and were freshly absent before cleanup. A fresh inventory also found no active executable at the reviewed test/probe paths. Exact manifest/root/executable checks, non-link resolved containment beneath workspace target and absent fixture subdirectories preceded native PowerShell removal of the nine exact runtime roots. Only safe logs/manifests/network counts were archived under ignored `target/review20-evidence/task20-runtime-<UUID>`; no generated vault/backup was archived. Known old30688/32032 and unrelated lifecycle previews were untouched. No kill/resume/elevation, broad Temp scan, default/reference vault, user clipboard, real startup registration or OS security/privacy/network change occurred.

Scope estimate: approximately25 source/report files plus dependency metadata and primary advisory pages; selected security boundaries and changed code, not exhaustive formal verification or a whole-application visual audit. Parent owns statuses, integration, any minor documentation correction, new-source hosted validation and continuing21–24.
