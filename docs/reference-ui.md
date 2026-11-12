# Reference UI evidence

Inspection date: 12 November 2026. Repository baseline `f7ea0cc` had PRD.md, tasks/ and Git metadata only: **no original screenshots, reference assets, DESIGN.md, source or theme**.

Local non-destructive discovery (`Get-AppxPackage` name filter) found installed package `FirstOrder.AI.AuthenticatorApp`. The process/window-title inventory found no running reference-app window (the terminal titled with the project is not the reference). The package was **not launched**: the app's vault state is unknown, and opening it might expose real credentials. No package data/vault or credential contents were inspected; no sync/settings/import/export/write operation was performed. Installation is not evidence that safely viewable reference screenshots are available.

Use PRD.md section 5's observed inventory as the authorized fallback: dark narrow mobile-like window; 2FA Codes, Backup Codes and Settings; blue accents; searchable icon/service/account/code cards with remaining-time progress; screenshot-first Scan QR Code chooser; manual Account/advanced settings; account-linked recovery codes; folders, sorting, trash and local settings. Exclude profile/sign-in, sync/cloud, language, remote time changes, developer mode and companion promotions.

Exact reference typography, colors, dimensions, transitions, DPI behavior, hidden flows and pixel fidelity are **unverified**. Task 01's `src/ui/theme.rs` values are proposed choices. Do not cite the new app's own screenshots as reference evidence. Task 04 will extend the inventory language to shared shell components; task 21 must separately report available visual comparisons and unverified reference details.

Sanitized task 01 spike evidence (when recorded) lives under `docs/qa/task01/`. It depicts only inert test controls in the new app, no credentials and no reference-app contents. The foundation spike does not demonstrate a working capture/QR/vault workflow.
