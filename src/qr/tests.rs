use super::*;
use crate::domain::otp::{Algorithm, Digits, Period};
use image::{ExtendedColorType, ImageEncoder};
use qrcode::{Color, EcLevel, QrCode};
use std::error::Error;

type TestResult = Result<(), Box<dyn Error>>;
// RFC 6238's published SHA-1 seed, never a service credential.
const URI: &str =
    "otpauth://totp/Disposable:alice?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Disposable";

fn qr(payload: &[u8]) -> Result<(u32, Vec<u8>), Box<dyn Error>> {
    let code = QrCode::with_error_correction_level(payload, EcLevel::M)?;
    let size = (code.width() + 8) * 4;
    let mut pixels = vec![255; size * size];
    for y in 0..code.width() {
        for x in 0..code.width() {
            if code[(x, y)] == Color::Dark {
                for dy in 0..4 {
                    for dx in 0..4 {
                        pixels[((y + 4) * 4 + dy) * size + (x + 4) * 4 + dx] = 0;
                    }
                }
            }
        }
    }
    Ok((size as u32, pixels))
}
fn encoded(
    width: u32,
    height: u32,
    pixels: &[u8],
    jpeg: bool,
) -> Result<Zeroizing<Vec<u8>>, Box<dyn Error>> {
    let mut bytes = Zeroizing::new(Vec::new());
    if jpeg {
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut *bytes, 90).write_image(
            pixels,
            width,
            height,
            ExtendedColorType::L8,
        )?;
    } else {
        image::codecs::png::PngEncoder::new(&mut *bytes).write_image(
            pixels,
            width,
            height,
            ExtendedColorType::L8,
        )?;
    }
    Ok(bytes)
}
fn only(review: &ScanReview) -> Result<&AccountDraft, Box<dyn Error>> {
    assert_eq!(review.candidates().len(), 1);
    match &review.candidates()[0] {
        ReviewCandidate::Supported(draft) => Ok(draft),
        _ => Err("expected validated candidate".into()),
    }
}
fn mosaic(payloads: &[&[u8]]) -> Result<(u32, u32, Vec<u8>), Box<dyn Error>> {
    let codes = payloads
        .iter()
        .map(|p| qr(p))
        .collect::<Result<Vec<_>, _>>()?;
    let cell = codes.iter().map(|(w, _)| *w).max().ok_or("empty mosaic")? + 16;
    let columns = codes.len() as u32;
    let rows = (codes.len() as u32).div_ceil(columns);
    let width = cell * columns;
    let height = cell * rows;
    let mut pixels = vec![255; (width * height) as usize];
    for (i, (w, code)) in codes.iter().enumerate() {
        let left = (i as u32 % columns) * cell + 8;
        let top = (i as u32 / columns) * cell + 8;
        for y in 0..*w {
            let start = ((top + y) * width + left) as usize;
            pixels[start..start + *w as usize]
                .copy_from_slice(&code[(y * w) as usize..((y + 1) * w) as usize]);
        }
    }
    Ok((width, height, pixels))
}

#[test]
fn png_jpeg_and_pixels_preserve_every_algorithm_digits_period() -> TestResult {
    for (algorithm, name) in [
        (Algorithm::Sha1, "SHA1"),
        (Algorithm::Sha256, "SHA256"),
        (Algorithm::Sha512, "SHA512"),
    ] {
        for digits in [Digits::Six, Digits::Eight] {
            for period in [
                Period::Seconds15,
                Period::Seconds30,
                Period::Seconds60,
                Period::Seconds120,
            ] {
                let uri = Zeroizing::new(format!(
                    "{URI}&algorithm={name}&digits={}&period={}",
                    digits.count(),
                    period.seconds()
                ));
                let (width, pixels) = qr(uri.as_bytes())?;
                let memory = decode_pixels(
                    width,
                    width,
                    PixelLayout::Gray8,
                    Zeroizing::new(pixels.clone()),
                )?;
                let png = decode_image(encoded(width, width, &pixels, false)?)?;
                let jpg = decode_image(encoded(width, width, &pixels, true)?)?;
                for result in [&memory, &png, &jpg] {
                    let draft = only(result)?;
                    assert_eq!(draft.service(), "Disposable");
                    assert_eq!(draft.account_identifier(), Some("alice"));
                    assert_eq!(draft.settings().algorithm, algorithm);
                    assert_eq!(draft.settings().digits, digits);
                    assert_eq!(draft.settings().period, period);
                    assert!(draft.secret().same_seed(only(&memory)?.secret()));
                    assert!(!result.requires_choice());
                }
            }
        }
    }
    Ok(())
}

