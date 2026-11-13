# Task13 parent corrections

13 November 2026. The distinct reviewer withheld source approval for two P2
findings. Parent corrections follow the existing Task13 checklist and shared
ROOT controls; no new visual component, theme value or unrelated page is added.
This is part of the authorized import workflow, including its shared engine
decision data, rather than an isolated style adjustment. The existing same-kind
source calibration remains limited as described in the independent report.

The changed checklist positions are: folder selectors offer only authenticated,
engine-supported decisions with the default retained; final affected-data review
shows the actual candidate's paginated account–folder associations and record
dates/Trash states through the same heading/body/secondary/card roles; named
pagination controls identify which collection they navigate. Other Task13 roles
retain the existing checklist. No new majority or exact-reference claim is made.

`FolderDecisions` is computed once on the preview worker from current global IDs
and same-ID/name folder indices. The engine's existing validation uses the same
permission rule. The UI shows new-folder guidance instead of an invalid selector
for a unique incoming folder; same-ID/name reuse/copy and pure cross-type copy
remain available. No write policy or source/local revision binding is relaxed.

The final plan now renders actual candidate memberships, five per page, with
account service/identifier/ID, folder identity/metadata, membership ID and its
creation/update/Trash state. It shows no seed, OTP or recovery value. Mapping is
read after the engine applies KeepBoth remaps and folder reuse.

Initial focused checks passed:

- `rtk cargo test --locked --lib folder_decisions_match_engine`:1 passed,0.00s;
  unique, same-name, stable-ID and cross-type cases compare every offered choice
  with real engine planning, including rejected choices and default success.
- `rtk cargo test --locked --lib backup_import_unique_folder_and_paginated`:
  1 passed,28.06s; fresh export/preview/plan, unique-folder guidance, active and
  trashed associations, exact endpoint IDs, five-row pagination and secret-free
  AX text at340/600/960. Framework events remain synthetic.

Corrected full `rtk cargo test --locked --all-targets --all-features` passed143
tests,4 directly ignored subprocess helpers exercised by their parents,16 suites
in91.69s. A subsequent pagination focus correction retains both named buttons
while more than one page exists, disabling unavailable directions instead of
conditionally reusing Next's identity for Previous. The new regression checks
both AX identities across forward/back pages. Final `rtk cargo test --locked
--lib backup_import_` passed10 tests in42.40s after this correction. Strict
all-target/all-feature Clippy, format/diff checks and explicit production/lib/
owned-import-example build passed on the final source. No all-target standalone
build or unrelated-preview cleanup was attempted.

Independent focused re-review approved both corrections with no new material
findings: engine-decision1 passed0.00s, import10 passed40.80s, diff check passed.
See the appended report. Native picker/input/framebuffer/visual
gates remain open; no new native launch is implied by these corrections. Parent
fix reading cost is approximately10 project files and8 grouped searches; exact
totals were not instrumented. The whole Task13 is workflow implementation with
shared engine logic, not a small style change; its author's/reviewer's broader
counts remain in their evidence.
