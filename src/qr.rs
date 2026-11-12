//! Bounded, local-only QR decoding. No capture, vault access, writes or logging.
use crate::domain::{records::AccountDraft, uri::parse_totp_uri};
use crate::error::AppError;
use image::{ColorType, ImageDecoder, ImageFormat, ImageReader, Limits};
use std::{
    fmt,
    fs::File,
    io::{Cursor, Read},
    path::Path,
};
use zeroize::{Zeroize, Zeroizing};

pub const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_DIMENSION: u32 = 4096;
pub const MAX_PIXELS: usize = 4 * 1024 * 1024;
pub const MAX_DECODED_BYTES: usize = MAX_PIXELS * 8;
pub const MAX_PAYLOAD_BYTES: usize = 2048;
// quircs itself stops at eight grids / 32 capstones. Reject saturation rather
// than offer an apparently complete, silently truncated set of credentials.
pub const MAX_CANDIDATES: usize = 7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QrError {
    FileUnavailable,
    UnsupportedImage,
    ImageLimit,
    InvalidPixels,
    DecodeFailure,
    CandidateLimit,
    NoQr,
}
impl fmt::Display for QrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::FileUnavailable => "The image could not be read. Choose another image or use manual entry.",
            Self::UnsupportedImage => "Choose a PNG or JPEG image, or use manual entry.",
            Self::ImageLimit => "This image is too large. Select a smaller region or image.",
            Self::InvalidPixels => "The selected image pixels are invalid. Capture the region again.",
            Self::DecodeFailure => "The image could not be decoded. Retry with a clearer PNG/JPEG image or use manual entry.",
            Self::CandidateLimit => "Too many QR patterns were found. Select a smaller region and retry.",
            Self::NoQr => "No QR code was found. Try a closer, clearer region, an image or manual entry.",
        })
    }
}
impl std::error::Error for QrError {}

/// Tight rows, top to bottom; no stride, premultiplication or orientation flag.
/// Alpha is straight and composited over white. Windows GDI's unused BGRA byte
/// must be normalized to 255 by the capture adapter, never passed as alpha=0.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PixelLayout {
    Gray8,
    Rgb8,
    Rgba8,
    Bgra8,
}
impl PixelLayout {
    fn channels(self) -> usize {
        match self {
            Self::Gray8 => 1,
            Self::Rgb8 => 3,
            Self::Rgba8 | Self::Bgra8 => 4,
        }
    }
}

/// Index corresponds to the detected QR's entry in `ScanReview::candidates`.
/// Invalid entries contain only fixed enums, never the decoded source text.
pub enum ReviewCandidate {
    Supported(AccountDraft),
    Unsupported,
    MalformedCredential(AppError),
    InvalidPayload,
    DecodeFailure,
}
impl fmt::Debug for ReviewCandidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReviewCandidate([REDACTED])")
    }
}
pub struct ScanReview {
    candidates: Vec<ReviewCandidate>,
}
impl ScanReview {
    pub fn candidates(&self) -> &[ReviewCandidate] {
        &self.candidates
    }
    /// Moves the validated review data to the caller, without selecting or saving
    /// any credential. Even a single supported candidate needs explicit review.
    pub fn into_candidates(self) -> Vec<ReviewCandidate> {
        self.candidates
    }
    pub fn supported_count(&self) -> usize {
        self.candidates
            .iter()
            .filter(|c| matches!(c, ReviewCandidate::Supported(_)))
            .count()
    }
    pub fn requires_choice(&self) -> bool {
        self.supported_count() > 1
    }
}
impl fmt::Debug for ScanReview {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ScanReview([REDACTED])")
    }
}

fn pixel_count(width: u32, height: u32) -> Result<usize, QrError> {
    let count = (width as usize)
        .checked_mul(height as usize)
        .ok_or(QrError::ImageLimit)?;
    if width == 0
        || height == 0
        || width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || count > MAX_PIXELS
    {
        return Err(QrError::ImageLimit);
    }
    Ok(count)
}

/// Opens only the explicitly supplied file, read-only. Both metadata and a
/// max+1 bounded read protect against oversized files and growth after metadata.
pub fn decode_file(path: &Path) -> Result<ScanReview, QrError> {
    // Reject directories, links and special/device files before attempting an
    // open; the handle metadata check below also covers replacement races.
    let selected = std::fs::symlink_metadata(path).map_err(|_| QrError::FileUnavailable)?;
    if !selected.is_file() || selected.file_type().is_symlink() {
        return Err(QrError::FileUnavailable);
    }
    if selected.len() > MAX_FILE_BYTES as u64 {
        return Err(QrError::ImageLimit);
    }
    let file = File::open(path).map_err(|_| QrError::FileUnavailable)?;
    let metadata = file.metadata().map_err(|_| QrError::FileUnavailable)?;
    if !metadata.is_file() {
        return Err(QrError::FileUnavailable);
    }
    if metadata.len() > MAX_FILE_BYTES as u64 {
        return Err(QrError::ImageLimit);
    }
    let bytes = read_bounded(file)?;
    decode_image(bytes)
}