#[test]
fn capture_layouts_rotation_and_deterministic_noise_decode() -> TestResult {
    let (width, pixels) = qr(URI.as_bytes())?;
    for layout in [
        PixelLayout::Gray8,
        PixelLayout::Rgb8,
        PixelLayout::Rgba8,
        PixelLayout::Bgra8,
    ] {
        let mut packed = Vec::with_capacity(pixels.len() * layout.channels());
        for &p in &pixels {
            match layout {
                PixelLayout::Gray8 => packed.push(p),
                PixelLayout::Rgb8 => packed.extend_from_slice(&[p, p, p]),
                _ => packed.extend_from_slice(&[p, p, p, 255]),
            }
        }
        assert_eq!(
            only(&decode_pixels(
                width,
                width,
                layout,
                Zeroizing::new(packed)
            )?)?
            .service(),
            "Disposable"
        );
    }
    let mut rotated = pixels.clone();
    for _ in 0..3 {
        let before = rotated.clone();
        for y in 0..width {
            for x in 0..width {
                rotated[(y * width + x) as usize] = before[((width - 1 - x) * width + y) as usize];
            }
        }
        // Low-amplitude sensor/JPEG-like noise does not change modules.
        let noisy = rotated
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if *p == 0 {
                    (i % 23) as u8
                } else {
                    255 - (i % 23) as u8
                }
            })
            .collect();
        only(&decode_pixels(
            width,
            width,
            PixelLayout::Gray8,
            Zeroizing::new(noisy),
        )?)?;
    }
    Ok(())
}

#[test]
fn transparent_png_and_sixteen_bit_png_keep_qr_contrast() -> TestResult {
    let (w, pixels) = qr(URI.as_bytes())?;
    let rgba = pixels
        .iter()
        .flat_map(|p| [0, 0, 0, if *p == 0 { 255 } else { 0 }])
        .collect::<Vec<_>>();
    let mut bytes = Zeroizing::new(Vec::new());
    image::codecs::png::PngEncoder::new(&mut *bytes).write_image(
        &rgba,
        w,
        w,
        ExtendedColorType::Rgba8,
    )?;
    only(&decode_image(bytes)?)?;
    let sixteen = pixels
        .iter()
        .flat_map(|p| ((*p as u16) * 257).to_ne_bytes())
        .collect::<Vec<_>>();
    let mut bytes = Zeroizing::new(Vec::new());
    image::codecs::png::PngEncoder::new(&mut *bytes).write_image(
        &sixteen,
        w,
        w,
        ExtendedColorType::L16,
    )?;
    only(&decode_image(bytes)?)?;
    // Explicit channel order and straight-alpha behavior for capture adapters.
    assert_eq!(luma(255, 0, 0, 255), 76);
    assert_eq!(luma(0, 0, 0, 0), 255);
    assert_eq!(
        image_gray(&[255, 0, 0], ColorType::Rgb8, 1)?.as_slice(),
        &[76]
    );
    Ok(())
}

#[test]
fn damaged_qr_has_a_safe_decode_failure_candidate() -> TestResult {
    let (w, mut pixels) = qr(URI.as_bytes())?;
    // Preserve all three finders and timing lines while destroying data/ECC.
    for y in w / 2..w - 16 {
        for x in w / 3..w - 16 {
            pixels[(y * w + x) as usize] = 255;
        }
    }
    let review = decode_pixels(w, w, PixelLayout::Gray8, Zeroizing::new(pixels))?;
    assert_eq!(review.candidates().len(), 1);
    assert!(matches!(
        review.candidates()[0],
        ReviewCandidate::DecodeFailure
    ));
    Ok(())
}

