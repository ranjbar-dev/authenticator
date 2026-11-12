# Task08 implementation handoff

12 November 2026. Author handoff retained below; subsequent distinct UI/security/data-loss
review approved the functional implementation with no findings requiring changes.
See [independent-review.md](independent-review.md) for independent results and limits.
Parent owns task status, coverage, commit/push. No real/default/reference vault was opened. The
existing unrelated `vault_lifecycle.exe` preview was left alone.

## Scope, source checklist and decisions

Read the task08 screen/acceptance checklist, CODEX_PROMPT, PRD FR2/FR2A and §§5,
7–12, checkpoint, task05/06/07 evidence, security-design, architecture,
ui-foundation and storage-format. RTK.md governed every shell entry. Reference
pixels remain unavailable as recorded in `docs/reference-ui.md`; PRD inventory
is the reference, with no claim of pixel fidelity. The user's UI lessons were
read. UI-consistency finding-patterns/values/decisions/implementing and their
applicable process/checklist/design-reading guidance informed this extension;
the user's existing plan/autonomy and no-child instruction governed execution.
No additional implementation/review child was spawned.

The production UI family remains one ROOT shell file. The grouped reach search
`rtk rg -l 'c::(button|labeled_field|labeled_secret_field|card|heading|status)' src`
finds `src/ui/shell.rs`; `components.rs` owns those helpers and `theme.rs` owns the
actual complete typography/metrics/color pairs. There is no repository DESIGN.md.
This is shared-piece reuse, not a fabricated majority over new pages. Diagnostic
FoundationApp/examples are not production family members. Approximately 25 source,
task and documentation files and 20 source searches were read/run, plus skill and
support reads (estimates, not a claim of an exact cost ledger).

The task's preapproved checklist is implemented in its existing shell positions:

1. ROOT CentralPanel + existing vertical holder/header/tabs; Screenshot to Scan
   stays primary. Image and one Manual action sit alongside it. Existing theme
   padding/gaps/content bound are reused. Roles/order are from the task/reference;
   mechanism/values are the shared piece, decision level4.
2. Candidate chooser: recognizable service/account labels, explicit selected
   state, **none selected by default when multiple supported candidates exist**,
   disabled Review until at least one is chosen. Existing full-width button and
   heading/secondary roles; no new custom selector styling.
3. Review: service, optional website/account, masked Base32 secret and intentional
   reveal, algorithm/digits/period through existing labeled native editors. New
   generic secret guidance also describes the blank manual fallback accurately.
   All URI-supported settings are preserved; every selected editor can be opened.
4. Nonblocking duplicate warning uses the same domain predicate for vault/trash
   and batch, including Unicode lowercasing. Explicit **Save selected as new
   accounts** preserves all existing entries; there is no overwrite path.
5. Fixed safe validation/write feedback retains edits, scrolls into view and
   offers correction/Save/Cancel. Save revalidates the entire selected batch.
6. Busy replaces editor/submit nodes with themed progress, accurate saving header,
   and Cancel-and-lock. Single worker + API gates prevent repeated activation.
7. Success shows actual committed IDs/account/code and scrolls the imported card
   into view. CODE typography and ordinary non-live accessibility text avoid
   repeated OTP announcements; no invented business records/current codes exist.
8. Cancel/back/Escape outside a dialog clear review/editor buffers and return
   Screenshot focus. Escape inside Help closes Help while preserving edits.
   Lock clears credential nodes and focuses the passphrase field.
9. Native picker is a documented platform exception: legacy Win32 common dialog
   on its own worker, no COM requirement, does not disable ROOT. Timer cancellation
   dismisses it; bounded read/decode is also off the event thread. No path logging.
10. Excluded profile/cloud/language/extension controls were not added. Existing
    task07 native **19px GDI row** exception was not changed. No new production
    visual component or private theme was introduced.

The new `import_review` module is ephemeral workflow data, not a parallel app or
decrypted committed-record cache. `AppState` remains the sole repository owner;
the repository moves to one worker for save and is returned unlocked on ordinary
write failure. Secret/editor strings zeroize and editor history is dropped at
boundaries. Framework/font/allocator/OS copies cannot be guaranteed erased.

