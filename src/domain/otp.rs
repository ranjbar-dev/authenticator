//! Deterministic generation only: no URI parser, persistence or system-clock reads.
use super::clock::Clock;
use crate::error::AppError;
use data_encoding::{BASE32, BASE32_NOPAD};
use std::fmt;
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Algorithm {
    #[default]
    Sha1,
    Sha256,
    Sha512,
}

impl Algorithm {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            v if v.eq_ignore_ascii_case("SHA1") => Ok(Self::Sha1),
            v if v.eq_ignore_ascii_case("SHA256") => Ok(Self::Sha256),
            v if v.eq_ignore_ascii_case("SHA512") => Ok(Self::Sha512),
            _ => Err(AppError::InvalidAlgorithm),
        }
    }

    fn primitive(self) -> totp_rs::Algorithm {
        match self {
            Self::Sha1 => totp_rs::Algorithm::SHA1,
            Self::Sha256 => totp_rs::Algorithm::SHA256,
            Self::Sha512 => totp_rs::Algorithm::SHA512,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Digits {
    #[default]
    Six,
    Eight,
}

impl Digits {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "6" => Ok(Self::Six),
            "8" => Ok(Self::Eight),
            _ => Err(AppError::InvalidDigits),
        }
    }

    pub fn count(self) -> u8 {
        match self {
            Self::Six => 6,
            Self::Eight => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Period {
    Seconds15,
    #[default]
    Seconds30,
    Seconds60,
    Seconds120,
}

impl Period {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "15" => Ok(Self::Seconds15),
            "30" => Ok(Self::Seconds30),
            "60" => Ok(Self::Seconds60),
            "120" => Ok(Self::Seconds120),
            _ => Err(AppError::InvalidPeriod),
        }
    }

    pub fn seconds(self) -> u64 {
        match self {
            Self::Seconds15 => 15,
            Self::Seconds30 => 30,
            Self::Seconds60 => 60,
            Self::Seconds120 => 120,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OtpSettings {
    pub algorithm: Algorithm,
    pub digits: Digits,
    pub period: Period,
}

/// Decoded seed. No Display, serialization, or implicit string conversion.
#[derive(Clone)]
pub struct Secret(Zeroizing<Vec<u8>>);

impl Secret {
    pub const MAX_INPUT_BYTES: usize = 4096;
    pub const MAX_DECODED_BYTES: usize = 1024;

    /// Accept ASCII case plus SP/HT/CR/LF anywhere. Padding must be either absent
    /// or exactly RFC 4648 padding. Reject aliases (0/1), Unicode spaces and
    /// nonzero trailing bits. The caller owns/clears the original form buffer.
    pub fn from_base32(input: &str) -> Result<Self, AppError> {
        if input.len() > Self::MAX_INPUT_BYTES {
            return Err(AppError::InvalidSecret);
        }
        // Filtering has no exact iterator length. Reserve the already bounded
        // input once, rather than reallocating plaintext Base32 copies as it grows.
        let mut normalized = Zeroizing::new(Vec::with_capacity(input.len()));
        normalized.extend(
            input
                .bytes()
                .filter(|b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
                .map(|b| b.to_ascii_uppercase()),
        );
        if let Some(first_padding) = normalized.iter().position(|b| *b == b'=')
            && normalized[first_padding..].iter().any(|b| *b != b'=')
        {
            return Err(AppError::InvalidSecret);
        }
        let encoding = if normalized.contains(&b'=') {
            BASE32
        } else {
            BASE32_NOPAD
        };
        let capacity = encoding
            .decode_len(normalized.len())
            .map_err(|_| AppError::InvalidSecret)?;
        // Input is already bounded. Zeroize even partial decoded output on error.
        let mut bytes = Zeroizing::new(vec![0; capacity]);
        let length = encoding
            .decode_mut(&normalized, &mut bytes)
            .map_err(|_| AppError::InvalidSecret)?;
        bytes.truncate(length);
        Self::from_bytes(bytes)
    }

    /// Also used by the encrypted-storage boundary after authenticated decoding.
    pub fn from_bytes(bytes: Zeroizing<Vec<u8>>) -> Result<Self, AppError> {
        if bytes.is_empty() || bytes.len() > Self::MAX_DECODED_BYTES {
            return Err(AppError::InvalidSecret);
        }
        Ok(Self(bytes))
    }

    pub fn expose_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn expose_base32(&self) -> Zeroizing<String> {
        Zeroizing::new(BASE32_NOPAD.encode(&self.0))
    }

    pub fn same_seed(&self, other: &Self) -> bool {
        bool::from(self.expose_bytes().ct_eq(other.expose_bytes()))
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret([REDACTED])")
    }
}

pub struct CurrentCode {
    value: Zeroizing<String>,
    pub step: u64,
    pub remaining_seconds: u64,
    /// None only when the mathematical next boundary exceeds u64 UTC seconds.
    pub next_boundary: Option<u64>,
}

impl CurrentCode {
    pub fn expose_value(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for CurrentCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CurrentCode([REDACTED])")
    }
}

/// Always reads the injected clock once and recomputes; there is no code cache.
pub fn current_code(
    secret: &Secret,
    settings: OtpSettings,
    clock: &dyn Clock,
) -> Result<CurrentCode, AppError> {
    let now = clock.unix_seconds()?;
    let period = settings.period.seconds();
    let remaining_seconds = period - now % period;
    // The maintained library implements HMAC, truncation and fixed-width output.
    // Noncompliant construction allows importing nonempty legacy short seeds;
    // our enums enforce its other preconditions (nonzero step, 6/8 digits).
    let generator = totp_rs::Builder::new()
        .with_algorithm(settings.algorithm.primitive())
        .with_digits(settings.digits.count())
        .with_step_duration(period)
        .with_secret(secret.expose_bytes())
        .build_noncompliant();
    let value = Zeroizing::new(generator.generate(now).to_string());
    Ok(CurrentCode {
        value,
        step: now / period,
        remaining_seconds,
        next_boundary: now.checked_add(remaining_seconds),
    })
}