#[test]
fn mixed_codes_preserve_safe_outcomes_and_require_explicit_choice() -> TestResult {
    let invalid = b"otpauth://totp/Broken?secret=0&issuer=Disposable";
    let unsupported = b"https://example.invalid/";
    let (w, h, pixels) = mosaic(&[URI.as_bytes(), invalid, unsupported, URI.as_bytes()])?;
    let review = decode_pixels(w, h, PixelLayout::Gray8, Zeroizing::new(pixels))?;
    assert_eq!(review.candidates().len(), 4);
    assert_eq!(review.supported_count(), 2);
    assert!(review.requires_choice());
    assert_eq!(
        review
            .candidates()
            .iter()
            .filter(|c| matches!(c, ReviewCandidate::Unsupported))
            .count(),
        1
    );
    assert_eq!(
        review
            .candidates()
            .iter()
            .filter(|c| matches!(
                c,
                ReviewCandidate::MalformedCredential(AppError::InvalidSecret)
            ))
            .count(),
        1
    );
    assert_eq!(review.into_candidates().len(), 4);
    Ok(())
}

#[test]
fn actual_qr_payloads_are_strict_utf8_control_and_length_bounded() -> TestResult {
    for payload in [
        &b"\xff\xfe"[..],
        &b"otpauth://totp/test\0?secret=MY"[..],
        &vec![b'A'; MAX_PAYLOAD_BYTES + 1][..],
    ] {
        let (w, pixels) = qr(payload)?;
        let review = decode_pixels(w, w, PixelLayout::Gray8, Zeroizing::new(pixels))?;
        assert!(matches!(
            review.candidates()[0],
            ReviewCandidate::InvalidPayload
        ));
    }
    assert!(matches!(
        classify(Zeroizing::new(vec![b'A'; MAX_PAYLOAD_BYTES])),
        ReviewCandidate::Unsupported
    ));
    Ok(())
}

#[test]
fn qr_boundary_keeps_task02_strict_parser_policy() {
    for (uri, expected) in [
        (
            format!("{URI}&unknown=value"),
            AppError::UnsupportedParameter,
        ),
        (format!("{URI}&period=31"), AppError::InvalidPeriod),
        (format!("{URI}&algorithm=MD5"), AppError::InvalidAlgorithm),
        (
            format!("{URI}&issuer=Disposable"),
            AppError::DuplicateParameter,
        ),
        (
            URI.replace("issuer=Disposable", "issuer=Other"),
            AppError::ConflictingIssuer,
        ),
    ] {
        assert!(
            matches!(classify(Zeroizing::new(uri.into_bytes())),ReviewCandidate::MalformedCredential(e) if e == expected)
        );
    }
    for uri in [
        "otpauth://hotp/A?secret=MY",
        "https://example.invalid",
        "otpauth-migration://offline?data=AA",
    ] {
        assert!(matches!(
            classify(Zeroizing::new(uri.as_bytes().to_vec())),
            ReviewCandidate::Unsupported
        ));
    }
}

#[test]
fn saturated_detector_rejects_before_any_payload_decode_or_partial_review() -> TestResult {
    let (w, h, pixels) = mosaic(&[URI.as_bytes(); 8])?;
    assert_eq!(
        decode_pixels(w, h, PixelLayout::Gray8, Zeroizing::new(pixels)).err(),
        Some(QrError::CandidateLimit)
    );
    let (w, h, pixels) = mosaic(&[URI.as_bytes(); MAX_CANDIDATES])?;
    assert_eq!(
        decode_pixels(w, h, PixelLayout::Gray8, Zeroizing::new(pixels))?.supported_count(),
        MAX_CANDIDATES
    );
    Ok(())
}

