# Authenticator implementation tasks

This is the implementation plan for [PRD.md](../PRD.md). Execute numbered tasks **01–24 in ascending order**, one implementation child at a time, followed by an independent review child after the implementer exits. All tasks are required for v1 completion; task 22 requires an evidence-based discovery result, not unproven reference-format compatibility.

No app implementation has been started by generating this plan. Every initial status is `todo`.

## Execution order

| Task | Work | Status |
|---|---|---|
| 01 | [Rust foundation, reference evidence and architecture](01-foundation-and-decisions.md) | done |
| 02 | [TOTP engine, data model and credential validation](02-totp-domain-and-uri-parser.md) | done |
| 03 | [Encrypted local persistence and key management](03-encrypted-local-vault.md) | done |
| 04 | [Reference-based Rust shell and shared controls](04-shared-ui-shell.md) | done |
| 05 | [Local vault setup, unlock and lock lifecycle](05-vault-setup-and-lock-screen.md) | in_progress; implementation reviewed, hardware/integration QA open |
| 06 | [Bounded local image and QR decoding](06-local-qr-decoding.md) | done |
| 07 | [Windows screen-region capture overlay](07-windows-region-capture.md) | in_progress; implementation reviewed, hardware QA open |
| 08 | [Complete screenshot-to-2FA workflow](08-screenshot-review-and-import.md) | in_progress; functional implementation approved, disconnected-machine QA open |
| 09 | [Manual creation and account editing](09-manual-account-editor.md) | todo |
| 10 | [Complete 2FA list, search, sort, views and copy](10-codes-list-and-clipboard.md) | todo |
| 11 | [Authenticated encrypted export/import engine](11-portable-backup-engine.md) | todo |
| 12 | [Easy encrypted backup export screen](12-export-backup-screen.md) | todo |
| 13 | [Easy backup import, preview and conflict decisions](13-import-backup-screen.md) | todo |
| 14 | [Backup Codes tab and account selection](14-backup-codes-list.md) | todo |
| 15 | [Add/edit backup codes from text and file](15-backup-codes-editor.md) | todo |
| 16 | [Reveal/copy, used-state and delete recovery codes](16-backup-codes-detail.md) | todo |
| 17 | [Folder management and filtered navigation](17-folders-and-organization.md) | todo |
| 18 | [Trash, restoration and permanent deletion](18-trash-and-restoration.md) | todo |
| 19 | [Settings, auto-start and contextual offline help](19-settings-startup-and-help.md) | todo |
| 20 | [Failure resilience and independent security audit](20-reliability-and-security-review.md) | todo |
| 21 | [Full offline workflow, accessibility and visual QA](21-complete-ui-and-workflow-qa.md) | todo |
| 22 | [Reference-format discovery and truthful compatibility](22-reference-migration-discovery.md) | todo |
| 23 | [Windows packaging, clean-install recovery and documentation](23-windows-packaging-and-user-docs.md) | todo |
| 24 | [Final completeness audit, PRD reconciliation and main push](24-final-prd-audit-and-main-push.md) | todo |

## Start in Codex CLI

Run from PowerShell with an authenticated Codex CLI:

```powershell
rtk proxy codex --enable multi_agent -a never -s danger-full-access -c agents.max_concurrent_threads_per_session=1 exec -C "D:\Projects\github\authenticator" "Read tasks/CODEX_PROMPT.md and execute its instructions."
```

This command uses the installed CLI's verified approval/sandbox/feature options. It disables approval prompts and allows local build/install/network operations. The explicit prompt requires sequential children even if a different runtime ignores the concurrency configuration. Do not change the user's global Codex configuration. If RTK is unavailable on another system, run the same Codex command directly.

The reusable full prompt is [CODEX_PROMPT.md](CODEX_PROMPT.md). It is for a future execution session; generating this plan does not start implementation.

## Settled defaults

PRD.md §12 is authoritative. Defaults: Windows-first Rust UI; evaluate egui/eframe, change only with documented feasibility evidence; local passphrase unlock; five-minute idle lock and lock on OS lock/suspend; thirty-second app-owned clipboard expiry; many-to-many folders; trash has no automatic expiry; PNG/JPEG QR image input; UTF-8 recovery-code input up to 1 MiB; native portable encrypted backup; reference-format support only after proof; English-only and no runtime network requirement.

Use established encryption libraries, authenticated encryption, a memory-hard KDF and an encrypted vault containing metadata/preferences as well as secrets. Document calibrated parameters during implementation instead of inventing production cryptographic guarantees in planning.

## Review and completion rules

