# Task 06 implementation evidence

Implementation ready for a separate focused parser/privacy/security review;
not self-certified or marked accepted. Task02 strict parser/domain and task03
transaction/encrypted storage were read with their accepted evidence. Task05
hardware and future integration QA remain open. The parent owns task statuses,
coverage and commits. No UI change, screenshot, real credential, default-profile
read, actual capture or vault mutation occurs here.

Changed paths: Cargo.toml/Cargo.lock, src/lib.rs, src/qr.rs, src/qr/tests.rs,
docs/architecture.md and this evidence file. The parent's tasks/README.md
checkpoint was preserved. Actual security design is docs/security-design.md;
the requested docs/architecture/security-design.md does not exist. Full task06,
PRD requirements/security/settled decisions, task README, domain parser/records/
OTP/errors and storage APIs informed the implementation.

The API, exact formats/layouts/limits, selection behavior, source links and
dependency clearing constraints are documented in architecture.md. Pinning adds
image0.25.10 png/jpeg only, quircs0.10.3, and dev-only qrcode0.14.1. New resolved
codec versions are png0.18.1 and zune-jpeg0.5.15/zune-core0.5.3. No network
dependency is used at runtime. All test QR images are generated in memory from
published RFC6238 seed data or deliberately invalid/disposable bytes; no fixture
image is committed. Generic randomized temp files are fixture originals only;
the app opens them read-only, never changes/copies/writes them.

## Checks and evidence

| Exact command | Outcome |
|---|---|
| `rtk cargo test --locked qr:: --lib` | Final rerun15 passed,31 filtered,1.08s after path preflight. |
| `rtk cargo test --locked --test totp_domain --test foundation` | Final rerun15 passed across2 suites,0.02s. |
| `rtk cargo test --locked --all-targets -- --test-threads=1` | 60 passed, 2 ignored helpers invoked by parent tests, 45 filtered subprocess entries; 8 suites, 116.73s. Performed once before the final read-only path-metadata preflight; no storage/KDF code changed. |
| `rtk cargo build --locked --bin Authenticator` | Passed. |
| `rtk cargo clippy --locked --all-targets --all-features -- -D warnings` | Passed, no issues. |
| `rtk cargo fmt --all -- --check` | Passed. |
| `rtk git diff --check` | Passed. |
| `rtk cargo audit --json` | Exit0, 377 lockfile dependencies, zero known vulnerabilities; 3 existing Unix accessibility unmaintained warnings (zbus_names, zvariant, zvariant_derive). RustSec database commit af7b3ec33460b8e2de8d422e16d628518891a519, updated2026-11-12. |
| `rtk cargo tree -e features -i image` | Only png/jpeg enabled; no default formats/rayon. |
| `rtk cargo tree --target all -i zbus_names` | Notices originate in existing accesskit_unix/atspi/eframe tree. |
| `rtk rg -n 'write\|create\|copy\|rename\|println\|dbg!\|log::\|tracing\|reqwest\|http\|telemetry' src/qr.rs src/qr/tests.rs` | Inspected results: production only fixed fmt::write_str and documentation; fixture writes/encoders live under cfg(test). No production write/log/network call. |

One complete KDF-containing regression was run for integration confidence;
focused tests are fast and do not repeat storage mutation/KDF cycles. Final
path metadata preflight retains the handle metadata and cap+1 read checks;
focused QR/domain tests and build/Clippy/format passed again after that change.

## Meaningful acceptance coverage

- `png_jpeg_and_pixels_preserve_every_algorithm_digits_period`: all 24
  algorithm/digits/period combinations encoded as generated PNG and JPEG and
  in-memory pixels; service, account, decoded seed and supplied settings agree.
- `capture_layouts_rotation_and_deterministic_noise_decode`: all four packed
  pixel layouts plus90/180/270-degree rotations with deterministic low-amplitude
  sensor noise preserve validated candidates.
- `transparent_png_and_sixteen_bit_png_keep_qr_contrast`: transparent black
  background composites to white, opaque modules remain black;16-bit grayscale
  QR also decodes. Explicit color weighting/alpha convention checked.
- `mixed_codes_preserve_safe_outcomes_and_require_explicit_choice`: two valid,
  one unsupported HTTP QR and one invalid-secret QR remain separate outcomes;
  supported count/choice required and ownership transfer asserted.
- `damaged_qr_has_a_safe_decode_failure_candidate`: data/ECC damaged while
  finders remain, yielding fixed per-code DecodeFailure rather than NoQr.
