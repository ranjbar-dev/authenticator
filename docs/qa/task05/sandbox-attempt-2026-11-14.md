# Task 05: measured Sandbox provisioning attempt, 14 November 2026

**Sandbox is restored to Disabled, but Windows still reports queued servicing:**
CBS `RebootPending` and `C:/Windows/WinSxS/pending.xml` remain present. They were
absent before this attempt. Both enable and supported disable returned
`RestartNeeded=true`. No host restart occurred. This is a partial restoration of
feature state, **not** a claim that all host servicing effects were undone.

The sole implementation child `resume05_runtime` investigated the earliest open
Task 05 on source `8ac9f38d9ced3bbf148a31c14647b2e09da90f8f`. It made a bounded
servicing attempt after parent authorization within the user's dependency-install
scope. No production app source, requirement, task status or checkbox changed.
No app, vault, clipboard or network acceptance journey was run.

## New official evidence and decision

[Microsoft's Sandbox overview](https://learn.microsoft.com/en-us/windows/security/threat-protection/windows-sandbox/windows-sandbox-overview)
describes Sandbox as included with supported Pro/Enterprise/Education editions
and independent of a separately maintained VM installation. This gives a route
that does not depend on the unresolved retail/evaluation guest prerequisites in
the earlier [read-only report](resume-2026-11-14.md).

[Sandbox installation guidance](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-install)
directs enabling the optional feature and restarting if prompted. Current
[winapp Sandbox setup](https://learn.microsoft.com/en-us/windows/apps/dev-tools/winapp-cli/sandbox-execution)
also places restart before opening Sandbox. The local component payload was
available: Sandbox was Disabled, rather than DisabledWithPayloadRemoved; package
manifests were present, Hyper-V was already active and C: had about 80 GB free.
`RestartRequired=Possible` is feature metadata, not proof of restart-free enable.

The [enable cmdlet](https://learn.microsoft.com/en-us/powershell/module/dism/enable-windowsoptionalfeature?view=windowsserver2025-ps)
documents `NoRestart` as suppressing restart, and `LimitAccess` as avoiding
Windows Update for feature sources. The parent therefore authorized one exact
component enable with these options, without `All`. Existing dependencies were
already enabled. No host networking or policy change was needed.

## Actual commands and measured result

All shell invocations were prefixed `rtk`; servicing ran inside
`rtk proxy powershell -NoProfile -Command '<script>'`. The scripts recorded
before/after JSON and used the uniquely owned ignored directory
`target/task05-sandbox-d9830dfe1640408fa77306dc7a931d21` for scratch and logs.
The exact operative commands, with `$taskScratch` resolved to that directory,
were:

```powershell
Enable-WindowsOptionalFeature -Online -FeatureName Containers-DisposableClientVM -NoRestart -LimitAccess -ScratchDirectory $taskScratch -LogPath (Join-Path $taskScratch dism.log) -ErrorAction Stop
Disable-WindowsOptionalFeature -Online -FeatureName Containers-DisposableClientVM -NoRestart -ScratchDirectory $taskScratch -LogPath (Join-Path $taskScratch rollback-dism.log) -ErrorAction Stop
```

| Observation | Before | After enable | After supported rollback |
|---|---|---|---|
| UTC observation | 18:32:24.1048643 | 18:32:56.5471323 | 18:33:59.6504100 |
| Sandbox feature | Disabled | **Enabled**, not EnablePending | Disabled |
| Operation `RestartNeeded` | No operation | true | true |
| CBS restart marker | absent | present | present |
| Pending XML | absent | present | present |
| Windows Update restart marker | absent | not sampled | absent |
| Hyper-V/Hypervisor/VirtualMachinePlatform | Earlier baseline Enabled; Hypervisor freshly checked Enabled | not resampled | All freshly checked Enabled |

Both wrapper processes exited 0. Enable emitted the explicit warning that
restart was suppressed. The reported feature state was Enabled despite the
returned restart requirement. Following Microsoft's documented servicing
sequence, no pre-restart launch was attempted. This is a deliberate decision
to avoid using incomplete servicing; it is **not** an observed Sandbox launch
failure or a claim that a tool/policy forbids a normal launch.

The [supported disable cmdlet](https://learn.microsoft.com/en-us/powershell/module/dism/disable-windowsoptionalfeature?view=windowsserver2025-ps)
was then used on only the originally Disabled feature, with `NoRestart` and
without `Remove`. It restored that feature state but also returned a restart
requirement. No registry/pending XML deletion, package surgery, forced servicing,
restart, host lock/suspend/logoff, service/policy/network mutation or bypass was
used. OS-controlled servicing side effects are bounded by the observations
above, rather than asserted absent. Raw DISM logs stay ignored in the owned
scratch directory; [sanitized state evidence](sandbox-attempt-2026-11-14.json)
contains only prerequisite states and command attribution.

## Checks, acceptance and concrete resume

The new [read-only servicing snapshot](sandbox-servicing-state.ps1) reports
string-valued feature/restart states, the three pending indicators and launcher
availability. It never enables/disables/launches Sandbox or restarts Windows:

```powershell
rtk proxy powershell -NoProfile -File docs/qa/task05/sandbox-servicing-state.ps1
```

The snapshot exited 0 at `18:35:34.5476748Z`: Sandbox Disabled, all three existing
dependencies Enabled, CBS/pending XML true and Update restart false. Neither
`WindowsSandbox.exe` nor `wsb.exe` resolved through `Get-Command`; zero
`WindowsSandbox*` processes were observed. This checks command availability in
the current environment, not every possible install path. Sanitized JSON parsed
successfully, and `rtk git diff --check` exited 0 (new evidence files were still
untracked, so this is not a complete whitespace scan of them).

Task 05 offline first use and actual Windows session-lock/suspend acceptance
remain unperformed. No new screenshots or pointer/keyboard evidence exist for
this attempt, and earlier app/idle/injected-event results retain their original
scope. The shared isolated-runtime blockers in Tasks 08/19/20/21/23, plus
separate logon and heard-speech checks, remain open. No source regression suite
is warranted for this evidence-only change; actual servicing and the new
read-only snapshot are the relevant checks.

Next, during a separately scheduled host maintenance window, complete the queued
Windows servicing restart. The agent has not scheduled or authorized that
restart. Rerun the snapshot; then use the documented Sandbox enable command in
that maintenance context and complete any newly required restart/client setup.
Create a fresh Sandbox only after prerequisites are ready. Use a custom `.wsb`
with `Networking=Disable`, `ClipboardRedirection=Disable`, microphone/video/printer
redirection disabled, a read-only mapping of just the validated package and
disposable fixture tools, and a separate owned output mapping only if necessary.
[Microsoft's configuration reference](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-configure-using-wsb-file)
documents these controls; default Sandbox networking and clipboard sharing are
enabled, so a default launch is insufficient evidence of isolation.

Verify inside the guest that networking is effectively disabled, Rust is absent
and graphics/input work before Task 05 Create/unlock/wrong-phrase/relock/relaunch.
Then obtain independent review and proceed sequentially to the screenshot and
recovery checks. Sandbox readiness alone will not establish actual host/guest
sleep, session-lock/logon or screen-reader speech delivery. No independently
ready runtime was identified by this attempt; process-scoped speech observation
could be researched separately, but has not been tested here.