#[test]
fn no_qr_corrupt_and_unsupported_images_have_distinct_fixed_errors() -> TestResult {
    assert_eq!(
        decode_pixels(
            128,
            128,
            PixelLayout::Gray8,
            Zeroizing::new(vec![255; 128 * 128])
        )
        .err(),
        Some(QrError::NoQr)
    );
    assert_eq!(
        decode_image(Zeroizing::new(b"GIF89a".to_vec())).err(),
        Some(QrError::UnsupportedImage)
    );
    assert_eq!(
        decode_image(Zeroizing::new(b"\x89PNG\r\n\x1a\n".to_vec())).err(),
        Some(QrError::DecodeFailure)
    );
    assert_eq!(
        decode_image(Zeroizing::new(vec![0xff, 0xd8, 0xff, 0xda])).err(),
        Some(QrError::DecodeFailure)
    );
    let (w, pixels) = qr(URI.as_bytes())?;
    let png = encoded(w, w, &pixels, false)?;
    assert_eq!(
        decode_image(Zeroizing::new(png[..png.len() / 2].to_vec())).err(),
        Some(QrError::DecodeFailure)
    );
    Ok(())
}

#[test]
fn file_input_sniffs_content_preserves_originals_and_writes_no_artifacts() -> TestResult {
    let directory = tempfile::tempdir()?;
    let (w, pixels) = qr(URI.as_bytes())?;
    for jpeg in [false, true] {
        // Only the fixture producer writes. The service receives a generic path
        // with the wrong extension; it neither modifies nor copies the original.
        let file = tempfile::Builder::new()
            .suffix(".dat")
            .tempfile_in(directory.path())?;
        let bytes = encoded(w, w, &pixels, jpeg)?;
        std::fs::write(file.path(), &bytes)?;
        let before = std::fs::read_dir(directory.path())?.count();
        assert_eq!(only(&decode_file(file.path())?)?.service(), "Disposable");
        assert_eq!(std::fs::read(file.path())?, *bytes);
        assert_eq!(std::fs::read_dir(directory.path())?.count(), before);
    }
    let file = tempfile::NamedTempFile::new_in(directory.path())?;
    std::fs::write(file.path(), b"invalid disposable image")?;
    let before = std::fs::read_dir(directory.path())?.count();
    assert_eq!(
        decode_file(file.path()).err(),
        Some(QrError::UnsupportedImage)
    );
    assert_eq!(std::fs::read(file.path())?, b"invalid disposable image");
    assert_eq!(std::fs::read_dir(directory.path())?.count(), before);
    assert_eq!(
        decode_file(directory.path()).err(),
        Some(QrError::FileUnavailable)
    );
    Ok(())
}

#[test]
fn metadata_and_growth_race_reads_are_bounded() -> TestResult {
    let file = tempfile::NamedTempFile::new()?;
    file.as_file().set_len(MAX_FILE_BYTES as u64 + 1)?;
    assert_eq!(decode_file(file.path()).err(), Some(QrError::ImageLimit));
    // Represents a source that grows after its small metadata observation.
    let source = std::io::repeat(0x41).take((MAX_FILE_BYTES + 1024) as u64);
    assert_eq!(read_bounded(source).err(), Some(QrError::ImageLimit));
    assert_eq!(read_bounded(&b"short"[..])?.as_slice(), b"short");
    Ok(())
}

