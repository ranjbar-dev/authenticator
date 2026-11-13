# Task18 hosted Windows follow-up

Parent integration, 13 November 2026. Reviewed source checkpoint `9910b4ab18c9d3b4e8785af888b9ea9ec3c3e19e` was committed and pushed to main, and remote equality/clean working tree verified before Task19 began.

[Windows CI43383451358](https://github.com/ranjbar-dev/authenticator/actions/runs/43383451358) **FAILED**. Formatting and strict lint passed; the library suite reported **190 passed, 2 failed, 4 directly ignored helper entrypoints, 624.46s**. The production build step was skipped. This supersedes any pending-CI description; no green full-suite result exists for this checkpoint.

Failures:

- `ui::shell::export_tests::export_close_and_idle_clear_forms_before_pending_work`: immediate `!app.state.session_active()` assertion at `src/ui/export_tests.rs:697` after closing an open export form with dirty display preferences. Task18 now discards the form before eligible asynchronous preference flush and locks afterward. Source diagnosis must prove immediate presentation/AX privacy, saved latest intent and eventual locking, rather than simply deleting the assertion.
- `ui::shell::trash_tests::trash_context_widths_pagination_safe_dialog_cancel_focus_keyboard_and_metadata_only`: `missing keyboard target`. The keyboard branch presses Enter to initiate asynchronous preparation, then immediately snapshots the `Restore record` target. Unlike the pointer helper, it does not wait for preparation. Source diagnosis must hold preparation deterministically, verify the intermediate state, then await completion while retaining actual Context Tab/Enter and painted clip/focus assertions.

All eight new domain Trash cases and the other new Trash worker/Close/fault/portable cases reported `ok`. The previous folder portable and recovery-detail deep-scroll cases also passed. These individual passes do not override the aggregate failure or establish native acceptance.

The sole Task19 implementation child received both exact failures for sequential diagnosis/repair. The parent has not edited product/test source concurrently. Distinct review19 must recheck both corrections along with Task19, followed by a new hosted full-suite result. Native screenshots/input remain zero, and earlier hardware/offline/clipboard gates stay open.

Results were retrieved from GitHub's authenticated Actions API using the configured Git credential in process memory, without printing or persisting authentication. Signed log storage was fetched without forwarding that credential. Only selected test names, status summaries and the failure location were emitted; no user vault or real credential was accessed.
