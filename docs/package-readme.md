# Authenticator for Windows x64

Extract this folder and open `Authenticator.exe`. No Rust, Python, Node or app
registration is required for runtime. Keep the notices and documentation with it.

The distributable uses a static C runtime; its PE import check rejects an external
VC redistributable DLL. Windows system libraries and a usable graphics driver are
still required. This does not replace an actual clean-machine compatibility test.

Read **User Guide.md** for setup, screenshot scan, code/recovery management,
encrypted backup/restore and update/uninstall instructions. Read **Security
Limits.md** for the tested boundary and outstanding acceptance. This unsigned
preview does not claim full release or security certification.

`build-info.json` records the exact executable/dependency-lock checksums, compiler,
source commit and whether the source tree was dirty. A dirty build is a review
artifact, not an assertion that its files exactly equal that commit. ZIP SHA256
is distributed beside the archive. Repeating documented build steps is supported;
bit-for-bit identical builds are not claimed.

Local data stays in Windows' per-user LocalAppData `Authenticator` directory,
independent of this extracted folder. Deleting/updating this folder does not
delete the vault. Use encrypted `.authpbk` export/import for portable recovery;
the reference app's private `.vault` format is unsupported.

Source and task/evidence history: https://github.com/ranjbar-dev/authenticator
