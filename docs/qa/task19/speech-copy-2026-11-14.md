# Task19/21: Narrator copy observer investigation, 14 November 2026

Author `resume19_speech`, source cfffcc5c3da2002fdec4adb0dc9b689bf4b9f2fb.
**No speech positive control ran.** The native fixture launch prerequisite failed
before a target fixture or Narrator session was observed. This is a new bounded
observer investigation, not a repeated speech test or acceptance pass. Source,
task statuses, checkboxes and requirements are unchanged.

## Verified shortcut semantics

Current primary [Microsoft Narrator guide](https://support.microsoft.com/en-us/accessibility/windows/narrator/complete-guide-to-narrator)
and [Narrator basics](https://support.microsoft.com/en-us/accessibility/windows/narrator/chapter-2-narrator-basics)
document Narrator+Ctrl+X as copying **only the last spoken phrase**. Narrator+Alt+X
opens Speech Recap with up to 500 strings; standard reading/arrow/copy commands can
access its content. This offers a distinct possible route when UIA exposes only
history-window chrome, as in the [earlier failed observer](../task24/follow-up/README.md#narrator-observation-limit).
It does not establish that those commands work in the present environment.

A known static-label copy is needed first. Equality of two last-phrase samples
cannot prove that no intermediate OTP announcement occurred; command feedback
can itself change the most recent phrase. A full observed history or a validated
equivalent is needed for that claim. No raw speech history, transcript, clipboard
value or credential was collected in this attempt.

## Actual native attempt

Read Computer Use SKILL.md and its full guidance, confirmations and API before
native work. Initialized a fresh node_repl session with the documented
`@oai/sky` import, then `sky.list_apps()` succeeded. No current fixture or Narrator
window was in that inventory. Subsequent output was limited to target candidates.

The exact native launch call was:

```javascript
await sky.launch_app({
  app: 'D:\\Projects\\github\\authenticator\\target\\debug\\examples\\final_matrix.exe'
});
```

It returned `GetCursorPos failed: Access is denied. (0x80070005)`.
Fresh `sky.list_windows()` filtered for the fixture/Narrator/LockApp returned zero
matches. Read-only `Get-Process final_matrix,Narrator` found no process, and the
newest enumerated `task24-owned-*` root was historical, created 01:15:48 UTC.
One normal retry after that fresh observation returned the identical error.
Native input stopped after two failed normal launches and no targetable fixture window. No shell launch, alternate
input provider, session/desktop switch, security change or access bypass was used.

Python hashlib independently measured the existing fixture:
41,938,432 bytes, SHA256
`2c7adfe7a0d2d700662ff197cf8b2121d7a758311ca054138aa05336bf08f626`.
The source was inspected for argument-free fresh owned-root creation and default
vault avoidance. Its compiled fixture mode/source identity was not established
by runtime, and no successful launch or new root is inferred from the hash.

## Bounded read-only diagnosis

An exec Python ctypes call used only ProcessIdToSessionId, GetThreadDesktop,
OpenInputDesktop(0,false,DESKTOP_READOBJECTS), GetUserObjectInformationW(UOI_NAME)
and CloseDesktop; no UI Automation, input, desktop switch or policy mutation.
At 18:45:04.655461 UTC the **exec caller**, not the sky helper, was in session 1 on
desktop Default. OpenInputDesktop returned NULL/error 5. C: had 76,321,472,512 bytes free.
The [sanitized result](speech-copy-2026-11-14.json) retains these bounded facts.

[GetCursorPos documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getcursorpos)
describes window-station access/current-input-desktop prerequisites.
[OpenInputDesktop](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-openinputdesktop)
and [GetUserObjectInformationW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getuserobjectinformationw)
document the read-only handle/name observations. These prerequisites do not
diagnose this helper's exact failure. No LockApp match is not proof the desktop
was unlocked; neither a lock, a permission cause, nor a relationship to earlier
disk-related recovery is asserted. No attempt was made to enter a secure desktop.

One incidental enumeration command's appended `Get-Date -AsUTC` failed because
this shell lacks that parameter. The preceding process/root output was obtained;
the authoritative later UTC time came from Python datetime. No successful exit
is claimed for that incidental command as a whole.

## Outcome and resume

Narrator was never started, no clipboard action or screenshot occurred, and no
new fixture root or surviving fixture/Narrator process was observed. No owned
native resource requires cleanup. Existing windows, app data, OS scaling/contrast,
servicing, startup and audio settings were left untouched. This does not assert
the absence of every unrelated process or a successful normal-desktop state.

The next safe action is a later authorized execution with normal interactive
desktop access available to Computer Use. Refresh target inventory and attempt
one normal owned-fixture launch following documented target-selection and recovery guidance. Then test
the known-label copy positive control, followed by complete-history observation
of meaningful status changes and non-code-focus rollover. Process transcripts
only in memory and retain counts/booleans; guarded cleanup must preserve newer
clipboard content. If no validated history observer is available, announcement
behavior remains unverified. Do not reuse unchanged last-phrase equality as proof.

No source build or regression suite was needed for this evidence-only change.
JSON/whitespace/diff checks are recorded separately. Parent owns distinct review,
integration and any status reconciliation; no Task19/21 completion is claimed.