- Each task file contains implementation steps, acceptance criteria, dependencies, and status/evidence instructions.
- All UI tasks carry their own screen checklist. Reuse task 04's theme/widgets. Parent review or a distinct sequential reviewer is required; authors cannot certify their own review.
- Read the actual source as implementation grows; this plan's baseline contains no existing UI conventions, component counts or measured design tokens.
- Use real pointer and keyboard testing. A passing unit test, app launch, HTTP response or absence of overflow is insufficient UI evidence.
- Record exact commands/results and sanitized evidence; do not commit real seeds, vaults, backups, passphrases or screen captures containing credentials.
- Update [coverage.md](coverage.md) per accepted task. Do not check off acceptance items without evidence. Do not delete this plan or lower the PRD scope to finish it.
- Parent owns integration, task statuses, coherent commits and pushes to `main`; child agents implement or review only.
- Runtime limits, missing credentials, unavailable hardware or mandatory policy cannot be overridden by a prompt. Persist a precise checkpoint, try safe alternatives, continue independent work and report any irreducible blocker honestly rather than inventing a pass.

## Planning evidence and handoff

Sized as a whole new application, ending as a handoff for another execution session. The repository contained only PRD.md plus Git metadata when planned. No source screens, shared theme, DESIGN.md or reference screenshots were present; no UI reuse counts or reference measurements are asserted. Task 01 owns non-destructive reference evidence gathering and framework feasibility; task 04 owns shared components and checker calibration.

The UI consistency finding-patterns, values, decisions and planning guidance informed the checklists. The user's explicit request settles repository placement in `tasks/`, retention of task files and autonomous decisions; no further planning approval is needed. Planning read one product document and performed one project file-inventory search (excluding Git and CLI checks); skill/support-file reads are approximately ten. App rendering and implementation validation are not yet performed.

Codex orchestration was checked against the installed CLI help/features and [official subagent documentation](https://learn.chatgpt.com/docs/agent-configuration/subagents). No fixed model is forced.

## Checkpoint

12 November 2026: Task08 functional implementation approved by distinct read-only review_08 with no findings requiring changes. Parent integrating now; next is one implementation child for09, followed by a distinct reviewer. Do not stop at this commit. Continue09–24, with all remaining mandatory features/gates retained. Runtime has no close-thread API; completed children are inactive. No child currently active after review_08 final.

Defining flow works: real pointer selected QR in another app → explicit multi-choice → masked editable review → whole-batch encrypted save → actual current codes independently matched using stdlib HMAC. Reviewer matched SHA1/6/30 at1794506687, and SHA512/8/120 plus SHA1/6/30 at1794506886. Failure/retry, duplicates preserved as new IDs, Help/Cancel/pickerLock/privacy and340/600/960 layouts passed. Exact commands/results/screenshots: docs/qa/task08/implementation-evidence.md and independent-review.md. Full76 tests before final minor UI refinements; later focused UI14/domain12/duplicate3/atomic regression and independent checks passed.

Last fully accepted task06 abab2c8/CI43254531108. Reviewed05 f02aa94/CI43250246850; reviewed07 21e7736/CI43268483695. All pushed and hosted Windows checks succeeded. Task08 reviewed source is next coherent commit; record its SHA/CI in the following checkpoint. Tasks05/07/08 remain in progress for unperformed gates: actual networking-disabled journey, physical150/200/mixed DPI/right monitor capture, real OS session lock/hardware suspend/hot-unplug/permission denial, Narrator/reference pixels. Task05 offline checkbox corrected to open; source-local behavior is not disconnected-machine proof. Task08 closes current capture/review/save/code cleanup integration, while future clipboard/recovery/backup integrations require their own tests. No real/default/reference vault was used.

For09 reuse src/ui/import_review.rs Editor/ImportReview and shared full-role components, single AppState store/worker and cancellable transaction. Implement real stable-ID editing, deliberate secret replacement, advanced validated selections, optional normalized recovery codes atomically with account creation, retained errors and explicit Save/Cancel; read full09 task/PRD. Existing examples/import_flow.rs and qr_batch_fixture.rs create fresh owned TempDirs; generated PNG input is original public fixture, never a captured screenshot. All own task08 review windows closed and generated PNG observed absent. Unrelated vault_lifecycle.exe isolated preview may still run; leave it and real profiles alone. Shell is elevated: normally launch own examples through documented UI helper; physical press_key for egui. Actual root font metrics differ from measured native19px GDI exception; preserve both. See calibration reports for their limited sensitivity claims.

Resume: Read tasks/CODEX_PROMPT.md and execute its instructions.
