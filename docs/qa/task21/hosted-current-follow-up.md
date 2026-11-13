# Hosted source validation after native21 corrections

[CI43446743225](https://github.com/ranjbar-dev/authenticator/actions/runs/43446743225)
completed **SUCCESS** on13November2026 at15:09:31UTC for
`3b5185187894d8b1be1d6b0a6abe01dd33671ce1`.
Format, strict lint, all-target tests and Windows app build passed.

Library207 ordinary cases passed,0failed,4 direct helper entries ignored,
885.34seconds. Main1ordinary passed/1direct helper ignored; other top-level
suites18ordinary passed. Total226ordinary cases, with5direct ignored helper
entries exercised by their parent tests. Nested subprocess summaries with
nonzero filtered counts are not counted a second time.

The exact test log confirms all three recent regressions passed:

- `rejected_published_copy_is_not_success_and_retains_guarded_cleanup`
- `account_evidence_masks_revealed_seed_and_creation_backup_through_lock`
- `trash_permanent_scope_fault_retry_latest_preferences_and_encrypted_restart`

Parent queried the public jobs/run APIs and retrieved bounded logs through the
existing credential helper without exposing credentials. The ignored
`target/current-ci-counts.py` selects one Tests log and sums only top-level
zero-filtered summaries. Earlier failed runs remain documented in
current-ci-follow-up.md; this success does not rewrite their history.

This closes the current source full-suite gate, including the MSRV-compatible
test-counter correction on the hosted compiler. It does not establish physical
OS/DPI/offline/accessibility/logon checks, exact native reference parity or new
Task23 package/resource/dependency behavior. Task22's following documentation-only
commit and later packaging changes are not claimed as this run's SHA.
