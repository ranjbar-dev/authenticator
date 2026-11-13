# Clean-source package integration

13 November 2026. Parent integrated the independently reviewed packaging as
`e8b92fd` and viewport icon as `ebd80e25794dccd699c9b3316914fe4af1b0c11b`,
both pushed to main. The first integration omitted the already-reviewed
src/main.rs staging; Git status exposed it, so it was committed separately before
the clean build. The intermediate dirty archive was replaced only after its
existing checksum/manifest validated. It is not the final artifact.

With Git status empty, `rtk proxy python scripts/package_windows.py --replace`
ran the default locked release build and passed package validation:

- `dist/Authenticator-0.1.0-windows-x86_64.zip`: 6,023,315 bytes.
- ZIP SHA256: `dcd9bdf4680553400ee598b7b76d6287293f5ab95335b50894b2d36057563ab4`.
- Manifest source: `ebd80e25794dccd699c9b3316914fe4af1b0c11b`;
  **source_dirty=false**; 364 archive members.
- Production executable SHA256:
  `99a2df81e1792bd6eda93ae2be0b16c757402e9b6661aa1aea102e382e6ec9df`.
  It is byte-identical to the executable used for the reviewed actual packaged
  native journeys. No new native behavior is inferred from the rebuild.

The adjacent checksum agrees. Public artifacts remain ignored local outputs.
New hosted check/package jobs are pending, not covered by the earlier source CI.
Environment/manual acceptance remains exactly as recorded in the implementation
and independent-review reports. Later documentation/audit commits may be newer
than this explicitly identified artifact source.