## Atomicity and cancellation

`VaultStore::transaction_cancellable` keeps the existing detached-candidate,
whole-vault validation, authenticated encryption, external-replacement check and
same-directory ciphertext atomic rename. Cancellation shares the rename mutex;
additional lifecycle/idle guard is checked at that gate. Selected batches commit
once. Precommit invalid edits, faults, cancel and stale results cannot partially
mutate committed state. A cancellation that **loses** the rename gate may leave
the **complete encrypted batch**, then a locked session. Busy copy states that
fact; no postcommit rollback/deletion is claimed.

API tests reject capture/image/save overlap even when bypassing ordinary shell
controls. Tests exercise recoverable write failure + retry, duplicate separate
IDs, completed-but-unpolled save cancellation, immediate cancellation of a real
new worker and locked/stale result rejection. Storage tests inject partial-write
and before-rename failures, invalidate a guard after edits but before rename,
verify unchanged ciphertext/memory and no temporary remnants, then verify both
IDs survive an encrypted lock/unlock. Existing capture tests cover actual decoded
secret-bearing publication races and native epoch/idle rejection. The extended
review AX test proves masked secret, metadata removal on lock and Help/Escape
retaining review.

## Actual pointer, keyboard and image journeys

Used the documented Computer Use `@oai/sky` API from node_repl, with exact returned
owned windows and fresh state observations. Pointer actions were SendInput-based,
not synthetic element.click. Physical `press_key` was used for egui editing;
native filename typing used the native edit control and Alt+N mnemonic. No raw
PowerShell UIAutomation, private helper protocol, OS networking/display/security
change or installed reference app interaction occurred.

Argument-free `examples/import_flow.rs` creates a fresh encrypted TempDir vault.
`qr_fixture` / `qr_batch_fixture` are separate processes drawing published RFC6238
fixtures in memory. The preview additionally generates an **original** PNG input
in a separate owned TempDir for the image fallback test; it is not a screenshot.
No source QR, selected pixels, raw payload or user-desktop image was saved to the
repository/evidence. Both generated PNG files from the final two previews were
observed absent after closing their owning previews. Graceful preview exit also
drops its owned encrypted test vault. The public fixture windows were closed.

Observed journeys:

- Primary screenshot action → native overlay → real pointer drag over another
  app's QR at physical `(95,342)432×432` → masked Public Fixture/disposable review.
  No secret typing or screenshot-file step. Source URI settings SHA1/6/30 stayed.
  Physical period edit30→9 → explicit Save rejected →9 retained with safe error.
  Physical Ctrl+Z restored30; Save committed the actual entry and current code.
- Two QR selection at `(75,319)730×360` → chooser with neither selected. Space
  selected Second Fixture; pointer selected Public Fixture; Review selected both.
  Second URI SHA512/8/120 remained unchanged. Both editors were available, selected
  batch stayed intact through failure/retry, and success rendered both real IDs.
- Preview-only F4 deliberately replaced its own ciphertext with inert conflict
  bytes. The application rejected Save without changing those **faulted pre-save
  bytes**, preserved its last committed in-memory vault and retained both editors.
  F4 then explicitly restored the original fixture ciphertext before retry.
  This test does **not** claim the harness never changed the file. Storage fault
  tests separately prove ordinary app write failures preserve the original file.
- A second capture of an existing account showed nonblocking duplicate guidance
  and separate-new-entry Save wording. Escape canceled and returned Screenshot
  focus; the existing two-account vault remained unchanged.
- Unsupported HTTPS QR produced zero supported candidates and back/retry/image/
  manual recovery. Final source removes the redundant unsupported summary and
  initially focuses Back. The persisted unsupported screenshot is from before
  this small copy/focus refinement; it is not claimed as an exact final rendering.
- Native image picker Escape returned without importing. A second picker stayed
  open while ROOT remained usable and showed no account nodes. Physical Ctrl+L
  locked ROOT, dismissed the picker through its timer and left no account/secret
  nodes in locked AX (`locked-accessibility.txt`).
