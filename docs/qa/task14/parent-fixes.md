# Task14 parent identity correction

13 November 2026. Distinct reviewer14 found that a missing identifier and the
literal identifier `No account identifier` could render identical identities
without disambiguating UUIDs. Parent corrected checklist position7 in the existing
Task14 process: compare what the person sees, and show the exact account ID in
rows, selected summary and handoff whenever those displayed identities collide.
The shared typography/components and all other checklist positions are retained.

`src/ui/backup_codes.rs` now uses one borrowed `display_identity` function for
rendering, projection ambiguity indexing and handoff ambiguity detection. It
compares service and the displayed identifier fallback consistently, instead of
comparing the original optional identifier. No records, keys, persistence policy,
search/sort behavior, route ID bindings or downstream15/16 capability changes.

`backup_display_placeholder_collision_has_exact_ids_in_list_summary_and_both_handoffs`
creates four validated encrypted accounts: missing/literal identifier pairs with
and without existing sets. It checks list UUIDs, all selector UUIDs, each exact
selected summary, distinct actual editor/detail ID targets and concealed handoff
UUIDs at340/600/960. Only generated public recovery values are used; no real data.
Context input/AX remains synthetic, not native or rendered visual proof.

Focused command `rtk cargo test --locked --lib backup_display_placeholder_collision`
passed1 test in6.12s. Corrected `rtk cargo test --locked --all-targets --all-features`
passed151 tests,4 direct helper entrypoints exercised by their parents,17 suites
in103.62s. Strict all-target/all-feature Clippy, format/diff checks and explicit
Authenticator/lib/owned-backup_codes-example all-feature build passed. Focused
independent re-review approved the corrected source scope: collision1 passed7.29s,
existing route/help/Lock1 passed21.36s, diff check passed; no new material finding.
Task15/16 body and
native/manual gates remain open. Parent correction reading cost is approximately
3 project files and1 grouped search; previous author/reviewer costs remain in
their reports. No new calibration or general visual sensitivity claim.
