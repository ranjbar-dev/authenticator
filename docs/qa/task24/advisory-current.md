# Current dependency advisory snapshot

Parent check, 2026-11-13T17:04:09.991Z. Installed cargo-audit 0.22.2 scanned the complete current Cargo.lock (385 dependencies), including Task23 build additions. Lock SHA256: 66ebe10679cb171d20d7da40a04b1afebfc54639fe65df9e61fe0608e41ab43d. No advisory IDs, target families or severity classes were ignored.

A fresh fetch used an owned D: database under target/task24-advisory-a1c0112b-6721-4b73-811c-c05e8fb58923/db. RustSec reported 1,509 advisories, revision 4ef2df8e806a65e2f1ecc14f56f51249062b23d6, last updated 2026-11-13T13:33:03+01:00. Direct Git HEAD inspection independently agreed. A following structured no-fetch audit against that database returned exit 0, vulnerabilities.found=false/count0 and three unmaintained warnings. No-fetch omits revision/date in its JSON; those null fields are not treated as a failed or fabricated fresh fetch. Raw structured report remains in the owned ignored root.

Commands used the installed executable through RTK (hidden child processes):

    rtk proxy C:/Users/root/.cargo/bin/cargo-audit.exe audit --file D:/Projects/github/authenticator/Cargo.lock --db <owned-root>/db --json
    rtk proxy C:/Users/root/.cargo/bin/cargo-audit.exe audit --file D:/Projects/github/authenticator/Cargo.lock --db <owned-root>/db --no-fetch --json
    rtk proxy git --git-dir <owned-root>/db/.git rev-parse HEAD

Warnings retained: zbus_names 4.3.5/RUSTSEC-2026-0371, zvariant 5.15.1/RUSTSEC-2026-0370 and zvariant_derive 5.15.1/RUSTSEC-2026-0372. The reviewed Windows normal/build package inventory contains none of these three names (independently read all 184 records in THIRD-PARTY-NOTICES.json). They remain visible all-target lockfile maintenance warnings; they are not silently suppressed or evidence of runtime exploitability.

This is a dated known-advisory check, not a guarantee of no vulnerabilities, malicious dependencies, unreachable-code safety or future advisories. Source/security reviews and required native/privacy gates remain separate. The initial hidden shell attempt produced no structured output and was not counted; the direct executable runs above supplied actual JSON and verified database state. Global Cargo/toolchain configuration and application data were not modified.

Primary method: [RustSec cargo-audit](https://github.com/rustsec/rustsec/tree/cargo-audit/v0.22.2/cargo-audit). Advisories: [0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html), [0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html), [0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html).
