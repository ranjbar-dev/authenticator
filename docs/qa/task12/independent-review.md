# Task12 independent UI/security review

Parent record,13 November2026. Distinct read-only `review_12` was not the author
or calibration agent. Original review requested a P2 correction; focused
re-review resolved it. **Source is approved within the reviewed scope. Task12
remains in_progress because native UI/visual and Task13 GUI restore gates are open.**

## Finding and correction

**P2: export could omit the latest displayed preferences.** Entering the form
during the500ms debounce blocked ordinary display-preference persistence, but
export used the older stored vault. An independent external source-copy probe
changed the cache/query through the real dirty-preference path, performed a real
worker export and authenticated engine preview, and failed the current-query
assertion (13.00s). This demonstrated incomplete current preferences in an otherwise
successful backup; it was not recorded as a passing acceptance case.

Parent added a guarded preparation stage using the existing mutation worker.
The form hides fields/submit while saving current query/sort/view/folder intent;
ROOT waits for successful current completion and any newer coalesced changes,
then starts export. The exact reviewed destination and explicit overwrite consent
remain bound throughout. Failure clears backup phrases, retains pending intent
for retry and stops before backup writing. Lock or unexpected repository locking
drops queued form/path/history. A completed earlier preference save is not falsely
rolled back if later export fails/cancels; documentation states this explicitly.

Focused independent re-review passed:

| Check | Result |
|---|---|
| Current query/sort/view/folder and complete exported codec | 1 passed,16.50s |
| Failed preference save preserves ciphertext/no backup, then retry | 1 passed,14.99s |
| Unexpected repository lock clears queued form and prior AX content | 1 passed,6.21s |
| `rtk git diff --check` | Passed |

No new finding arose. The existing write-boundary destination-change checks
remain intact. The source was freshly rebuilt after an external probe had used
the shared Cargo target cache; final counts refer to the repository source.

## Original independent scope

| Command | Result |
|---|---|
| `rtk cargo test --locked --lib export_` | 9 passed,29.89s |
| `rtk cargo test --locked --lib ui::export::tests` | 2 passed |
| Theme contrast test | 1 passed |
| `rtk git diff --check` | Passed |

Source review confirmed that file selection grants no overwrite consent, explicit
ROOT replacement retains the inspected target, passphrases use concealed shared
controls and clear at boundaries, busy prevents repeats, and guarded worker
publication/cancellation rejects late UI adoption. No additional security or
data-loss defect was established. Contrast resolves through the shared dark
theme: primary text5.85:1 and error text8.69:1. The eight checklist regions reuse
the actual ROOT controls/whole typography roles; no new colors/font scale were added.

The author fixed a parent-identified focus risk before review: the error holder
stays mounted so validation feedback does not change Submit identity. Synthetic
Tab/Enter/correction/default-Keep events exercise focus and overwrite behavior.
These and the AX/width tests are deterministic framework tests, not native input
or visual quality proof.

## Native limits and cost

The reviewer made one documented normal owned preview launch, which returned
`GetCursorPos failed: Access is denied (0x80070005)`. Fresh exact-title inventory,
process and TempDir checks found no owned preview/artifact. No additional attempt
occurred during focused re-review, and no cause is inferred or permission bypassed.

Consequently zero native pages/screenshots were rendered/evaluated by this check.
Real pointer/keyboard/focus, Save-dialog selection/cancel/Lock, new-user completion,
rendered340/600/960, long paths, short-window modal clipping, actual scroll/hit areas
and native error/success presentation remain unevaluated. Synthetic Context/AX
tests cannot close those gates. Task13 GUI restore, actual OS/hardware/offline/
Narrator/reference and prior Task10 Copy gates remain open.

Original review estimated three ROOT branches,24 project files and11 grouped
searches; focused review six additional file openings/two grouped searches,
including one preparation state. Combined cost estimates30 file openings and13
searches, zero rendered pages. Existing calibration proves source-level detection
with documented pixel/live limits; no new visual sensitivity claim is made.

The external probe copy was removed after exact resolved owned-root verification.
No repository edits, clipboard/default/reference access, production app launch or
unrelated lifecycle-preview interaction occurred. Final checks found no owned
preview process or Task12 TempDir; no new copy during focused re-review.

## Final parent checks

`rtk cargo test --locked --all-targets --all-features`: **131 passed**, four
directly ignored subprocess helpers exercised by parents,15 suites76.52s.
Strict all-target/all-feature Clippy, formatting/diff and production/lib/owned
export-example build passed. The known standalone all-target LNK1104 example
artifact limitation remains; no unrelated preview was stopped. Commit reference
follows in the parent checkpoint to avoid circular self-reference.