- `actual_qr_payloads_are_strict_utf8_control_and_length_bounded`: actual
  generated QR carries invalid UTF-8, NUL and2049-byte payload; all invalid.
  Exact2048-byte classification boundary checked separately.
- `qr_boundary_keeps_task02_strict_parser_policy`: rejects unknown parameter,
  issuer conflict, duplicate issuer, incompatible algorithm/period; HOTP,
  HTTP and proprietary migration are unsupported. Existing strict parser suite
  separately covers the full URI ambiguity/percent/control/Base32 matrix.
- `saturated_detector_rejects_before_any_payload_decode_or_partial_review`:
  actual8-code scan fails whole scan, actual7-code scan returns7 supported drafts.
- `no_qr_corrupt_and_unsupported_images_have_distinct_fixed_errors`: blank,
  GIF, truncated PNG and broken JPEG paths remain distinct and safe.
- `file_input_sniffs_content_preserves_originals_and_writes_no_artifacts`:
  PNG/JPEG stored under generic wrong-extension temp names decode; their bytes
  and directory entries are unchanged after service calls. Invalid source also
  remains byte-identical; directory input rejects. Temp fixture owners clean up.
- `metadata_and_growth_race_reads_are_bounded`: sparse oversized original
  rejected via metadata; continuous reader simulates growth after metadata and
  stops atcap+1; short exact input preserved.
- `adversarial_geometry_segments_and_pixel_allocations_fail_before_decode`:
  zero, u32max,4097edge and pixel-product overflow/overcap headers, JPEG65535
  dimensions and malformed segment lengths, wrong packed buffer size and encoded
  file cap reject without decoded-pixel allocation.
- `png_hostile_ancillary_length_is_bounded_before_body_allocation`: a valid
  small IHDR followed by a forged2GiB ancillary chunk length fails under decoder
  limits without trusting it for an app allocation.
- `deterministic_adversarial_bytes_and_noise_never_panic`:256 deterministic
  malformed buffers including PNG/JPEG prefixes plus checker/stripe/random
  hostile pixel patterns complete with no escaped panic or validated credential.
- `redacted_aggregate_debug_and_fixed_errors_never_include_metadata_or_source`:
  review/candidate/draft formatting excludes service, account, seed and raw URI.

Privacy evidence combines isolated filesystem assertions, explicit read-only
production API inspection and redacted Debug/error assertions. No runtime log
subscriber, telemetry, remote lookup, clipboard write or artifact writer is
introduced. This is not whole-machine filesystem tracing or a memory-forensics
claim. Build/test artifacts and reviewed source fixtures are expected; no raw
capture/image/payload file is emitted by the service. Codec/dependency-owned
encoded/scratch/error-path payload copies are not guaranteed zeroized; see the
specific limitations in architecture.md.

## Corrections and remaining review gates

First compile used a private quircs extract method; changed to its public bounded
iterator. First focused run was11pass/1fail: a two-row7-code synthetic mosaic
produced cross-row finder grouping (4 valid plus failed grids); the deterministic
limit fixture now places codes in a single row and all7 decode. The actual8-code
saturation rejection passed already. Formatting corrections were applied before
final verification. No requirement was waived.

Separate review should inspect pre-decoder limits including PNG metadata,
bounded regular-file reads, candidate saturation/collection before payload
decoding, strict parser/error mapping, raw buffer lifetime and dependency-copy
limitations. PNG/JPEG memory limits are not a hard total-memory/time sandbox;
catch_unwind is a fixed-error defense for upstream assertions, with the app's
existing nonprinting panic hook. Tests establish targeted bounded behavior,
not a claim of panic freedom for every possible upstream input.

Task07 must implement actual selected capture, normalize GDI/padded pixels and
bound allocation before acquisition; task08 must enforce unlocked state,
generation/cancel/lock discard, explicit selection/editable review and approved
atomic save. These integration/hardware checks are deliberately unperformed in
task06. No UI screenshot or real pointer action is relevant to this pure service.

## Parent acceptance

Distinctreview_06 approved puredecoder scope withoutblocking parser/privacy/security findings. Independentlypassed15QR tests,15domain/foundation tests,lockedallfeatureClippy/fmt/diff andfeaturetree; inspectedpinneddependencyallocation/saturation/clearing paths andsuppliedfullregression/audit evidence. Parentaccepted06. Actualcapture/cancellation/lock/review/save remain07/08; best-effortmemorybudget/dependencycopy caveats remainexplicit. ImplementationSHA recordednextcheckpoint.