fn read_bounded(reader: impl Read) -> Result<Zeroizing<Vec<u8>>, QrError> {
    // Reserve the entire cap once: growth must not leave abandoned image copies.
    let mut bytes = Zeroizing::new(Vec::with_capacity(MAX_FILE_BYTES + 1));
    reader
        .take((MAX_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| QrError::FileUnavailable)?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(QrError::ImageLimit);
    }
    Ok(bytes)
}

/// Encoded bytes are consumed and zeroized on every return path. Format is
/// sniffed from content; the filename/extension is never trusted or retained.
pub fn decode_image(bytes: Zeroizing<Vec<u8>>) -> Result<ScanReview, QrError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| decode_image_inner(bytes)))
        .map_err(|_| QrError::DecodeFailure)?
}

fn decode_image_inner(bytes: Zeroizing<Vec<u8>>) -> Result<ScanReview, QrError> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(QrError::ImageLimit);
    }
    let (format, width, height) = header(&bytes)?;
    let count = pixel_count(width, height)?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_DECODED_BYTES as u64);
    let mut reader = ImageReader::with_format(Cursor::new(bytes.as_slice()), format);
    reader.limits(limits);
    let decoder = reader.into_decoder().map_err(|_| QrError::DecodeFailure)?;
    if decoder.dimensions() != (width, height) {
        return Err(QrError::DecodeFailure);
    }
    let size = usize::try_from(decoder.total_bytes()).map_err(|_| QrError::ImageLimit)?;
    if size > MAX_DECODED_BYTES {
        return Err(QrError::ImageLimit);
    }
    let color = decoder.color_type();
    let mut raw = Zeroizing::new(vec![0u8; size]);
    decoder
        .read_image(&mut raw)
        .map_err(|_| QrError::DecodeFailure)?;
    let gray = image_gray(&raw, color, count)?;
    scan_gray(width, height, gray)
}

// Check dimensions before invoking format libraries, including before their
// header/metadata allocations. Segment traversal never trusts a source length.
fn header(bytes: &[u8]) -> Result<(ImageFormat, u32, u32), QrError> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        if bytes.len() < 33 || bytes[8..16] != *b"\0\0\0\rIHDR" {
            return Err(QrError::DecodeFailure);
        }
        let width = u32::from_be_bytes(
            bytes[16..20]
                .try_into()
                .map_err(|_| QrError::DecodeFailure)?,
        );
        let height = u32::from_be_bytes(
            bytes[20..24]
                .try_into()
                .map_err(|_| QrError::DecodeFailure)?,
        );
        pixel_count(width, height)?;
        return Ok((ImageFormat::Png, width, height));
    }
    if !bytes.starts_with(&[0xff, 0xd8]) {
        return Err(QrError::UnsupportedImage);
    }
    let mut i = 2;
    while i < bytes.len() {
        if bytes[i] != 0xff {
            return Err(QrError::DecodeFailure);
        }
        while i < bytes.len() && bytes[i] == 0xff {
            i += 1;
        }
        let marker = *bytes.get(i).ok_or(QrError::DecodeFailure)?;
        i += 1;
        if matches!(marker, 0x00 | 0xd8 | 0xd9 | 0xda | 0xd0..=0xd7) {
            return Err(QrError::DecodeFailure);
        }
        let length_bytes = bytes.get(i..i + 2).ok_or(QrError::DecodeFailure)?;
        let length = u16::from_be_bytes([length_bytes[0], length_bytes[1]]) as usize;
        if length < 2 {
            return Err(QrError::DecodeFailure);
        }
        let segment = bytes.get(i..i + length).ok_or(QrError::DecodeFailure)?;
        if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
            if segment.len() < 8 {
                return Err(QrError::DecodeFailure);
            }
            let height = u16::from_be_bytes([segment[3], segment[4]]) as u32;
            let width = u16::from_be_bytes([segment[5], segment[6]]) as u32;
            pixel_count(width, height)?;
            return Ok((ImageFormat::Jpeg, width, height));
        }
        i += length;
    }
    Err(QrError::DecodeFailure)
}