fn png_header(width: u32, height: u32) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    b.extend_from_slice(&width.to_be_bytes());
    b.extend_from_slice(&height.to_be_bytes());
    b.extend_from_slice(&[16, 6, 0, 0, 0, 0, 0, 0, 0]);
    b
}
#[test]
fn adversarial_geometry_segments_and_pixel_allocations_fail_before_decode() {
    for (w, h) in [
        (0, 1),
        (1, 0),
        (u32::MAX, u32::MAX),
        (MAX_DIMENSION + 1, 1),
        (4096, 4096),
    ] {
        assert_eq!(
            decode_image(Zeroizing::new(png_header(w, h))).err(),
            Some(QrError::ImageLimit)
        );
        assert_eq!(
            decode_pixels(w, h, PixelLayout::Bgra8, Zeroizing::new(Vec::new())).err(),
            Some(QrError::ImageLimit)
        );
    }
    assert_eq!(
        decode_pixels(
            12,
            12,
            PixelLayout::Bgra8,
            Zeroizing::new(vec![0; 12 * 12 * 4 - 1])
        )
        .err(),
        Some(QrError::InvalidPixels)
    );
    let jpeg = vec![
        0xff, 0xd8, 0xff, 0xc0, 0, 11, 8, 0xff, 0xff, 0xff, 0xff, 1, 1, 0x11, 0,
    ];
    assert_eq!(
        decode_image(Zeroizing::new(jpeg)).err(),
        Some(QrError::ImageLimit)
    );
    for len in [0, 1, 2, 65535] {
        let mut jpeg = vec![0xff, 0xd8, 0xff, 0xe1];
        jpeg.extend_from_slice(&(len as u16).to_be_bytes());
        assert_eq!(
            decode_image(Zeroizing::new(jpeg)).err(),
            Some(QrError::DecodeFailure)
        );
    }
    assert_eq!(
        decode_image(Zeroizing::new(vec![0; MAX_FILE_BYTES + 1])).err(),
        Some(QrError::ImageLimit)
    );
}

#[test]
fn png_hostile_ancillary_length_is_bounded_before_body_allocation() -> TestResult {
    let bytes = encoded(1, 1, &[255], false)?;
    let mut hostile = bytes[..33].to_vec();
    hostile.extend_from_slice(&0x7fff_ffffu32.to_be_bytes());
    hostile.extend_from_slice(b"tEXt");
    hostile.extend_from_slice(&bytes[33..]);
    assert_eq!(
        decode_image(Zeroizing::new(hostile)).err(),
        Some(QrError::DecodeFailure)
    );
    Ok(())
}

#[test]
fn deterministic_adversarial_bytes_and_noise_never_panic() -> TestResult {
    let mut state = 0x1234_5678u32;
    for len in 0..256 {
        let mut bytes = vec![0; len];
        for byte in &mut bytes {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            *byte = (state >> 24) as u8;
        }
        if len > 8 {
            if len % 2 == 0 {
                bytes[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
            } else {
                bytes[..2].copy_from_slice(&[0xff, 0xd8]);
            }
        }
        assert!(decode_image(Zeroizing::new(bytes)).is_err());
    }
    for kind in 0..4 {
        let noise = (0..512 * 512)
            .map(|i| match kind {
                0 => {
                    if (i / 512 + i % 512) % 2 == 0 {
                        0
                    } else {
                        255
                    }
                }
                1 => {
                    if i % 11 < 5 {
                        0
                    } else {
                        255
                    }
                }
                _ => {
                    state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                    (state >> 24) as u8
                }
            })
            .collect();
        let result = decode_pixels(512, 512, PixelLayout::Gray8, Zeroizing::new(noise));
        if let Ok(review) = result {
            assert_eq!(review.supported_count(), 0);
        }
    }
    Ok(())
}

#[test]
fn redacted_aggregate_debug_and_fixed_errors_never_include_metadata_or_source() -> TestResult {
    let (w, pixels) = qr(URI.as_bytes())?;
    let review = decode_pixels(w, w, PixelLayout::Gray8, Zeroizing::new(pixels))?;
    for formatted in [
        format!("{review:?}"),
        format!("{:?}", review.candidates()),
        format!("{:?}", only(&review)?),
    ] {
        for forbidden in ["Disposable", "alice", "GEZDGNBV", "otpauth", "secret="] {
            assert!(!formatted.contains(forbidden));
        }
    }
    for error in [
        QrError::FileUnavailable,
        QrError::UnsupportedImage,
        QrError::ImageLimit,
        QrError::InvalidPixels,
        QrError::DecodeFailure,
        QrError::CandidateLimit,
        QrError::NoQr,
    ] {
        assert!(!format!("{error:?}: {error}").contains(URI));
    }
    Ok(())
}
