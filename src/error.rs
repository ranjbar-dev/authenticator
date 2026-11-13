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
    InvalidVault,
    UnsupportedVault,
    VaultLocked,
    VaultBusy,
    InvalidPassphrase,
    InvalidBackupCodes,
    ReviewBackupDuplicates,
    StaleAccount,
    StaleBackupCodes,
    BackupFileUnavailable,
    BackupFileTooLarge,
    InvalidBackupEncoding,
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
            Self::InvalidVault => "The passphrase is incorrect or the vault is damaged. Your existing file is unchanged.",
            Self::UnsupportedVault => "This vault version is unsupported. Use a compatible app version.",
            Self::VaultLocked => "Unlock the vault to continue.",
            Self::VaultBusy => "The vault is open in another instance. Close it and try again.",
            Self::InvalidPassphrase => "Use a passphrase between 12 and 1024 UTF-8 bytes.",
            Self::InvalidBackupCodes => "Enter one backup code per line: up to 1 MiB, 10,000 nonblank lines and 4096 bytes per code. Internal control characters are not allowed.",
            Self::ReviewBackupDuplicates => "Review and acknowledge removal of exact duplicate backup codes before saving.",
            Self::StaleAccount => "This account changed or is no longer active. Cancel and reopen the account to edit it safely.",
            Self::StaleBackupCodes => "This backup-code set changed or is in Trash. Cancel and reopen it before making changes.",
            Self::BackupFileUnavailable => "The local text file could not be read. Choose a regular local UTF-8 file or enter codes manually. Your input is kept.",
            Self::BackupFileTooLarge => "Choose a UTF-8 text file no larger than 1 MiB. Your input and stored codes are kept.",
            Self::InvalidBackupEncoding => "This file is not valid UTF-8. Save it as UTF-8 text, with an optional BOM, and retry. Your input is kept.",
        })
    }
}

impl std::error::Error for AppError {}
