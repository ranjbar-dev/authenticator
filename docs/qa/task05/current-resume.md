# Current Task05 native resume

13 November 2026, implementation child `resume05_native`. Bounded author
evidence awaiting independent review; Task05 is not complete.

## Preserved source and mask correction

Inspected and preserved the existing dirty `src/ui/import_review.rs`,
`src/ui/shell.rs`, `examples/workflow_qa.rs`, Task21 evidence and task file.
The existing correction enrolls the actual manual/QR secret-field rectangle
and optional creation recovery-text rectangle in debug evidence masks. It
preserves masks through Lock until the next paint pass, since emitted shapes
belong to their frame. No new production change was needed in this resume.
The harness is a disposable-only development tool, not a general metadata/PII
sanitizer. Do not use it on a real vault.

Current-source commands:

| Command | Observed result |
|---|---|
| `rtk cargo test --offline --locked --lib account_evidence_masks_revealed_seed_and_creation_backup_through_lock -- --exact --nocapture --test-threads=1` | 0 selected,210 filtered; wrong short exact filter, not passing evidence |
| `rtk cargo test --offline --locked --lib account_evidence_masks_revealed_seed_and_creation_backup_through_lock -- --nocapture --test-threads=1` | 1 passed,0 failed,209 filtered,5.76s |
| `rtk cargo build --offline --locked --example workflow_qa` | passed,9.35s |
| `rtk cargo fmt --all -- --check` | passed |
| `rtk git diff --check` | passed |

The meaningful regression inspects actual revealed text shapes for the published
RFC seed/disposable recovery text, rather than accepting a nonempty mask list.
It covers Create/Import at340/600/960 with1/1.5/2 app zoom, verifies containment,
then frame lifetime through form removal/Lock and next locked paint. It writes
no screenshot. Physical display scaling is not established by this test.

## Actual native input and outcome

Read Computer Use skill, guidance, API and confirmations. Fresh inventory found
no old `workflow_qa` process or historic root `task21-owned-8lrp7u`. Launched
the newly built argument-free fixture through `sky.launch_app`, selected the
unique returned window67906, and used fresh native observations between inputs.
No default/reference vault or unrelated application was opened or modified.

Actual pointer/physical-key path:

1. Masked Create vault passphrase and confirmation, pointer Create, visible
   busy/progress/Cancel, then empty Codes with Search focused.
2. Pointer Lock returned an empty focused Unlock passphrase field.
3. Deliberately wrong disposable phrase, pointer Unlock, visible busy, then
   safe incorrect-or-damaged error with the passphrase cleared and locked UI.
4. Correct retry returned empty Codes; real `Control_L+l` returned Unlock
   with empty passphrase, no credential/account nodes and focus in that field.

`sky.type_text` returned without insertion; physical `sky.press_key` inserted
characters correctly. The underlying helper issue is not diagnosed. One Tab
step did not visibly move focus, so confirmation used a real pointer selection;
this report does not claim a complete keyboard-only creation journey.

F10 saved actual600x800 content frames at100% app zoom. Copied and visually
opened [wrong phrase](current-wrong-phrase.png) and
[keyboard lock](current-keyboard-lock.png): readable shared dark/blue controls,
visible field focus, no clipping in these two states, empty passphrase and no
credential values. These clean empty-vault frames do not establish populated
concealment or full-app visual acceptance.

After native input, fresh process metadata and ownership manifest matched
PID24088, executable `D:/Projects/github/authenticator/target/debug/examples/workflow_qa.exe`,
creation13Nov2026 16:05:07 local, root `target/task21-owned-Ji68kC`.
The exact files001/002 and readiness markers were present. The process remains
open and locked for independent review. Recheck identity before any later action.
Foreground exclusivity was released to the parent before shell verification.

## Honest open gates

No new relaunch, populated capture/review/import, recovery/clipboard lifecycle,
idle expiry, offline-machine, actual OS session lock/suspend, physical/mixed
DPI, Narrator/high contrast or exact-reference comparison is claimed. Prior
Task05 and Task21 reports retain their attributed scope; this resume does not
re-certify them. No VM/disconnected profile was available in the prior21 probe;
shared adapters, OS session and security/privacy settings were not changed.
Task05's open acceptance boxes remain open.

Parent should obtain distinct read-only UI/security review of this report,
the existing source mask correction and prior Task21 workflow evidence, then
continue independent discovery/packaging work without waiving hardware gates.