fn luma(r: u8, g: u8, b: u8, alpha: u8) -> u8 {
    let value = (299 * r as u32 + 587 * g as u32 + 114 * b as u32) / 1000;
    ((value * alpha as u32 + 255 * (255 - alpha as u32)) / 255) as u8
}
fn image_gray(raw: &[u8], color: ColorType, count: usize) -> Result<Zeroizing<Vec<u8>>, QrError> {
    let channels = color.channel_count() as usize;
    let sixteen = matches!(
        color,
        ColorType::L16 | ColorType::La16 | ColorType::Rgb16 | ColorType::Rgba16
    );
    if !matches!(
        color,
        ColorType::L8
            | ColorType::La8
            | ColorType::Rgb8
            | ColorType::Rgba8
            | ColorType::L16
            | ColorType::La16
            | ColorType::Rgb16
            | ColorType::Rgba16
    ) {
        return Err(QrError::DecodeFailure);
    }
    let stride = channels * if sixteen { 2 } else { 1 };
    if raw.len() != count * stride {
        return Err(QrError::DecodeFailure);
    }
    let mut gray = Zeroizing::new(Vec::with_capacity(count));
    for p in raw.chunks_exact(stride) {
        let sample = |i: usize| {
            if sixteen {
                (u16::from_ne_bytes([p[2 * i], p[2 * i + 1]]) >> 8) as u8
            } else {
                p[i]
            }
        };
        let (r, g, b) = if channels <= 2 {
            (sample(0), sample(0), sample(0))
        } else {
            (sample(0), sample(1), sample(2))
        };
        let alpha = if channels == 2 || channels == 4 {
            sample(channels - 1)
        } else {
            255
        };
        gray.push(luma(r, g, b, alpha));
    }
    Ok(gray)
}

/// Consumes a selected-region buffer. The producer must enforce these same
/// limits before allocating/capturing; this boundary also validates exact size.
pub fn decode_pixels(
    width: u32,
    height: u32,
    layout: PixelLayout,
    pixels: Zeroizing<Vec<u8>>,
) -> Result<ScanReview, QrError> {
    let count = pixel_count(width, height)?;
    if pixels.len() != count * layout.channels() {
        return Err(QrError::InvalidPixels);
    }
    if layout == PixelLayout::Gray8 {
        return scan_gray(width, height, pixels);
    }
    let mut gray = Zeroizing::new(Vec::with_capacity(count));
    for p in pixels.chunks_exact(layout.channels()) {
        let (r, g, b) = if layout == PixelLayout::Bgra8 {
            (p[2], p[1], p[0])
        } else {
            (p[0], p[1], p[2])
        };
        gray.push(luma(r, g, b, if p.len() == 4 { p[3] } else { 255 }));
    }
    scan_gray(width, height, gray)
}

struct Recognizer(quircs::Quirc);
impl Drop for Recognizer {
    fn drop(&mut self) {
        self.0.pixels.zeroize();
    }
}
struct CodeBuffer(quircs::Code);
impl Drop for CodeBuffer {
    fn drop(&mut self) {
        self.0.cell_bitmap.zeroize();
    }
}
fn scan_gray(width: u32, height: u32, gray: Zeroizing<Vec<u8>>) -> Result<ScanReview, QrError> {
    let mut recognizer = Recognizer(quircs::Quirc::default());
    // Defense in depth for upstream extraction/arithmetic assertions. The app's
    // panic hook suppresses payloads; never format a dependency error or panic.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let q = &mut recognizer.0;
        let mut codes = Vec::with_capacity(MAX_CANDIDATES + 1);
        for code in q
            .identify(width as usize, height as usize, &gray)
            .take(MAX_CANDIDATES + 1)
        {
            codes.push(code.map(CodeBuffer));
        }
        if q.count() > MAX_CANDIDATES || q.capstones.len() >= 32 || q.regions.len() >= 65_534 {
            return Err(QrError::CandidateLimit);
        }
        if q.count() == 0 {
            return Err(QrError::NoQr);
        }
        let mut candidates = Vec::with_capacity(q.count());
        for code in codes {
            let candidate = match code {
                Ok(code) => match code.0.decode() {
                    Ok(mut data) => classify(Zeroizing::new(std::mem::take(&mut data.payload))),
                    Err(_) => ReviewCandidate::DecodeFailure,
                },
                Err(_) => ReviewCandidate::DecodeFailure,
            };
            candidates.push(candidate);
        }
        Ok(ScanReview { candidates })
    }))
    .map_err(|_| QrError::DecodeFailure)?
}

fn classify(payload: Zeroizing<Vec<u8>>) -> ReviewCandidate {
    if payload.len() > MAX_PAYLOAD_BYTES {
        return ReviewCandidate::InvalidPayload;
    }
    let Ok(text) = std::str::from_utf8(&payload) else {
        return ReviewCandidate::InvalidPayload;
    };
    if text.chars().any(char::is_control) {
        return ReviewCandidate::InvalidPayload;
    }
    match parse_totp_uri(text) {
        Ok(draft) => ReviewCandidate::Supported(draft),
        Err(AppError::UnsupportedOtpType) => ReviewCandidate::Unsupported,
        Err(error) => ReviewCandidate::MalformedCredential(error),
    }
}

#[cfg(test)]
mod tests;
