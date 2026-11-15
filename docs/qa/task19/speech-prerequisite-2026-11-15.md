# Task 19/21 speech prerequisite, 15 November 2026

Author `resume19_observer`, baseline source
`996b56967051f5c6f6e7ac9346e738c6d5505282`. **No speech test ran and no
acceptance gate closed.** This later execution tried one normal fixture launch
after the [separate current prerequisite recheck](../task05/resume-2026-11-15.md).
Production source, requirements, task statuses and checkboxes are unchanged.

Read the complete Task 19/21 checklists, PRD section 7, prior
[speech-observer investigation](speech-copy-2026-11-14.md),
[Recap limitation](../task24/follow-up/README.md#narrator-observation-limit) and
Computer Use SKILL.md plus its complete guidance/API/confirmations. Inspected
`examples/final_matrix.rs`: its argument-free factory creates a fresh owned
`target/task24-owned-*` fixture; it does not resolve the production vault.
No source fix or successful compiled fixture mode is inferred.

## Actual attempt

Initialized a fresh `@oai/sky` session through `node_repl` as documented.
At `18:44:49.236Z`, `sky.list_apps()` returned no app identifier matching
`final_matrix`, `Narrator` or `LockApp`. Only matching candidate metadata/counts
were emitted; unrelated window titles were not emitted or persisted.

The sole normal launch attempt at `2026-11-15T18:44:54.506Z` was:

```javascript
await sky.launch_app({
  app: 'D:\\Projects\\github\\authenticator\\target\\debug\\examples\\final_matrix.exe'
});
```

It returned `GetCursorPos failed: Access is denied. (0x80070005)`.
Fresh `sky.list_windows()` at `18:45:02.975Z` returned zero matching
fixture/Narrator/LockApp app identifiers. A separate read-only
`rtk proxy powershell -NoProfile -Command` process/root inventory found no
`final_matrix` or `Narrator` process; the newest enumerated `task24-owned-*`
directory remained the historical `task24-owned-oGaL6h`, created
14 November at 01:15:48 UTC. No fresh fixture resource was observed.

`rtk proxy python -c` using pathlib/hashlib measured the existing executable:
41,938,432 bytes, SHA256
`2c7adfe7a0d2d700662ff197cf8b2121d7a758311ca054138aa05336bf08f626`.
This equals the earlier measured file identity; it establishes neither a
successful launch nor its compile-time fixture mode.

## Outcome and resume

Stopped after the one failed normal launch; no shell-launch/input-provider
fallback, desktop switch, security change or access bypass was attempted.
The exec-caller desktop-read success in the Task 05 report does not establish
helper cursor access or diagnose the helper's exact failure. No LockApp match
does not prove an unlocked desktop. No locked-state cause is asserted.

Narrator was never started. No known-label positive control, speech-history
observation, rollover, screenshot, accessibility inspection, app input or
clipboard action occurred. No new native ownership required cleanup; existing
windows/settings and the shared host were left untouched. No source regression
suite was repeated for this documentation-only addition.

Resume when normal interactive helper launch/input works: use a disposable owned
fixture, validate a known static-label positive control, then inspect complete
speech history or an equivalent validated observer across meaningful status
changes and a non-code-focus rollover. Retain only counts/booleans, handle any
clipboard cleanup with ownership guards, and restore only owned resources.
Copying only Narrator's last phrase cannot prove absence of intermediate OTP
announcements. Offline, actual OS lifecycle/logon and heard-speech requirements
remain open. Parent owns distinct review and integration.
