# Independent review of the Narrator copy investigation

Reviewer `review05_sandbox` is distinct from author `resume19_speech` and the
original UI calibrator. Both are FINAL/inactive; no close-thread API is exposed.
**Scoped evidence integration approved after one P3 correction and focused
re-review. No material security, privacy or acceptance finding remains.**

Read the full Task19 screen checklist/evidence, Task21 acceptance, relevant PRD,
both new evidence files and all six tracked integration diffs. Read Computer Use
SKILL.md and full guidance. No production UI or new screenshot exists to verify;
historical rendering/pointer/keyboard work was not recertified. No Task19/21 or
global acceptance closes.

## Independent checks

- Current [Microsoft Narrator guidance](https://support.microsoft.com/en-us/accessibility/windows/narrator/complete-guide-to-narrator)
  confirms Ctrl+X copies the last phrase; Alt+X opens up to500-string Recap.
  Equal last-phrase samples cannot prove absence of intermediate announcements.
- Python hashlib matched the existing fixture:41,938,432 bytes,
  SHA2562c7adfe7a0d2d700662ff197cf8b2121d7a758311ca054138aa05336bf08f626.
  These identify bytes, not successful runtime or compiled fixture-mode identity.
- Python comparisons found24 task Status/checklist lists,48 coverage rows and
  PRD unchanged, with no production/example/build/package source diff. New JSON
  parses. Fixture source creates fresh task24-owned roots beneath canonicalized
  target, avoiding default-vault access.
- Read-only `Get-Process final_matrix,Narrator` returned no entries. This is a
  present observation, not retrospective proof of author launch outcomes.
- `rtk git diff --check` passed.

[GetCursorPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getcursorpos)
and [OpenInputDesktop](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-openinputdesktop)
document access prerequisites. The report correctly distinguishes the exec
caller's denied desktop handle from an unknown sky-helper cause and does not
infer Windows was locked. Launch errors, intervening inventory and earlier
caller diagnosis remain author-attributed; the reviewer did not reproduce them
or inspect independent runtime logs.

## Correction and limits

One P3: the author called its stopping decision a documented recovery limit,
although guidance gives a retry limit for capture/activation failures and requires
a targetable window, without a specific GetCursorPos launch limit. Parent changed
the report to the actual two failed normal launches/no-target decision, renamed
the JSON field to native_input_stopped_after_two_failed_launches and changed
future resume wording to target-selection/recovery guidance. Focused independent
inspection of both corrected files approved the fix.

The reviewer performed no native launch/input, clipboard/audio capture,
screenshot, positive control, rollover or speech test. Existing TITLE-weight
calibration proves no speech behavior. No new CI success is asserted. The
[author report](speech-copy-2026-11-14.md) retains exact remaining tests and limits.
