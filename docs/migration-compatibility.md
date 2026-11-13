# Backup and migration compatibility

Discovery date: 13 November 2026. **The reference app's private `.vault` format
is unsupported in Authenticator v1.** No converter or interoperability claim is
shipped. This is the completed discovery result permitted by PRD FR-6.4/§12.3,
not a failure of native encrypted backup support.

## Evidence and scope

Read-only installed-package metadata identified
`FirstOrder.AI.AuthenticatorApp_3.2.0.0_x64__8vmr2j06510pw`, display name
“Authenticator App · 2FA”, publisher display name “Shanghai FirstOrder Technology
Co., Ltd.” Its public application manifest names `Authenticator.exe` and a
Windows full-trust entry point. No reference process was launched and no package
user-data directory, vault, credential, backup or account was opened.

The publisher's [official product page](https://authenticator.firstorder.ai/)
describes encrypted export, and links the Windows product to
[Microsoft Store product9n6gl0bvkphn](https://apps.microsoft.com/detail/9n6gl0bvkphn).
The [publisher-owned extension listing](https://chromewebstore.google.com/detail/authenticator-app/lgndjfkadlbpaifdpbbobdodbaiaiakb?hl=en)
also links that website and mentions import/export. These are product claims,
not an on-disk specification. In the inspected pages and bounded public searches
for the package identity, `.vault`, backup format and import/export documentation,
no byte-level envelope/schema/KDF specification or disposable interoperability
fixture was established. The Store page itself returned410 to the browsing tool;
the link's provenance is the official product page, not a successfully inspected
Store description. This is not a claim that no documentation exists anywhere.

No disposable reference export was already available in the repository's known
test/evidence paths. Real backups were not searched for or requested. Creating
an export from an unknown live reference vault would not be a disposable test.
No subscription, sync, sign-in or purchase was enabled. Installed program binaries
and private data were not decompiled or inspected to guess a format.

The remaining unknowns include its magic, versions, serialization, encryption,
KDF parameters, password semantics and preservation of record associations/used
states/Trash. A shared filename extension or marketing encryption claim cannot
resolve those unknowns. Future support requires lawful format evidence, generated
disposable exports, bounded parsing, interoperability/round-trip tests and an
independent security/data-loss review before any support claim changes.

## Supported inputs

| Input | Current result |
| --- | --- |
| Native Authenticator portable backup, `AUTHPBK1` envelope1 | Supported encrypted export/import with authentication, preview and explicit Merge/Replace. Recommended extension: `.authpbk`. |
| Local Authenticator vault, `AUTHLOC1` / `vault.bin` | Not a portable backup; rejected by Import backup. Use Export encrypted backup from the unlocked app. |
| Reference FirstOrder private `.vault` | Unsupported; format/interoperability unproven. |
| Other apps' backups or renamed files | No inferred compatibility. Recognition uses authenticated native structure, not the extension. Renaming does not convert data. |
| Supported `otpauth://totp` QR or manual credential | Supported by account entry, separate from whole-vault import. This does not migrate another app's folders, recovery history or Trash. |

Native backup contains accounts and OTP parameters, metadata, folders/memberships,
recovery-code used/Trash states and preferences. Its separate passphrase and
encryption are independent of the original local vault keys. Import retains local
security/display choices unless explicitly selected where supported, and never
silently enables startup. See [native format](portable-backup-format.md) and
[actual full-graph native recovery evidence](qa/task21/remaining-native-workflows.md).

## Safe rejection and recovery guidance

`backup::envelope::parse` rejects a non-native magic or unsupported envelope/KDF
version before decryption. Malformed/truncated inputs also fail closed. Preview
performs no vault transaction; apply is possible only after a valid authenticated
preview and explicit reviewed plan. The existing UI names the native `AUTHPBK1`
format, explicitly excludes local `AUTHLOC1` and private reference `.vault`, and
explains that renaming is not conversion. Wrong phrase/corruption clears the
phrase and offers another backup without treating failure as an empty vault.

To transfer this app's data, use Settings, Export & Import, Export encrypted
backup; restore it through Import backup with the separate backup passphrase.
For an individual service credential, supported service-provided TOTP QR/image
or manual input is an alternative when legitimately available. Keep the original
reference data and its recovery methods; this app cannot restore an unsupported
private backup or recover a forgotten passphrase.

Current negative transactional tests and native positive recovery are recorded in
[Task22 evidence](qa/task22/implementation-evidence.md). No new parser or UI
behavior was necessary for this unsupported decision.
