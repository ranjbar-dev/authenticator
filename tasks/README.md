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
| 09 | [Manual creation and account editing](09-manual-account-editor.md) | done |
| 10 | [Complete 2FA list, search, sort, views and copy](10-codes-list-and-clipboard.md) | in_progress; reviewed source accepted, live Copy gate open |
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

13 November 2026 local: no active child. Distinct review_10 and focused re-review have emitted FINAL; P2/P3 resolved, reviewed Task10 source accepted. Task10 remains in_progress for actual interactive Copy success/expiry/Lock/exit. Parent records evidence/status/coverage/PRD, commits/pushes reviewed coherent work, then starts exactly one implement_11 for independent portable backup engine. Do not stop at this task/commit; continue11-24 and revisit open manual gates. Runtime exposes no close-thread API; completed children are inactive.

Last accepted/pushed main: **78566d4367e826e5ebee48d9b68bd617d56bf374** (Task09); hosted Windows CI43295295198 succeeded. Current Task10 source/evidence and this checkpoint are UNCOMMITTED. Tasks01-04,06,09 accepted;05/07/08 reviewed functional implementations retain explicit open gates. Task09 detailed evidence/reviews: docs/qa/task09/implementation-evidence.md and independent-review.md. No real/default/reference vault has been opened.

Review10 original FINAL found P2 Codes/Backup Codes shared-query desynchronization and P3 stale invalid-query feedback. Parent fixed separate route buffers/widget IDs, both cleared on Lock, and derived inline query errors without overwriting unrelated notices. New TextEdit paste/select-all route/correction/lock/encrypted-reopen regression passed1/10.42s. Full corrected suite104 passed,3 subprocess helpers ignored,14 suites62.84s; fmt/strict Clippy/diff and five binary/example target build passed. Focused independent re-review FINAL passed route-search1/8.89s, both-query lock1/7.79s and diff; no new finding. Parent may integrate and continue11. Original reviewer launch was blocked twice by GetCursorPos Access denied0x80070005, no owned window; independent real native isolated clipboard5+parent1, schema1 migration1, encrypted sorts1, Trash1, codes2, serial privacy7, components7 and fresh120/120 Python HMAC passed. No additional source security/data-loss finding established. Original author live Copy Changed gate remains open and must not be attributed to that separate reviewer launch denial.

Task10 author final:103 tests passed,3 subprocess helper entrypoints ignored directly but exercised by parents,14 suites66.12s; focused clipboard5/frame1/components7/serial capture-privacy7; fmt, strict all-target/all-feature Clippy, build and diff checks passed. Includes real list/grid/Unicode search/six sorts/All-Unfiled-Folder filters, generic asynchronous preferences and Trash, schema2 encrypted query/filter with frozen genuine schema1 migration, guarded clipboard actor, hidden stale-event/one-shot AX scrub, nonblocking held-save Lock, close flush/error/retry/discard/approved-exit and stable clipboard-footer focus. All8 native sanitized PNGs in docs/qa/task10/images are complete; parent independently verified CRCs/IEND/dimensions and viewed340/600/960, editor/delete/locked frames. Author actual pointer/key edit/discard, Delete Keep/confirmed encrypted Trash2-to-1, wheel/grid and normal exit/relaunch restored query public/Earliest/Grid/count1. Independent author HMAC120 helper cases and2 live values matched; JSON stores booleans/time only. Full author handoff: docs/qa/task10/implementation-evidence.md.

**OPEN core Copy gate**: final live pointer AND keyboard Copy returns Changed at fresh post-CloseClipboard HWND/marker/content verification. docs/qa/task10/copy-diagnostic.json confirms unlocked/listener_ok/listener_healthy/owner_nonzero/last_copy_queued true. Exact failing ownership subcondition/cause unknown. Do not attribute to a clipboard monitor, permissions or OS event without evidence. Native private-window-station/helper actor/message-pump and ownership/race tests PASS but do NOT replace interactive success/expiry/Lock/exit proof. Checks were not weakened; no unrelated clipboard text was read. Distinct reviewer must reproduce/source-audit this finding and other security/data-loss/UI acceptance before parent integrates. Keep task10 in_progress while unresolved. Continue independent later work only after reviewer FINAL and sequential findings handling; never silently mark the gate done.

Author owned previews/profiles/locator/scripts are absent. Unrelated vault_lifecycle.exe preview may still run; never touch it or real/default/reference profiles. For live UI use documented computer-use skill and @oai/sky through deferred node_repl tools, fresh returned own window IDs, normal medium sky.launch_app of argument-free codes_list.exe (NOT production Authenticator.exe). Shell is elevated; Start-Process cannot be input by the medium helper. Physical press_key works; egui Unicode type_text is inert. Native monitor-crop/CreateForMonitor/cursor permission errors are not bypassed. F6/F7/F8 resize340/600/960 logical; F10 actual native framebuffer redacts OTP bounds before disk (wait for complete PNG); F11 writes booleans only; F9 guarded owned retention must be cleaned. Scale2 frames do not certify physical DPI. Full source paths, exact commands and reproduction are in task10 evidence and reviewer brief.

Preserve Task09 cancellation: atomic REQUESTED/ADMITTED admission never blocks UI on filesystem mutex; background serialization only. UI/AX/query/history clears before a held admitted save is released. A complete post-admission encrypted commit may finish; no false rollback or late session adoption. Active worker keys/buffers may remain until I/O/crypto returns. Deliberate Reveal may expose AccessKit TextRuns despite PasswordInput; concealed/reconcealed/locked nodes must not contain raw values and password CopyText remains blocked. No complete memory-erasure claim.

Next: parent commits/pushes the reviewed Task10 source with explicit incomplete Copy acceptance and verifies CI. Then one implement_11 for portable encrypted backup engine, followed by distinct focused security/data-loss review; continue12-24. Native backup must have a distinct format/independent keys/passphrase, no plaintext export, bounded authenticated parse, atomic export/import and explicit conflict/merge/replace policy. Do not reuse local directory ACL protection on arbitrary destination parent folders. Preserve local startup/security policy unless explicitly preview-selected. Task17 must reuse actual schema2 selected-folder migration rather than assume schema1 or invent a second unnecessary migration. Later edits/imports must accommodate metadata timestamps after system-time correction without changing OS time or weakening validation;10 currently clamps only preference/Trash mutations.

Open later gates: actual disconnected-machine journey; physical150/200/mixed DPI/right-monitor capture; actual OS lock/hardware suspend/hot-unplug/permission delivery; Narrator/reference pixels; future backup/recovery integration. Task05 offline checkbox is correctly open; source-local operation is not adapter-off proof. Primary/negative-monitor real pointer capture and private-WTS tests are recorded, not hardware substitutes. Hidden/minimized contract tests are not full native hardware proof. No requirement is waived. Task24 only after prior outcomes evidenced; do not claim full completion with mandatory gates open.

Resume: Read tasks/CODEX_PROMPT.md and execute its instructions.
