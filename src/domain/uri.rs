//! Strict local Key URI parser. Never retain a raw payload or return source errors.
use super::{
    otp::{Algorithm, Digits, OtpSettings, Period, Secret},
    records::AccountDraft,
    validation,
};
use crate::error::AppError;
use std::collections::BTreeSet;
use zeroize::Zeroizing;

pub const MAX_URI_BYTES: usize = 8192;

/// RFC percent decoding with strict UTF-8. '+' is a literal plus, not form space.
fn decode(input: &str) -> Result<Zeroizing<String>, AppError> {
    if !validation::valid_percent_encoding(input) {
        return Err(AppError::InvalidUri);
    }
    let mut bytes = Zeroizing::new(Vec::with_capacity(input.len()));
    let input = input.as_bytes();
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'%' {
            let hex = |b: u8| -> u8 {
                if b.is_ascii_digit() {
                    b - b'0'
                } else {
                    b.to_ascii_lowercase() - b'a' + 10
                }
            };
            bytes.push(hex(input[i + 1]) * 16 + hex(input[i + 2]));
            i += 3;
        } else {
            bytes.push(input[i]);
            i += 1;
        }
    }
    let decoded = std::str::from_utf8(&bytes).map_err(|_| AppError::InvalidUri)?;
    if decoded.chars().any(char::is_control) {
        return Err(AppError::InvalidUri);
    }
    Ok(Zeroizing::new(decoded.to_owned()))
}

pub fn parse_totp_uri(input: &str) -> Result<AccountDraft, AppError> {
    if input.len() > MAX_URI_BYTES
        || !input.is_ascii()
        || input
            .bytes()
            .any(|b| b.is_ascii_whitespace() || b.is_ascii_control())
        || input.contains(['#', '\\'])
    {
        return Err(AppError::InvalidUri);
    }
    let (scheme, remainder) = input
        .split_once("://")
        .ok_or(AppError::UnsupportedOtpType)?;
    if !scheme.eq_ignore_ascii_case("otpauth") {
        return Err(AppError::UnsupportedOtpType);
    }
    let (host, rest) = remainder.split_once('/').ok_or(AppError::InvalidUri)?;
    if !host.eq_ignore_ascii_case("totp") {
        return Err(AppError::UnsupportedOtpType);
    }
    let (label, query) = rest.split_once('?').ok_or(AppError::InvalidUri)?;
    if label.is_empty() || label.contains('/') || query.is_empty() || query.contains('?') {
        return Err(AppError::InvalidUri);
    }
    let label = decode(label)?;
    let (prefix, account) = match label.split_once(':') {
        Some((issuer, account)) if !account.contains(':') => (Some(issuer.trim()), account.trim()),
        Some(_) => return Err(AppError::InvalidUri),
        None => (None, label.trim()),
    };
    if account.is_empty() || prefix == Some("") {
        return Err(AppError::InvalidUri);
    }
    let mut settings = OtpSettings::default();
    let mut secret = None;
    let mut issuer = None;
    let mut seen = BTreeSet::new();
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').ok_or(AppError::InvalidUri)?;
        let key = decode(key)?;
        let value = decode(value)?;
        // Names are case-sensitive, but case aliases also count as duplicates.
        if !seen.insert(key.to_ascii_lowercase()) {
            return Err(AppError::DuplicateParameter);
        }
        match key.as_str() {
            "secret" => secret = Some(Secret::from_base32(&value)?),
            "issuer" if !value.contains(':') => {
                issuer = Some(validation::text(&value, 256, AppError::InvalidUri)?)
            }
            "issuer" => return Err(AppError::InvalidUri),
            "algorithm" => settings.algorithm = Algorithm::parse(&value)?,
            "digits" => settings.digits = Digits::parse(&value)?,
            "period" => settings.period = Period::parse(&value)?,
            _ => return Err(AppError::UnsupportedParameter),
        }
    }
    if let (Some(prefix), Some(issuer)) = (prefix, issuer.as_deref())
        && prefix != issuer
    {
        return Err(AppError::ConflictingIssuer);
    }
    // Explicit query issuer has precedence only when it agrees with a prefix.
    // A bare label is a service name if no issuer was supplied.
    let service = issuer.as_deref().or(prefix).unwrap_or(account);
    let account_identifier = (issuer.is_some() || prefix.is_some()).then_some(account);
    AccountDraft::new(
        service,
        None,
        account_identifier,
        secret.ok_or(AppError::InvalidSecret)?,
        settings,
    )
}
