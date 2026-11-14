# Follow-up execution, 14 November 2026

Parent resumed the task prompt from clean main8ac9f38 after reading the PRD,
task index/coverage and numbered tasks. Existing work was preserved.
Sixteen scopes remain done and eight blocked; no acceptance was weakened.

## Baseline integration checks

- Captured Git: HEAD/remote main both8ac9f38d9ced3bbf148a31c14647b2e09da90f8f;
  porcelain empty.
- Public GitHub REST run/jobs: [CI43618587528](https://github.com/ranjbar-dev/authenticator/actions/runs/43618587528)
  successful at that SHA, check/package jobs successful. Format, Lint, Tests,
  Windows app build, package validation/upload successful. Logs/test counts
  were not newly downloaded or counted.
- `rtk proxy python scripts/test_package_windows.py`:17 tests, OK,1.148s.
- `rtk proxy python scripts/package_windows.py --validate dist/Authenticator-0.1.0-windows-x86_64.zip`:
  archive paths, CRCs and manifest checksums passed.
- `rtk cargo fmt --all -- --check`: exit0.
- Python ZIP SHA256:9c8b781018c3e6b66d06cbe3f5d5a5184b620f0d9925ce738e941b953badbcc6.
  Captured `git diff --name-only 079c08d HEAD -- src Cargo.toml Cargo.lock build.rs scripts assets .github`:
  no paths. Existing clean079c08d/native-tested artifact attribution is preserved.
- Parent Python:24 unchanged task Status/checklist lists,48 unchanged coverage
  rows, PRD unchanged; initial140 Markdown files/390 relative links, none missing.
  `rtk git diff --check` passed. Final staged checks follow integration.

## Concrete attempt and review

Sole implementer `resume05_runtime` researched the supported Sandbox route and
made one parent-authorized, restart-suppressed feature-enable attempt. It required
a restart. Supported disable restored the originally Disabled feature, but newly
present CBS RebootPending/pending.xml remain. No restart was performed or scheduled.
This residual host servicing is material; feature rollback did not undo it.
[Sanitized report/results/probe](task05/sandbox-attempt-2026-11-14.md) retain exact
commands, sources, observations and maintenance resume. Raw logs remain ignored.

Distinct read-only `review05_sandbox` independently checked state/logs,
requirements preservation and Git/CI/artifact attribution. Its P3 stale current
resume wording was corrected and independently rechecked.
[Scoped approval](task05/sandbox-independent-review-2026-11-14.md); both sequential
children FINAL/inactive, with no close-thread API exposed by this runtime.
No source suite/native journey was repeated for this evidence-only change.

Reviewed Task05 evidence was committed/pushed as
`cfffcc5c3da2002fdec4adb0dc9b689bf4b9f2fb`; captured Git commands confirmed remote
main equality and clean tree before the next child. Its CI43627997981 was in
progress at the first check; final result is recorded only after observation.

## Independent remaining speech route

After both Task05 children were FINAL, sole implementer `resume19_speech`
investigated the documented Narrator copy route independently of the shared
offline/OS prerequisites. Official guidance establishes that Narrator+Ctrl+X
copies the last phrase only. Equality is not evidence of no intermediate speech.
The full-history/known-label oracle remains a concrete future action.

Two normal fixture launches returned GetCursorPos Access denied, with fresh
inventory between attempts. No target fixture/Narrator/new owned root was
observed. Read-only diagnosis in the separate exec caller found input-desktop
Open denied/error5, without establishing the sky helper's cause or locked state.
No speech positive control, clipboard action, screenshot or native acceptance
journey ran; existing processes/settings/servicing were untouched.
[Exact report and sanitized facts](task19/speech-copy-2026-11-14.md).

Distinct read-only review05_sandbox [approved this scoped evidence](task19/speech-copy-independent-review-2026-11-14.md)
after one P3 wording correction/recheck: the stopping decision is observed, not a
specific documented GetCursorPos launch retry limit. Both author/reviewer are
FINAL/inactive. No native ownership or cleanup remains.

All eight blocked scopes retain their original acceptance; no new source suite
is warranted for this evidence update. Next isolated-runtime action requires a
maintenance restart/documented Sandbox setup; next speech action requires working
normal interactive Computer Use access, then a known-label positive control and
complete-history observer. Last-phrase equality alone is not silence proof.

Parent reran the read-only servicing probe at18:51:00.8962723Z: Sandbox Disabled,
three existing dependencies Enabled, CBS/pending XML true, Update restart false,
no resolved launcher commands or observed Sandbox processes. Public REST still
reported CI43627997981 in_progress/check job incomplete at that check; no new
passing CI is asserted. Verified full CI43618587528 remains the source-attributed
baseline. Parent owns final integration/commit/push, reported after creation.

Final parent staged checks passed: `rtk git diff --cached --check`; Python audit
of144 Markdown files/411 relative links found no missing target. All24 task
Status/checklist lists,48 coverage rows and PRD remained unchanged, with no
production/example/build/package source diff. All nine staged paths belong to
this reviewed evidence integration. A narrow diff scan found zero private-key
headers, GitHub-token prefixes, AWS-key forms or otpauth URIs; this is not a
universal secret-detection guarantee. No further functional test is claimed.
