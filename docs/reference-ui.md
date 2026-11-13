# Reference UI evidence

Inspection date: 12 November 2026. Repository baseline `f7ea0cc` had PRD.md, tasks/ and Git metadata only: **no original screenshots, reference assets, DESIGN.md, source or theme**.

Local non-destructive discovery (`Get-AppxPackage` name filter) found installed package `FirstOrder.AI.AuthenticatorApp`. The process/window-title inventory found no running reference-app window (the terminal titled with the project is not the reference). The package was **not launched**: the app's vault state is unknown, and opening it might expose real credentials. No package data/vault or credential contents were inspected; no sync/settings/import/export/write operation was performed. Installation is not evidence that safely viewable reference screenshots are available.

Use PRD.md section 5's observed inventory as the authorized fallback: dark narrow mobile-like window; 2FA Codes, Backup Codes and Settings; blue accents; searchable icon/service/account/code cards with remaining-time progress; screenshot-first Scan QR Code chooser; manual Account/advanced settings; account-linked recovery codes; folders, sorting, trash and local settings. Exclude profile/sign-in, sync/cloud, language, remote time changes, developer mode and companion promotions.

Exact reference typography, colors, dimensions, transitions, DPI behavior, hidden flows and pixel fidelity are **unverified**. Task 01's `src/ui/theme.rs` values are proposed choices. Do not cite the new app's own screenshots as reference evidence. Task 04 will extend the inventory language to shared shell components; task 21 must separately report available visual comparisons and unverified reference details.

## Task22 public reference discovery, 13 November 2026

Read-only package/manifest metadata reconfirmed version3.2.0.0, the title above
and publisher display name Shanghai FirstOrder Technology Co., Ltd. The
[official product website](https://authenticator.firstorder.ai/) links
[Windows Store product9n6gl0bvkphn](https://apps.microsoft.com/detail/9n6gl0bvkphn)
and now exposes four public marketing screenshots in its “See It In Action”
section. The earlier absence statement applies to repository/local evidence at
that earlier inspection; it must not be read as a current claim that no public
reference images are accessible.

- [Marketing image1](https://s3.ap-southeast-1.amazonaws.com/firstorder.official.website.cms/uploads/auth_screenshot_1_c8231ec6f8.webp)
- [Marketing image2](https://s3.ap-southeast-1.amazonaws.com/firstorder.official.website.cms/uploads/auth_screenshot_2_104a4b6ff4.webp)
- [Marketing image3](https://s3.ap-southeast-1.amazonaws.com/firstorder.official.website.cms/uploads/auth_screenshot_3_942256c239.webp)
- [Marketing image4](https://s3.ap-southeast-1.amazonaws.com/firstorder.official.website.cms/uploads/auth_screenshot_4_b8c44bf73a.webp)

Task22 recorded these official link targets without downloading or rendering
their pixels. Their depicted platform, app version and relevance to the inspected
Windows3.2.0.0 UI are **not established here**. They do not by themselves prove
pixel fidelity, export compatibility or any security property. Separate visual
inspection can use these public references without opening an unknown user vault;
excluded cloud/account/companion features remain excluded regardless of marketing.

The bounded format investigation and unsupported `.vault` decision are in
[migration-compatibility.md](migration-compatibility.md).

### Parent inspection of public marketing pixels

The parent fetched all four exact linked images into memory on13November2026;
each returnedHTTP200/WebP. No raw marketing image was written to the repository.
Images1/2 depict an iPhone-framed light/dark Codes grid: service/identifier above
local-looking service icons and grouped codes, thin blue progress, folder tiles,
search/organization controls, help, floating add and three bottom tabs. Image3
depicts iOS home-screen widgets; image4 depicts iCloud/Google Cloud synchronization.
Those widget/cloud features remain excluded. None establishes private-file format
or cryptographic interoperability.

The dark image adds usable palette/hierarchy evidence, but does not establish the
installed Windows version's layout, fonts or DPI. The present Windows build has
header tabs, explicit desktop Copy/Edit/Delete actions, outlined controls and a
larger direct Screenshot to Scan action; it must not be described as a pixel clone
of this iPhone grid. Exact Windows/reference parity remains open. PRD§5 and the
documented desktop accessibility/security choices remain authoritative where the
public phone pictures do not answer the Windows brief. A native, version-matched
comparison cannot be replaced by the new app's own screenshots.

Sanitized task 01 spike evidence (when recorded) lives under `docs/qa/task01/`. It depicts only inert test controls in the new app, no credentials and no reference-app contents. The foundation spike does not demonstrate a working capture/QR/vault workflow.
