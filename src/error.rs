use std::fmt;

/// Fixed public messages only. Never attach source payloads, paths or credentials.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppError {
    ClockUnavailable,
    CaptureUnavailable,
    InvalidSelection,
    StorageUnavailable,
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
        })
    }
}

impl std::error::Error for AppError {}
