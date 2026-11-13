# Task22 implementation evidence

13 November 2026, implementer `implement22_discovery`. Ready for independent
security/data-loss review. No task status, PRD scope, commit or push changed.

## Result

Completed bounded reference-format discovery: FirstOrder private `.vault` import
is unsupported in v1. No proven format specification or already available
disposable reference export was established. No converter, speculative parser,
runtime network path or UI change was added. Existing native backup behavior and
explicit unsupported-format guidance already meet this boundary.

Changed paths:

- `docs/migration-compatibility.md`: decision, inspected evidence, support table,
  safe native-backup path and proof required for future support.
- `docs/reference-ui.md`: newly discovered official marketing-image links and
  platform/version/unrendered limitations, preserving historical local evidence.
- `docs/architecture.md`: concrete format boundary.
- `README.md`: concise supported native/unsupported reference formats.
- This evidence file.

The parent's unrelated checkpoint edit in `tasks/README.md` is not this child's
change. No production source, tests, dependencies, backups or screenshots changed.

## Actual discovery

Read-only `Get-AppxPackage -Name FirstOrder.AI.AuthenticatorApp` and
`Get-AppxPackageManifest -Package <returned PackageFullName>` returned:

- Identity `FirstOrder.AI.AuthenticatorApp_3.2.0.0_x64__8vmr2j06510pw`.
- Display name Authenticator App · 2FA; publisher display name Shanghai FirstOrder
  Technology Co., Ltd.; entry executable Authenticator.exe, Windows full-trust.

Only installed program metadata was read. No reference launch, package user data,
real backup, vault, sign-in, sync, purchase or credential was accessed.

Primary public evidence inspected:

1. [Official publisher product page](https://authenticator.firstorder.ai/), opened
   13November2026: product-level encrypted-export statements, not a byte-level
   file-format contract. Its HTML links Windows Store product9n6gl0bvkphn.
2. [Publisher-owned Chrome listing](https://chromewebstore.google.com/detail/authenticator-app/lgndjfkadlbpaifdpbbobdodbaiaiakb?hl=en): links that official website and mentions import/export, without the required format details.
3. [Microsoft Store link](https://apps.microsoft.com/detail/9n6gl0bvkphn): browsing
   returned410; no Store-description or format claim is based on that response.

Bounded search queries included exact package identity, FirstOrder export/vault
format, and `site:authenticator.firstorder.ai` with `.vault`, backup format and
import/export documentation. Results were the product page/listings and service
enrollment guides; no envelope/schema/KDF specification or disposable export was
established. This is a limitation of inspected evidence, not proof of worldwide
documentation absence. No user directories were searched for backups.

The official product HTML also linked four public marketing images. Their full
URLs and exact provenance are in `docs/reference-ui.md`; this child did not
download/render them or assert their platform/version/fidelity. Parent was
notified so later visual work can inspect them without opening a real vault.

## Source and meaningful checks

Inspected `src/backup/envelope.rs`, `src/backup/mod.rs`, relevant real engine
tests, and `src/ui/backup_import.rs`/`backup_import_tests.rs`:

- Parsing checks exact `AUTHPBK1`, envelope and KDF versions before decryption;
  extension is not recognition. `AUTHLOC1` and unknown reference-like bytes reject.
- Preview authenticates/validates before creating any applicable plan and performs
  no vault transaction. UI Unsupported text explicitly identifies native format,
  excludes local/reference files and warns that renaming is not conversion.
- The existing negative worker/UI case compares destination ciphertext after
  each failure, requires no preview/plan, clears the phrase and keeps the current
  session unlocked. This is real encrypted storage/worker evidence, not a test
  of an invented reference format.

Commands used fresh owned D: TEMP/TMP **only in the subprocess**, avoiding the
known C: linker-scratch shortage. Canonical Cargo commands below were invoked
through `rtk proxy powershell -NoProfile -Command` with those process-local vars:

| Command | Result |
| --- | --- |
| `rtk cargo test --offline --locked --lib ui::shell::backup_import_tests::backup_import_wrong_password_tamper_future_local_reference_and_oversize_preserve_session -- --exact --nocapture --test-threads=1` | 1passed,210filtered,21.06s; all six failure classes preserved current ciphertext/session |
| `rtk cargo test --offline --locked --lib backup::tests::independent_local_keys_all_entities_parameters_and_policies -- --exact --nocapture --test-threads=1` | 1passed,210filtered,14.53s; full native entities/24 OTP settings, independent keys and preference policy |
| `rtk git ls-files '*.vault' '*.authpbk' '*.backup'` | no tracked backup paths |
| `rtk git diff --check` | passed |

Scratch roots: `target/task22-temp-2b25a1c8-a286-4526-b17f-415e9f7ccccf` and
`target/task22-positive-df172957-aa4e-4771-b35d-0096f870351c`. No recursive cleanup
performed. No failed test invocation or zero-selection result occurred. A few
initial source-location searches named nonexistent guessed files; actual paths
were inventoried and read before conclusions.

## Review boundary

There is no converter requiring a new parser/security surface. The reviewer
should independently assess evidence/claim accuracy, safe rejection and native
guidance. The generated `case-*.vault` negative inputs are **not** real reference
exports or interoperability proof. Current native GUI full-graph recovery remains
attributed to Task21; this task did not repeat native UI or claim a full suite,
offline-machine, physical accessibility/DPI or precise reference visual pass.
