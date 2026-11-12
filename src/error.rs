use std::fmt;

/// Fixed public messages only. Never attach source payloads, paths or credentials.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppError {
    ClockUnavailable,
    CaptureUnavailable,
    InvalidSelection,
    StorageUnavailable,
    InvalidSecret,
    MissingService,
    InvalidAccount,
    InvalidWebsite,
    InvalidUri,
    UnsupportedOtpType,
    UnsupportedParameter,
    DuplicateParameter,
    ConflictingIssuer,
    InvalidAlgorithm,
    InvalidDigits,
    InvalidPeriod,
    InvalidRecord,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ClockUnavailable => "Check the Windows date and time, then try again.",
            Self::CaptureUnavailable => {
                "Screen capture is unavailable. Try an image or manual entry."
            }
            Self::InvalidSelection => "Select a non-empty region within the display.",
            Self::StorageUnavailable => {
                "The vault could not be accessed. Your existing file is unchanged."
            }
            Self::InvalidSecret => "Enter a non-empty, valid Base32 secret key.",
            Self::MissingService => "Enter a service name without control characters (up to 256 bytes).",
            Self::InvalidAccount => "Enter an account identifier without control characters (up to 320 bytes).",
            Self::InvalidWebsite => "Enter an HTTP or HTTPS website address without embedded credentials.",
            Self::InvalidUri => "This is not a valid TOTP credential URI. Try manual entry.",
            Self::UnsupportedOtpType => "Only otpauth TOTP credentials are supported. HOTP is not supported.",
            Self::UnsupportedParameter => "This credential contains an unsupported parameter. Review it using manual entry.",
            Self::DuplicateParameter => "This credential repeats a parameter. Use an unambiguous credential URI.",
            Self::ConflictingIssuer => "The credential's issuer and label disagree. Review the service name using manual entry.",
            Self::InvalidAlgorithm => "Choose SHA-1, SHA-256 or SHA-512.",
            Self::InvalidDigits => "Choose 6 or 8 digits.",
            Self::InvalidPeriod => "Choose a period of 15, 30, 60 or 120 seconds.",
            Self::InvalidRecord => "The record is inconsistent. Your existing data is unchanged.",
        })
    }
}

impl std::error::Error for AppError {}
