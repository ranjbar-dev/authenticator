# Task21 interrupted QA checkpoint

13 November 2026. **The user stopped Computer Use with physical Escape. All UI interaction stopped immediately; do not resume it in this turn or terminate the open app to bypass the stop.** This is a checkpoint, not Task21 acceptance or independent review.

Source baseline committed/pushed: `4e0f715963a9dceefb33cdef948f47fcf3a7a679`. Hosted [CI43397135687](https://github.com/ranjbar-dev/authenticator/actions/runs/43397135687) passed formatting/lint,205library+1main+18other ordinary tests and the Windows app build. That validates source through20, not the new21 harness or incomplete live journey. Task22–24 have not run.

## Current native outcome

The initial documented Node/Sky initialization failed with C: disk-full error112. Parent cache recovery is recorded in [environment-recovery.md](environment-recovery.md). Fresh initialization/window inventory subsequently worked. The current-source isolated `workflow_qa.exe` launched normally, exposing Create vault with masked fields. No default/reference vault or unrelated window was captured.

Passphrase entry did not populate the fields. A later own-app observation showed a parent CLI terminal covering the app, and input reported user activity. Input stopped; the parent held all CLI/UI commands for an exclusive native phase. The next fresh own-window activation reported: `Computer Use was stopped by the user with the physical Escape key`. The child made no further UI call, retry, close, typing or cleanup.

**No successful native vault creation, screenshot capture, review/import, current-code verification, Copy, export/import, folder or recovery journey was observed.** Successful launch/one clean setup frame does not close those gates. Earlier cursor-denial causes remain unknown; disk recovery is not proof of their cause.

## Source and attributed checks

`examples/workflow_qa.rs` is a new uncommitted development-only clean QA harness: its own TempDir under D: target, actual `with_repository(cc)` native HWND, no production default profile, and F10 OTP masking before PNG output. It still needs parent source inspection and distinct review; do not ship or call it accepted from this checkpoint.

Author reported `rtk cargo build --offline --locked --example workflow_qa --example qr_fixture` passed5.13s after restoring archive availability through the cache link. Five focused current framework checks passed: lifecycle9.31s, review privacy5.80s, codes/clock5.51s, all24 account create/edit/settings/reopen37.07s, six sorts/views/preferences/restart15.84s. These are synthetic framework/storage evidence, not native journey passes. Exact wrapper/manifests/logs remain in the owned target roots; no final independent21 review occurred.

## Preserve resources for a later authorized resume

- Open owned app PID44548, executable `D:/Projects/github/authenticator/target/debug/examples/workflow_qa.exe`; reported start13November2026 at12:40:27 local. Re-verify identity/start/path before any future action; do not trust a reused PID.
- Owned root `target/task21-owned-8lrp7u`, including `ownership.txt` and `sanitized-frame-001.png/.ready`. The sole saved frame was reported as a clean setup frame; the parent has not independently inspected it. Do not confuse it with a completed workflow screenshot.
- Six completed test roots `target/task21-runtime-*`, with manifests/logs. Reported PIDs16360,42668,44604,47232,47376,35288; fresh absence was not checked after the stop. Future cleanup must verify exact manifests, current process absence, resolved containment/non-links and use native PowerShell literal paths. No broad Temp scan or speculative deletion.
- Wrapper `target/task21-run.py`; public Cargo archives now live at the exact D: cache documented in environment-recovery.md.
- Older retained30688/32032 and unrelated lifecycle preview44740 remain untouched.

The parent is persisting this checkpoint only, without further CLI/Computer Use, commit or cleanup after the stop. Last pushed SHA is4e0f715; working changes include Task21 harness plus parent CI/environment/status notes. Those changes are not asserted committed or clean. On a later explicit resume, read this checkpoint, inspect Git/source and fresh owned resources, continue21 with exclusive native foreground access, obtain distinct review, then22discovery/23packaging/24final audit. Native/offline/hardware/Narrator/interactive-clipboard/package gates remain open; no full completion claim.
