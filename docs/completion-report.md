# Authenticator completion audit — incomplete acceptance

The [14 November resumed execution](qa/resume-2026-11-14.md) freshly verified main/remote equality, successful CI43548413816, local package integrity, 17 packaging regressions and formatting. [Capability inspection](qa/task05/resume-2026-11-14.md) found Hyper-V available but no ready Windows guest/media; provisioning prerequisites and all existing external acceptance remain open. This update changes documentation only and closes no acceptance gate.

The implementation and all identified feasible software/native QA cases are independently reviewed. **16 numbered scopes are done; eight remain blocked by mandatory external acceptance.** Final source/CI/Git integration is verified in the [integration record](qa/task24/final-integration.md). The project is not declared fully complete. The [24 task files](../tasks/README.md), [48 coverage rows](../tasks/coverage.md) and [open checklist audit](qa/task24/task-checklist-audit.json) retain the original requirements and exact remaining items.

## Shipped behavior and reviewed repairs

The Rust Windows app implements local TOTP; selected-region, image and manual entry; concealed editable review and explicit encrypted Save; search, six sorts and list/grid; guarded clipboard leases; recovery codes with used/Undo; many-to-many folders; Trash/restore/confirmed permanent deletion; locking/settings/startup/help; and native portable encrypted export/import. AUTHLOC1 and AUTHPBK1 encrypt metadata and preferences. Defaults remain 300-second idle lock and 30-second clipboard expiry. Native backup keys are independent of local vault keys. Private reference `.vault` is unsupported after accepted discovery. No app sign-in, cloud storage/sync, translation infrastructure, telemetry or required runtime service was added.

The [follow-up review](qa/task24/follow-up/independent-review.md) approved page-entry and shared-focus repairs, deterministic packaging and exact-byte notice checkout. The original hosted failure was reproduced as a focused Review button extending 13 pixels below the viewport. Native testing found a second clipped initial Cancel action. Minimal reveal for three actual frames after gained/first-observed focus repairs both; wheel, pointer or lost focus cancels the temporary reveal. Original visibility assertions were retained. No crypto, storage transaction, capture backend or clipboard authority was weakened.

The [state re-review](qa/task24/state-gaps/independent-review.md) closed all five remaining feasible groups: distinct recovery list/chooser empty/no-match; invalid UTF-8 picker error preserving exact draft/preview; zero-active-code detail and restoration of IDs/value hashes/used history; folder no-match/Clear retaining membership; and readable empty Trash plus deliberate conflict consent/Cancel nonmutation. The only additional support is an owned QA example and appended debug-only digest getters, with no production shortcut/data-path override. Existing production prefixes are unchanged and a real release rebuild retained identical EXE bytes.

## Checks and native evidence

