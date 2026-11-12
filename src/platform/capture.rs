use crate::error::AppError;

/// Physical virtual-desktop pixels; left/top may be negative.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopBounds {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

impl DesktopBounds {
    pub fn validate(self) -> Result<Self, AppError> {
        if self.width <= 0 || self.height <= 0 {
            return Err(AppError::CaptureUnavailable);
        }
        self.left
            .checked_add(self.width)
            .ok_or(AppError::CaptureUnavailable)?;
        self.top
            .checked_add(self.height)
            .ok_or(AppError::CaptureUnavailable)?;
        Ok(self)
    }
}

/// Feasibility probe only: reads display geometry, never screen pixels.
#[cfg(windows)]
pub fn desktop_bounds() -> Result<DesktopBounds, AppError> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };
    // SAFETY: GetSystemMetrics accepts these documented constants and no pointers.
    unsafe {
        DesktopBounds {
            left: GetSystemMetrics(SM_XVIRTUALSCREEN),
            top: GetSystemMetrics(SM_YVIRTUALSCREEN),
            width: GetSystemMetrics(SM_CXVIRTUALSCREEN),
            height: GetSystemMetrics(SM_CYVIRTUALSCREEN),
        }
    }
    .validate()
}

#[cfg(not(windows))]
pub fn desktop_bounds() -> Result<DesktopBounds, AppError> {
    Err(AppError::CaptureUnavailable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_desktop_coordinates_are_valid() -> Result<(), AppError> {
        let b = DesktopBounds {
            left: -1920,
            top: -300,
            width: 3840,
            height: 2160,
        };
        assert_eq!(b.validate()?, b);
        Ok(())
    }

    #[test]
    fn rejects_empty_and_overflowing_geometry() {
        for b in [
            DesktopBounds {
                left: 0,
                top: 0,
                width: 0,
                height: 10,
            },
            DesktopBounds {
                left: i32::MAX,
                top: 0,
                width: 1,
                height: 10,
            },
        ] {
            assert_eq!(b.validate(), Err(AppError::CaptureUnavailable));
        }
    }
}