- Actual native PNG selection succeeded: pointer/Alt+N filename → Return → worker
  bounded decode → shared masked Public Image/fixture review. F4 fault → safe
  retained review error was automatically visible; restore → Save → actual new
  account/current code automatically visible. No source PNG is in evidence.
- Final executable: Manual opens the same blank editor with generic Base32
  guidance and service focus. Pointer Help → Escape preserved review and restored
  Help focus. Blank Save was safely rejected with error in view; Escape outside
  Help canceled, left the test vault empty and focused Screenshot. Ctrl+L during
  a previous manual review hid all credential/editor nodes.

The helper normalizes a 6400-wide selection surface to2560 horizontally while
retaining vertical coordinates, as task07 already documented. Used the observed
overlay screenshot ID rather than its16px owner placeholder: fixture physical
x is mapped as `(x+1920)/2.5`; y stays physical. Selected geometry above confirms
the mapping. No additional physical DPI claim follows from tool normalization.
An early AX-index click returned geometry-unavailable and did not start capture;
fresh screenshot-backed pointer input recovered. No failed action was counted.

## Independent current-code proof

`independent-code-proof.json` records screenshot import metadata only:
Unix1794504799–1794504800, counter59816826, SHA1,6digits,30seconds, **match=true**.
`independent-image-code-proof.json` records native-image import at
Unix1794505681, counter59816856, same settings, **match=true/same_counter=true**.

Independent implementation used Node.js standard-library `crypto.createHmac`,
an eight-byte big-endian counter, RFC dynamic truncation and decimal padding,
with the published RFC6238 SHA1 fixture bytes. It did not call the Rust engine,
totp-rs or the app parser. Compared the real committed UI text at the observed
counter. Neither JSON contains seed, payload, image, actual or expected OTP.
Persisted success components black out OTP pixels. An initial read-only AX
observation returned a prior frame and the first comparison failed; a fresh
physical-input observation/time bracket produced the recorded matching proof.
That failed observation is not claimed as a pass or a code-timer certification.

## Visual evidence

All JPEGs are readable bounded component/window evidence containing only public
disposable metadata and masked secrets; OTP pixels are redacted from success
components. The `.jpg` extensions match actual Computer Use JPEG output.

| Evidence | Observed state |
|---|---|
| chooser-340/600/960.jpg | Explicit selection, disabled empty-choice review, full labels/focus |
| review-340/600/960.jpg | Shared editor/card hierarchy, metadata labels and focus |
| masked-settings-340.jpg | Masked seed, reveal action, preserved SHA512/8 settings |
| save-controls-340.jpg | Preserved120-second period, readable Save/Back/Cancel |
| invalid-edit-480.jpg | Retained invalid9 period and safe validation feedback |
| write-error-340.jpg | Whole-batch retry guidance with retained values |
| write-error-final-480.jpg | Final auto-scroll brings recoverable error/Save/Cancel into view |
| duplicate-600.jpg | Nonblocking warning and explicit separate-entry Save |
| cancel-review-600.jpg | Cancel feedback and Screenshot focus |
| unsupported-600.jpg | Zero-supported recovery; before final summary-copy refinement |
| image-worker-600.jpg | Cancellable off-UI picker/decoder, credential nodes omitted |
| image-review-480.jpg | Native PNG decoded into same review |
| imported-card-340-redacted.jpg | Real batch-import identity/code card; code blacked out |
| imported-image-card-480-redacted.jpg | Real PNG-import identity/current-code card; code blacked out |
| locked-after-picker-600.jpg + locked-accessibility.txt | Locked and picker dismissed; no credential nodes |
| locked-after-review-480.jpg | Review cleared and passphrase focus |
| manual-final-480.jpg | Final blank shared editor/service focus |
| manual-validation-final-480.jpg | Final empty-manual validation before any save |

Source typography/theme and actual screenshots were inspected at all three
logical widths; buttons/labels remain readable, field values retain contrast,
active icon tiles are opaque and focus is visible. Narrow settings/actions are
reachable through the shared scroll holder. The test-only notice consumes extra
height; production has no diagnostic notice. Broader accessibility/reference
comparison belongs to the distinct reviewer and final QA, not this author.

## Exact checks

