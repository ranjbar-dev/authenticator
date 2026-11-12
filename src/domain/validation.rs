use crate::error::AppError;
use url::Url;

pub(crate) fn text(input: &str, max_bytes: usize, error: AppError) -> Result<String, AppError> {
    // Check before trim so controls are never silently removed from identifiers.
    if input.len() > max_bytes || input.chars().any(char::is_control) {
        return Err(error);
    }
    let normalized = input.trim();
    if normalized.is_empty() {
        return Err(error);
    }
    Ok(normalized.to_owned())
}

pub(crate) fn optional_text(
    input: Option<&str>,
    max_bytes: usize,
    error: AppError,
) -> Result<Option<String>, AppError> {
    match input {
        None | Some("") => Ok(None),
        Some(value) if value.trim().is_empty() && !value.chars().any(char::is_control) => Ok(None),
        Some(value) => text(value, max_bytes, error).map(Some),
    }
}

pub(crate) fn website(input: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(value) = optional_text(input, 2048, AppError::InvalidWebsite)? else {
        return Ok(None);
    };
    // WHATWG permits corrections (e.g. backslashes/spaces). Do not accept those
    // spellings as if the user had entered a different address.
    if value.chars().any(char::is_whitespace)
        || value.contains('\\')
        || !valid_percent_encoding(&value)
    {
        return Err(AppError::InvalidWebsite);
    }
    let parsed = Url::parse(&value).map_err(|_| AppError::InvalidWebsite)?;
    let authority = value
        .split_once("://")
        .map(|(_, rest)| rest.split(['/', '?', '#']).next().unwrap_or(""));
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || !value
            .to_ascii_lowercase()
            .starts_with(&format!("{}://", parsed.scheme()))
        || authority.is_none_or(|host| host.is_empty() || host.contains('@'))
    {
        return Err(AppError::InvalidWebsite);
    }
    Ok(Some(value))
}

pub(crate) fn valid_percent_encoding(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len()
                || !bytes[i + 1].is_ascii_hexdigit()
                || !bytes[i + 2].is_ascii_hexdigit()
            {
                return false;
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    true
}
