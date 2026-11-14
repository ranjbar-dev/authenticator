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

Task05 and shared offline/OS/logon boundaries remain open. Parent next investigates
the independent documented Narrator copy route sequentially; earlier Speech Recap
UIA extraction failed its known-utterance positive control. No speech result or
project completion is implied. Parent owns commit/push; identity and remote
equality are reported after creation, avoiding circular attribution.
