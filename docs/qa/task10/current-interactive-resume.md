# Current interactive clipboard resume

13 November 2026. Implementation child `resume10_clipboard`; independent review
required. Production source and ownership checks are unchanged. Task10 remains
incomplete: keyboard Copy and rejected-published-copy cleanup have an unresolved
native failure.

## Harness and safe evidence

Added `examples/interactive_clipboard.rs`, an argument-free debug-only Windows
probe using the actual `AuthenticatorApp`, clipboard actor, default30-second
preference and an isolated encrypted vault with one published RFC-seed account.
It never opens the default/reference vault. The release/non-Windows entry is
inert and its diagnostic module is excluded.

The observer opens the clipboard exclusively and inspects format/owner metadata.
It reads a bounded Unicode value only for its exact app HWND with the app lease
format present and a queued-copy observation, or the unchanged sequence of a
previously verified fixture copy. Comparisons use only generated public-fixture
codes; no text, code, hash, seed or passphrase is written. A separate owned hidden
HWND supplies an intentional harmless subsequent copy, read only when that exact
HWND owns it. Arbitrary initial clipboard contents are never read. Format presence
is not proof of the actor's exact random nonce/digest; the observer does not claim
that stronger proof or diagnose the actor's exact rejection subcondition.

The first diagnostic polled immediately after input and could itself contend with
the clipboard actor. Final source postpones automatic observation250ms after
input and closes its OS clipboard handle before file I/O. The final native run
still returned Changed. This correction changes only the probe, not production
timing, lease admission, sequence or ownership safeguards.

Evidence files contain booleans, timestamps and clipboard sequence metadata:

- `current-interactive/initial-observations.jsonl`
- `current-interactive/sequence-lock-observations.jsonl`
- `current-interactive/quiet-observer-observations.jsonl`
- `current-interactive/sequence-run.png`
- `current-interactive/quiet-observer.png`

The two actual600×800 F10 PNGs were masked before their only disk write and opened
after copying. Both conceal the code rectangle and show visible keyboard focus on
Copy. They are scrolled focused-component frames, not full-page/three-width visual
acceptance or screenshots of the short-lived error status. No raw screenshots or
clipboard text were saved. Transient UI observations supplied the exact statuses
below; no native tree/code value is included in repository evidence.

## Actual pointer/keyboard outcomes

Read Computer Use skill/guidance/API/confirmations. Launched each uniquely returned
owned executable through `sky.launch_app`. Used actual pointer clicks and physical
keys with fresh UI states. Accessibility-only snapshots sometimes lagged an input
frame; a second observation established settled focus. An initial pointer action
without screenshot geometry was rejected without action; refreshing screenshot
geometry allowed the real click. No synthetic `element.click()` was used.

| Journey | Actual result |
|---|---|
| Pointer Copy, default timeout | Success status. Own HWND/lease/public-code match=true at Unix1794574094; Unicode absent1794574124, exactly30s later. |
| Newer harmless copy | Second pointer Copy matched1794574157. F3 separate-HWND writer1794574175; sentinel intact1794574216, beyond the original deadline1794574187. |
| Keyboard reachability | From Search, real Tab reached Sort → Folder → View → Manage folders → Copy; settled AX focus reported Copy. Return did not establish activation. Space dispatched actual clipboard attempts. |
| Confirmed pointer Copy then Ctrl+L | In sequence-aware run: Copy success/native match at1794574510, sequence64. Ctrl+L locked1794574517; Unicode became absent and sequence65 in that same second. Locked UI had empty passphrase/no account/code nodes. |
| Early keyboard/Lock anomaly | Initial unsequenced run showed own lease/text after keyboard attempt and later Lock/exit, but immediate success/error and sequence were not captured. Cause unestablished; this is not proof that a successfully leased Copy bypasses Lock cleanup. |
| Observer contention/retry | Sequence-aware probe's immediate polling could race the actor; two Space attempts returned explicit Busy. Final harness avoids sampling for250ms after input. Those errors are recorded, not passes. |
| Final quiet-observer keyboard Copy | Settled Copy focus → physical Space immediately showed **“The clipboard changed before copying finished. Try Copy again; newer clipboard data is preserved.”** The probe then matched the public fixture code with app HWND/lease present at1794574732, sequence73. No keyboard success is claimed. |
| Exit after rejected published Copy | Normal Alt+F4 invoked real `app.on_exit`/Lock/shutdown. At1794574800, the same sequence73 and own HWND/lease still contained the matched public fixture code, unlocked=false. This is an unresolved rejected-copy residual, not a passing cleanup result. |

The final outcome warrants focused independent security review. Source
`native::copy_after_close` returns Changed when post-close content proof fails;
the actor then has no new lease to expire/clear. The native evidence establishes
residual fixture text after rejection and shutdown, but does not establish which
HWND/nonce/size/digest subcondition originally failed. Do not weaken the sequence,
random marker or digest checks to force a pass. One possible repair to assess is
retaining a published attempt's provisional evidence solely for guarded cleanup,
while still reporting failure; this report does not implement or approve it.

## Commands and source validation

| Command | Result |
|---|---|
| `rtk cargo fmt --all` | passed after final probe changes |
| `rtk cargo clippy --offline --locked --example interactive_clipboard -- -D warnings` | passed after correcting two initial collapsible-if diagnostics |
| `rtk cargo build --offline --locked --example interactive_clipboard` | final build passed2.70s |
| `rtk cargo check --offline --locked --release --example interactive_clipboard` | passed26.21s; no release diagnostic module |
| `rtk cargo test --offline --locked --lib platform::clipboard::tests::actor_expiry_hidden_lifecycle_lock_shutdown_and_newer_identical_external_copy -- --exact --nocapture --test-threads=1` | 1passed,209filtered,1.25s; injected actor regression, separate from native evidence |
| `rtk git diff --check` | passed |

Initial Clippy diagnostics were fixed; no failing command is silently called
passing. No full-suite or current hosted-CI result is claimed here. Parent's
unrelated Trash CI follow-up remains outside this child's changes.

## Ownership and remaining work

All three probe windows were normally closed; fresh `sky.list_windows` returned
none and shell process queries found PIDs2608,30028,14808 absent. Exact retained
owned roots (no recursive cleanup performed):

| Root under target/ | PID / native root HWND | Scope |
|---|---|---|
| `task10-interactive-JDYCX6` | 2608 /1640128 | initial expiry/newer-copy and unsequenced anomaly |
| `task10-interactive-Lg4xtF` | 30028 /7669830 | sequence-aware pointer Lock success/Busy keyboard |
| `task10-interactive-LHVgSo` | 14808 /7735366 | quiet-observer Changed/residual/exit |

Each root has an ownership manifest with the exact executable and root. Its vault
is disposable encrypted data; none is included in Git. Recheck identity before
future cleanup and never act on a recycled PID. Other windows/processes, including
the parent's locked workflow fixture, were untouched. The desktop clipboard may
still retain the expired public fixture code after the demonstrated failure;
no arbitrary subsequent user clipboard was read or cleared.

Changed paths are this report, the example and the five evidence files above.
No task status, PRD, production source, commit or push was changed by this child.
Native success-path exit, keyboard Copy success, rejected-copy cleanup and full
Task10 acceptance remain open. Physical DPI, actual OS lock/suspend/clock delivery,
network-disabled machine, Narrator and exact reference fidelity remain unverified.
Parent should obtain distinct review, integrate/fix findings sequentially and
re-review material corrections. Do not mark the task done from these partial passes.
