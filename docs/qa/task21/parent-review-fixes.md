# Task21 review corrections

13 November 2026, parent changes after `review_native21` returned FINAL.

The import review caption now says “ID already in current vault” for
`local_global_id_collision`. That value covers all existing record IDs, including
same-kind folder IDs; “cross-type” was inaccurate. Counts, plans and merge policy
are unchanged.

Trash identity now omits the second generic ID for an Account only when the
shared account identity already rendered its exact ID. Missing-account fallback
and the distinct folder/set/code/membership IDs remain. This removes redundancy
without discarding disambiguation or changing stored records.

Checks:

- `rtk cargo fmt --all`: passed.
- `rtk cargo test --offline --locked --lib ui::shell::trash_tests::trash_context_widths_pagination_safe_dialog_cancel_focus_keyboard_and_metadata_only -- --exact --nocapture --test-threads=1`:
  **1passed,210filtered,13.85s**, using fresh owned D: TEMP/TMP subprocess scratch.
- `rtk cargo clippy --offline --locked --all-targets --all-features -- -D warnings`:
  passed.
- `rtk git diff --check`: passed.

No new tests mirror these small presentation changes. The existing case exercises
all record-kind metadata, sizes, focus, dialogs and keyboard behavior. No new
native screenshot or whole-suite result is claimed; focused independent recheck
remains required.