All shell commands were RTK-prefixed on Windows/MSVC Rust1.98.

| Command | Observed result |
|---|---|
| `rtk cargo check --locked --all-targets` | Passed after integration and final UI refinements |
| `rtk cargo test --locked import_ --lib -- --test-threads=1` | 3passed,57filtered,20.93s (initial editor/storage/worker tests) |
| `rtk cargo test --locked editable_review_masks --lib -- --test-threads=1` | 1passed,60filtered,6.33s (initial AX test) |
| `rtk cargo test --locked --all-targets -- --test-threads=1` | 76passed,2ignored helper entrypoints,61filtered subprocess entries;12suites151.51s |
| `rtk cargo test --locked ui:: --lib -- --test-threads=1` | Final scoped regression14passed,48filtered,52.59s; includes extended real-worker cancellation and Help/Escape/locked AX |
| `rtk cargo test --locked duplicate --lib` | Final3passed,59filtered,0.01s, including Unicode consistency |
| `rtk cargo test --locked --test totp_domain` | Final12passed,0.01s |
| `rtk cargo test --locked import_batch_fault_cancel_at_commit_and_success_are_atomic --lib -- --test-threads=1` | Final1passed,61filtered,8.15s |
| `rtk cargo build --locked --bin Authenticator --example import_flow --example qr_batch_fixture` | Final passed after closing own running fixture executable |
| `rtk cargo clippy --locked --all-targets --all-features -- -D warnings` | Passed, no issues |
| `rtk cargo fmt --all -- --check` | Passed |
| `rtk git diff --check` | Passed |

Intermediate compile/lint failures (Zeroizing Display/test constructor fields,
status helper returning unit, redundant formatting borrows) were corrected and
checks rerun. A final fixture build initially hit Windows LNK1104 because the own
running `qr_batch_fixture.exe` was locked; closed that owned window, retried the
same command and passed. No user/reference process was terminated or altered.

## Remaining gates and limits

Subsequent Task09 review corrected the shared save cancellation implementation:
atomic admission and nonblocking cancellation replace the old mutex that could
delay UI Lock behind an admitted filesystem action. Independent held-save and
publication-race tests passed; all-or-nothing persistence is unchanged. See
task09/independent-review.md. Active worker memory is not forcibly erased during
kernel I/O/crypto. Earlier mutex descriptions reflect the Task08 checkpoint.

- Separate UI and focused security/data-loss review subsequently passed within
  the documented functional scope; author did not self-accept or change statuses.
- Actual networking-disabled journey is **unverified**. No OS network setting was
  changed; source/dependency path uses local operations and no runtime network
  client, but that is not a substitute for the requested disconnected-machine gate.
- Physical150/200%/mixed-DPI, right-monitor capture, real hot-unplug/permission
  denial, actual OS lock/hardware suspend and Narrator remain task05/07/final-QA
  gates. Private WTS tests are not those hardware actions. This task's actual
  screenshot capture used the primary96DPI fixture; prior task07 negative-monitor
  proof is preserved rather than reasserted as a new full import/DPI test.
- Live decode/save are short; cancellation timing is additionally exercised at
  worker/publication/commit boundaries by deterministic tests. No claim is made
  of manually interrupting Argon2, decoder internals or an already-entered rename.
- Basic manual fallback is usable now; task09 still owns full editing/recovery-code
  creation. Task10 still owns complete list/sort/views/copy workflows. Backup,
  organization, packaging and wider security gates remain their required tasks.
- No complete memory erasure, power-loss durability, reference-pixel fidelity,
  migration compatibility or external security certification is claimed.

Changed implementation paths: Cargo.toml; src/domain/records.rs;
src/platform/{mod.rs,image_picker.rs}; src/storage/{mod.rs,tests.rs};
src/ui/{mod.rs,import_review.rs,state.rs,shell.rs};
examples/{import_flow.rs,qr_batch_fixture.rs}; docs/{architecture.md,
security-design.md,storage-format.md,ui-foundation.md}; this evidence directory.
Parent-owned task/README/coverage modifications were preserved and not authored
as part of the implementation handoff. No commit/push was made by this child.
