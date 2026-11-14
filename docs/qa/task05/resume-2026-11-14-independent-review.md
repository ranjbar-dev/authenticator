# Independent review of resumed Task05 evidence

Reviewer: `review_resume05`, distinct from `resume05_acceptance` and the parent.
Reviewed 14 November 2026 against HEAD
`aebf31a6c9c8f6d02806f9d40f350face836ab46` and the uncommitted evidence update.

**Outcome: approve this bounded documentation/probe update. No material security,
data-loss or acceptance finding. Task05 and the other seven blocked scopes remain
incomplete.** This is not approval of the full product or a new native UI pass.

## Scope and finding

Read the full Task05, including its screen checklist and original unchecked
offline/OS lifecycle acceptance, relevant PRD locking/local-only/security/default
requirements, current task index/coverage/completion report and reference/shared UI
guidance. Inspected the five tracked documentation diffs and the three new author/
parent report/probe files. No production UI, domain, storage, capture, startup,
clipboard or build/package source changed. No UI skill or screenshot check was
needed for these documentation-only changes; existing native evidence was not
recertified.

One P3 wording finding was corrected and rechecked before this final review:
Task05's new evidence paragraph called provisioning prerequisites absent. The
actual inspection establishes unresolved prerequisites and no ready guest/media
in specific locations. The parent adopted that narrower wording. No requirement,
status or checkbox was changed to resolve it.

The probe uses read-only Get/Test/enumeration APIs and emits machine capabilities,
counts/statuses and known default paths. It does not read vaults, credentials,
clipboard contents, screen pixels, media contents or account data; does not change
features/firewall/services/guests; and does not cause OS lock/suspend/logon. Its
absence claims are explicitly bounded. Firewall Enabled flags and sound-device
OK states are correctly excluded as offline or heard-speech proof.

## Independent checks

- `rtk proxy powershell -NoProfile -File docs/qa/task05/resume-2026-11-14-probe.ps1`:
  exit0, 5.068 seconds, inventory timestamp `2026-11-14T16:55:41.7363462Z`.
  Independently observed Hyper-V/VirtualMachinePlatform Enabled; hns/vmcompute/vmms
  Running; zero registered VMs; Sandbox Disabled; 16 logical CPUs and
  34,189,406,208-byte memory capacity; zero inspected top-level media; three named
  alternative executable paths absent; three firewall profile Enabled flags 1;
  four sound statuses OK; Narrator absent. Disk free space is time-dependent.
- A separate strict `rtk proxy powershell -NoProfile -Command` enumeration used
  `Get-ChildItem -LiteralPath $r -File -ErrorAction Stop` for both default Hyper-V
  roots and `C:/Users/root/Downloads`, filtering `.iso/.vhd/.vhdx/.ova/.ovf`.
  Exit0; all three counts zero. Thus the observed zero counts were not a suppressed
  access-error result. No recursive or whole-disk absence claim was made.
- Public REST GETs to
  `https://api.github.com/repos/ranjbar-dev/authenticator/actions/runs/43548413816`
  and its `/jobs` endpoint, executed using `rtk proxy python -c` with
  `urllib.request.urlopen`/`json.load`, confirmed completed/success at exact
  `aebf31a6c9c8f6d02806f9d40f350face836ab46`. Both check/package jobs and every
  listed step succeeded. Logs/test counts were not independently downloaded.
- Captured Git subprocesses inside that Python command independently confirmed
  local HEAD equals remote `refs/heads/main`; `git diff --name-only 079c08d HEAD --
  src Cargo.toml Cargo.lock build.rs scripts assets .github` returned empty.
- Python hashlib independently confirmed local ZIP SHA256
  `9c8b781018c3e6b66d06cbe3f5d5a5184b620f0d9925ce738e941b953badbcc6`.
- `rtk proxy python scripts/package_windows.py --validate dist/Authenticator-0.1.0-windows-x86_64.zip`:
  exit0, paths/CRCs/manifest checksums passed. This validates the existing079c08d
  artifact; it does not attribute a new build or runtime pass to aebf31a.
- Independent Python comparisons against `git show HEAD:<path>` found PRD text
  unchanged, all24 task Status/checklist line lists unchanged and all48 coverage
  data rows exactly unchanged. The parent separately reports its own link audit.
- `rtk git diff --check`: exit0 before review and after the P3 wording correction.
  Final `Select-String` inspection confirmed the corrected Task05 paragraph.

Parent package17/fmt results remain accurately attributed parent evidence. No
source regression suite was required or newly claimed by this evidence-only review.

## Official-source check and remaining boundary

Independent current browsing confirmed Microsoft's
[evaluation page](https://www.microsoft.com/en-us/evalcenter/download-windows-11-enterprise)
offers ISO links while stating registration and Microsoft-account installation
prerequisites. The [former developer VM URL](https://developer.microsoft.com/en-us/windows/downloads/virtual-machines/)
redirected to a generic developer-tools page. The
[retail media page](https://www.microsoft.com/en-us/software-download/windows11)
describes licensing prerequisites; it does not establish this user's guest
entitlement. [Sandbox instructions](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-install)
describe optional-feature enablement and a restart if prompted. No registration,
authentication, download, licensing workaround or host mutation was performed.

These observations support the qualified provisioning report. They do not prove
virtualization impossible or exhaust every future provisioning route. A ready,
appropriately licensed/evaluated isolated interactive Windows environment is the
concrete next step. Effective offline first use, no-Rust/other-user recovery,
actual OS lifecycle/logon and positive-control heard Narrator checks remain
unperformed. Application Lock, injected messages, AX, CI and available Hyper-V
resources cannot close those checks. Parent owns subsequent integration/status,
commit/push and final Git equality; this review changes only this report.