| Evidence | Actual result and scope |
|---|---|
| [Hosted 109171a](qa/task23/hosted-109171a.md) | Run 43536070275 passed fmt, strict all-target Clippy, 230 ordinary tests, Windows build, release package,17 packaging regressions, validation and artifact upload. Five direct helper entrypoints are exercised by parent subprocess tests; nested summaries are not double-counted. |
| [Post109171a additions](qa/task24/state-gaps/checks.json) | Formatting, strict example Clippy and the existing real-file retention regression passed, then independent re-review repeated the relevant checks. Committed source `079c08ddcda27c652f452e5b5756ed2500523fc9` passed [CI 43546047875](https://github.com/ranjbar-dev/authenticator/actions/runs/43546047875): full formatting/lint/tests/Windows build and release-package checks. [Final integration](qa/task24/final-integration.md) records Git equality and clean state. |
| [Reproducibility](qa/task24/follow-up/reproducibility.md) | Two complete builds from empty host/target release outputs each compiled 183 crates and matched EXE/ZIP bytes. This is fixed-toolchain, same-path/environment evidence, not universal cross-host identity. |
| [Final release equivalence](qa/task24/state-gaps/release-equivalence.json) | Own-source compile/link 91.979 seconds with the exact static-CRT /Brepro flags retained the native-tested EXE hash. No package command overwrote the clean local ZIP. |
| [Production runtime](qa/task24/follow-up/production-smoke.json) | Exact local EXE 9656… passed fresh setup, negative-monitor QR, concealed edited review, keyboard Save, exactly-one independent HMAC, real keyboard Copy, 406-byte encrypted export, differently keyed fresh-vault restore, restart Locked, correct unlock/current HMAC and Ctrl+L privacy. |
| [Independent native replay](qa/task24/follow-up/reviewer-evidence/native-result.json) | Actual native picker, authenticated 257-record review, pointer mode selection, whole focused Review/Apply/Cancel/Confirm, wheel-with-focus, Escape/Space cancellation and Ctrl+L passed with exact nonmutation. |

The same checker individually opened 85 of the original 195 images, all 59 follow-up author images and all 14 state additions: **158 of268 author images**, plus two separate independent dialog frames. The original 110 historical images remain visually unopened. Integrity/source attribution was checked separately; masked regions receive no readability pass. No all-image, all-state/physical-DPI Cartesian or exact-reference-pixel claim is made.

Primary physical 96/144/192 DPI capture and mixed primary 144/negative-left 96 crop are evidenced separately from application zoom. The measured mixed selected rectangle matched the independent QR bounds and decoded before explicit Save/HMAC. Earlier failed NoQR attempts remain historical failures without invented causes. Actual overlay Ctrl+L concealed private AX and preserved ciphertext. A configured 60-second idle overlay was active at 55.134seconds and Locked by 79.697seconds; this is the observed bound, not a fabricated 65-second endpoint. The encrypted 300-second default was restored.

## Identifiable packages

| Artifact | Source and byte identity |
|---|---|
| **Local native-tested EXE in current ZIP** [clean build](qa/task23/clean-build-079c08d.md) | `dist/Authenticator-0.1.0-windows-x86_64.zip`, 6,026,393 bytes; SHA256 `9c8b781018c3e6b66d06cbe3f5d5a5184b620f0d9925ce738e941b953badbcc6`. Metadata identifies committed `079c08ddcda27c652f452e5b5756ed2500523fc9`, source_dirty=false, epoch 1794623007. The contained EXE exactly matches native runtime evidence. |
| **Local EXE** | 12,179,456 bytes; SHA256 `9656ed317d6b2e36a96cf17ac5c0d2729680cb6abb850941c295f7418272978e`. Clean packaging and subsequent QA-only source rebuild exactly match the independently reviewed full-build/native-runtime bytes. |
| Current hosted artifact [record](qa/task23/hosted-079c08d.md) | Artifact14095690010, source079c08d/dirty=false; ZIP SHA `22b2c11944a2e33716559d8af51bf639be945bf4d451e68dd330a5caafe51103`, EXE SHA `8806765c4ada8d152b6dbe35d35befa69cd877c7bd355fafff76ff315a7beb3e`. API digest/checksum/paths/CRCs/manifest validated; no native runtime test. |
| Historical hostedA artifact [record](qa/task23/hosted-109171a.md) | Artifact 14091015742, expiry 28 November 2026. Download digest/checksum/paths/CRCs/manifest validated; source 109171a/dirty=false. ZIP SHA `e84e5fc2cbc652a5132385c28a473333ab571be6a5a6bbf826e4f6742dd1e1ba`; EXE SHA `8806765c4ada8d152b6dbe35d35befa69cd877c7bd355fafff76ff315a7beb3e`. This hosted EXE was not native-tested; its cross-host bytes are not substituted for local 9656… proof. |

The earlier clean109171a ZIPee7af10e… is preserved in `target/task24-preserved-clean-109171a`; its record is historical. The dirty-source ZIP60f9… belongs to the full-rebuild comparison, not the current dist file. Prior 99a2…/9165…/b7ac… binaries remain historical. Source timestamp means commit/source time, not actual build wall time. No GitHub Release was published. Post109171a QA-only additions preserve production EXE bytes. [Final source/CI/Git attribution](qa/task24/final-integration.md) records079c08d; the following documentation-only commit supplies its own SHA/CI result after push, avoiding a circular reference.

## All numbered outcomes

| Task | Status | Accepted scope or remaining boundary |
|---|---|---|
| [01](../tasks/01-foundation-and-decisions.md) | done | Accepted numbered scope; evidence in task file. |
| [02](../tasks/02-totp-domain-and-uri-parser.md) | done | Accepted numbered scope; evidence in task file. |
| [03](../tasks/03-encrypted-local-vault.md) | done | Accepted numbered scope; evidence in task file. |
| [04](../tasks/04-shared-ui-shell.md) | done | Accepted numbered scope; evidence in task file. |
| [05](../tasks/05-vault-setup-and-lock-screen.md) | blocked | isolated offline and actual Windows session-lock/suspend acceptance unavailable |
| [06](../tasks/06-local-qr-decoding.md) | done | Accepted numbered scope; evidence in task file. |
| [07](../tasks/07-windows-region-capture.md) | done | implementation and measured monitor/DPI categories independently reviewed |
| [08](../tasks/08-screenshot-review-and-import.md) | blocked | effective disconnected-machine screenshot workflow unavailable |
| [09](../tasks/09-manual-account-editor.md) | done | Accepted numbered scope; evidence in task file. |
| [10](../tasks/10-codes-list-and-clipboard.md) | blocked | actual hardware sleep/resume lifecycle acceptance unavailable |
| [11](../tasks/11-portable-backup-engine.md) | done - engine approved after distinct security/data-loss review and focused re-review | Accepted numbered scope; evidence in task file. |
| [12](../tasks/12-export-backup-screen.md) | done | numbered UI scope independently reviewed with native state evidence |
| [13](../tasks/13-import-backup-screen.md) | done | numbered UI scope independently reviewed with native state evidence |
| [14](../tasks/14-backup-codes-list.md) | done | numbered UI scope independently reviewed with native state evidence |
| [15](../tasks/15-backup-codes-editor.md) | done | numbered UI scope independently reviewed with native state evidence |
| [16](../tasks/16-backup-codes-detail.md) | done | numbered UI scope independently reviewed with native state evidence |
| [17](../tasks/17-folders-and-organization.md) | done | numbered UI scope independently reviewed with native state evidence |
| [18](../tasks/18-trash-and-restoration.md) | done | numbered UI scope independently reviewed with native state evidence |
| [19](../tasks/19-settings-startup-and-help.md) | blocked | offline/logon and heard screen-reader acceptance unavailable |
| [20](../tasks/20-reliability-and-security-review.md) | blocked | effective isolated offline runtime inspection unavailable |
| [21](../tasks/21-complete-ui-and-workflow-qa.md) | blocked | isolated runtime, OS lifecycle and heard-speech acceptance unavailable |
| [22](../tasks/22-reference-migration-discovery.md) | done - evidence-based unsupported reference-format decision independently reviewed | Accepted numbered scope; evidence in task file. |
| [23](../tasks/23-windows-packaging-and-user-docs.md) | blocked | isolated offline/no-Rust/other-user installation acceptance unavailable |
| [24](../tasks/24-final-prd-audit-and-main-push.md) | blocked | Final source/CI/Git verified; mandatory isolated-runtime/OS/speech acceptance remains unavailable. |

The done page tasks credit their earlier separately reviewed parser/fault/security/recovery tests and native workflows. Source tests are not relabeled as newly recorded native actions. All mandatory requirements remain; [normative sections 1–12](qa/task24/requirements-preservation.json) exactly match the reviewed baseline after CRLF normalization (28,322 characters, SHA 6d28a5ca846517d3a4519f048d2fae7352c774c5165f84f8dc2172423622fcf7).

## Genuine remaining acceptance and resume

Effective disconnected first use, screenshot/code/export/import without Rust installed, and recovery under another Windows user/isolated installation remain unverified. No ready isolated interactive VM/user is available; read-only Sandbox inspection found it disabled, and the earlier per-app firewall category did not enforce isolation. Its owned rules were removed. Same-user fresh keys and a clean CI builder do not prove the missing end-user runtime condition.

Actual Windows session-lock/suspend/logon delivery remains unverified on the shared host. App Ctrl+L, controlled idle, injected lifecycle events and registration toggles do not pretend to prove those events. Narrator live-code rollover/non-code focus was observed, but official Speech Recap exposed only window chrome and failed a known-utterance positive control twice. Heard/automatic code announcements remain unverified; semantic Role::Status/Live::Polite and AX labels are separate implementation evidence.

Resume on a ready isolated Windows environment: disable networking effectively, omit Rust from the runtime machine, run first-use scan/HMAC/Copy/export and independent-user restore/restart, then test actual OS lock/suspend/logon and Narrator with an audible or working speech observer and a positive control. Persist only sanitized/disposable results and obtain independent review before closing those acceptance items. No authentication/security bypass, shared-host lock/reboot or invented pass was used here.

Exact installed private Windows-reference pixels/version were unavailable; the authorized PRD inventory fallback and public reference limitations remain explicit. Private `.vault` incompatibility is completed discovery, not a waived native-backup requirement. Arbitrary unrendered hover/error/scale permutations are not automatically passed, but no additional required feasible case was identified by the final page-state re-review.

## Safe state and review limits

At the last native handoff all fresh owned helpers/production windows closed normally, Narrator was absent, monitors 100 percent, Contrast None and disposable idle 300 seconds. Ordinary AppData remained absent and the preexisting redirected directory was restored empty. Only proven fresh runtime files were removed after verified ciphertext preservation; old Task 21/23 windows were left untouched. GetPackageFullName 15700 and the observed redirected data root are separate facts, not a causal or other-user claim. No production data-path override exists.

No real credentials were used or committed. Retained QA images were individually inspected and mask sensitive-shaped fields before disk. Two transient test-tool outputs contained a published disposable OTP (one reviewer AX emission, one author unsettled screenshot); no value is repeated or retained in QA artifacts, and universal absence from all tool output is not claimed. Digest oracles are limited to public fixtures, not general credential telemetry or encryption. Endpoint/OS memory, clipboard history and external screenshots remain documented limits.

The [dated advisory audit](qa/task24/advisory-current.md) checked 385 locked dependencies against the recorded RustSec revision: zero reported vulnerabilities, three unsuppressed Unix maintenance warnings, none in the Windows normal/build inventory. This is scoped source/test evidence, not external cryptographic certification.

The [checker calibration](qa/task04/calibration.md) used the removed detached copy `C:/Users/root/AppData/Local/Temp/authenticator-ui-cal-4cg1yjfn/checkout`. A separate calibrator found the planted TITLE Regular 400 instead of SemiBold 600 in source; the original source was unchanged. Copied-executable launches returned ShellExecuteW 5, so live calibration was unproved. No plant tested pixels/rasterization, contrast, clipping, zoom, focus, pointer/key input or speech. Actual unmodified images and real interactions are separate evidence, and the implementation reviewer was neither author nor calibrator.
