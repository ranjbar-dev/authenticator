use crate::error::AppError;
use crate::{
    platform::lifecycle::LifecycleWatch,
    qr::{QrError, ScanReview},
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

#[cfg(windows)]
mod windows;

/// Physical monitor rectangles, in the dedicated PMv2 coordinate context.
#[cfg(windows)]
pub fn display_bounds() -> Result<Vec<DesktopBounds>, CaptureError> {
    windows::display_bounds()
}
#[cfg(not(windows))]
pub fn display_bounds() -> Result<Vec<DesktopBounds>, CaptureError> {
    Err(CaptureError::Unavailable)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Region {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
}
impl Region {
    /// Screen coordinates are physical signed pixels. Clip both drag directions
    /// before converting sizes; desktop holes are checked against the monitors.
    pub fn from_drag(a: Point, b: Point, desktop: DesktopBounds) -> Result<Self, CaptureError> {
        let desktop = desktop.validate().map_err(|_| CaptureError::Unavailable)?;
        let left = a.x.min(b.x).max(desktop.left);
        let top = a.y.min(b.y).max(desktop.top);
        let right = a.x.max(b.x).min(desktop.left + desktop.width);
        let bottom = a.y.max(b.y).min(desktop.top + desktop.height);
        if right <= left || bottom <= top {
            return Err(CaptureError::EmptySelection);
        }
        let region = Self {
            left,
            top,
            width: (i64::from(right) - i64::from(left)) as u32,
            height: (i64::from(bottom) - i64::from(top)) as u32,
        };
        region.byte_len()?;
        Ok(region)
    }
    /// Run before *any* GDI or owned pixel allocation, including overflow checks.
    pub fn byte_len(self) -> Result<usize, CaptureError> {
        let count = (self.width as usize)
            .checked_mul(self.height as usize)
            .ok_or(CaptureError::TooLarge)?;
        if self.width == 0 || self.height == 0 {
            return Err(CaptureError::EmptySelection);
        }
        if self.width > crate::qr::MAX_DIMENSION
            || self.height > crate::qr::MAX_DIMENSION
            || count > crate::qr::MAX_PIXELS
        {
            return Err(CaptureError::TooLarge);
        }
        self.left
            .checked_add_unsigned(self.width)
            .ok_or(CaptureError::Unavailable)?;
        self.top
            .checked_add_unsigned(self.height)
            .ok_or(CaptureError::Unavailable)?;
        count.checked_mul(4).ok_or(CaptureError::TooLarge)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureError {
    Unavailable,
    DisplayChanged,
    EmptySelection,
    TooLarge,
}
impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Unavailable => "Windows could not capture this region. Retry, choose an image or use manual entry.",
            Self::DisplayChanged => "The display layout changed. Start a new selection, choose an image or use manual entry.",
            Self::EmptySelection => "Select a non-empty region around the QR code. Retry, choose an image or use manual entry.",
            Self::TooLarge => "Select a smaller region: at most 4096 pixels per side and 4,194,304 pixels in total.",
        })
    }
}
#[derive(Debug)]
pub enum CaptureOutcome {
    Review(ScanReview),
    Cancelled,
    Locked,
    CaptureFailure(CaptureError),
    DecodeFailure(QrError),
}

struct Shared {
    cancelled: AtomicBool,
    decoding: AtomicBool,
    region: Mutex<Option<Region>>,
    result: Mutex<Option<CaptureOutcome>>,
}
/// Owned operation. Cancellation also drops an already-queued review, under the
/// same mutex used by the producer; no stale secret-bearing result can escape.
pub struct CaptureTask {
    shared: Arc<Shared>,
    watch: Option<LifecycleWatch>,
    deadline: Option<Instant>,
}
#[derive(Clone)]
pub(super) struct Gate {
    shared: Arc<Shared>,
    watch: LifecycleWatch,
    deadline: Instant,
}
impl Gate {
    fn locked(&self) -> bool {
        self.watch.changed() || Instant::now() >= self.deadline
    }
    fn stopped(&self) -> bool {
        self.locked() || self.shared.cancelled.load(Ordering::SeqCst)
    }
    fn stop_outcome(&self) -> CaptureOutcome {
        if self.locked() {
            CaptureOutcome::Locked
        } else {
            CaptureOutcome::Cancelled
        }
    }
    fn publish(&self, outcome: CaptureOutcome) {
        let mut slot = self.shared.result.lock().unwrap_or_else(|e| e.into_inner());
        *slot = Some(if self.stopped() {
            self.stop_outcome()
        } else {
            outcome
        });
    }
}
impl CaptureTask {
    /// `owner` is the existing ROOT HWND, never an arbitrary desktop window.
    /// The shell hides/restores ROOT via viewport commands; the adapter does not
    /// restore a potentially stale unlocked framebuffer after a native lock.
    pub fn start(
        owner: usize,
        watch: LifecycleWatch,
        deadline: Instant,
        repaint: impl Fn() + Send + 'static,
    ) -> Self {
        let shared = Arc::new(Shared {
            cancelled: AtomicBool::new(false),
            decoding: AtomicBool::new(false),
            region: Mutex::new(None),
            result: Mutex::new(None),
        });
        let gate = Gate {
            shared: shared.clone(),
            watch: watch.clone(),
            deadline,
        };
        std::thread::spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                #[cfg(windows)]
                {
                    windows::select_and_decode(owner, &gate, &repaint)
                }
                #[cfg(not(windows))]
                {
                    let _ = owner;
                    CaptureOutcome::CaptureFailure(CaptureError::Unavailable)
                }
            }))
            .unwrap_or(CaptureOutcome::CaptureFailure(CaptureError::Unavailable));
            gate.publish(outcome);
            repaint();
        });
        Self {
            shared,
            watch: Some(watch),
            deadline: Some(deadline),
        }
    }
    pub fn cancel(&self) {
        let mut slot = self.shared.result.lock().unwrap_or_else(|e| e.into_inner());
        self.shared.cancelled.store(true, Ordering::SeqCst);
        if slot.is_some() {
            *slot = Some(CaptureOutcome::Cancelled);
        }
    }
    pub fn poll(&self) -> Option<CaptureOutcome> {
        let mut result = self.shared.result.lock().unwrap_or_else(|e| e.into_inner());
        if self.watch.as_ref().is_some_and(LifecycleWatch::changed)
            || self.deadline.is_some_and(|d| Instant::now() >= d)
        {
            self.shared.cancelled.store(true, Ordering::SeqCst);
            *result = None;
            return Some(CaptureOutcome::Locked);
        }
        result.take()
    }
    pub fn decoding(&self) -> bool {
        self.shared.decoding.load(Ordering::SeqCst)
    }
    pub fn region(&self) -> Option<Region> {
        *self.shared.region.lock().unwrap_or_else(|e| e.into_inner())
    }
    #[cfg(test)]
    pub(crate) fn queued(outcome: Option<CaptureOutcome>) -> Self {
        Self {
            watch: None,
            deadline: None,
            shared: Arc::new(Shared {
                cancelled: AtomicBool::new(false),
                decoding: AtomicBool::new(false),
                region: Mutex::new(None),
                result: Mutex::new(outcome),
            }),
        }
    }
}
impl Drop for CaptureTask {
    fn drop(&mut self) {
        self.cancel();
    }
}

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
    fn drag_directions_negative_origin_clipping_and_exact_limits() -> Result<(), CaptureError> {
        let bounds = DesktopBounds {
            left: -1920,
            top: -300,
            width: 4480,
            height: 1740,
        };
        for (a, b) in [
            (Point { x: -1800, y: -200 }, Point { x: -800, y: 500 }),
            (Point { x: -800, y: 500 }, Point { x: -1800, y: -200 }),
            (Point { x: -1800, y: 500 }, Point { x: -800, y: -200 }),
        ] {
            assert_eq!(
                Region::from_drag(a, b, bounds)?,
                Region {
                    left: -1800,
                    top: -200,
                    width: 1000,
                    height: 700
                }
            );
        }
        assert_eq!(
            Region::from_drag(
                Point { x: -3000, y: -500 },
                Point { x: -1900, y: -280 },
                bounds
            )?,
            Region {
                left: -1920,
                top: -300,
                width: 20,
                height: 20
            }
        );
        assert_eq!(
            Region::from_drag(Point { x: 1, y: 1 }, Point { x: 1, y: 2 }, bounds),
            Err(CaptureError::EmptySelection)
        );
        assert_eq!(
            Region::from_drag(
                Point { x: 3000, y: 2000 },
                Point { x: 5000, y: 3000 },
                bounds
            ),
            Err(CaptureError::EmptySelection)
        );
        assert_eq!(
            Region {
                left: 0,
                top: 0,
                width: 2048,
                height: 2048
            }
            .byte_len()?,
            16_777_216
        );
        assert_eq!(
            Region {
                left: 0,
                top: 0,
                width: 4096,
                height: 1024
            }
            .byte_len()?,
            16_777_216
        );
        for (width, height) in [(4097, 1), (3840, 2160), (2048, 2049), (u32::MAX, u32::MAX)] {
            assert_eq!(
                Region {
                    left: 0,
                    top: 0,
                    width,
                    height
                }
                .byte_len(),
                Err(CaptureError::TooLarge)
            );
        }
        assert_eq!(
            Region {
                left: i32::MAX,
                top: 0,
                width: 1,
                height: 1
            }
            .byte_len(),
            Err(CaptureError::Unavailable)
        );
        Ok(())
    }
    #[test]
    fn cancelling_an_already_queued_result_discards_it_and_is_idempotent() {
        let task = CaptureTask::queued(Some(CaptureOutcome::DecodeFailure(QrError::NoQr)));
        task.cancel();
        task.cancel();
        assert!(matches!(task.poll(), Some(CaptureOutcome::Cancelled)));
        assert!(task.poll().is_none());
        assert!(task.shared.cancelled.load(Ordering::SeqCst));
    }
    #[cfg(windows)]
    #[test]
    fn cancellation_wins_before_or_after_secret_bearing_publication()
    -> Result<(), Box<dyn std::error::Error>> {
        let listener = crate::platform::lifecycle::Lifecycle::start(|| {})?;
        for _ in 0..16 {
            let code = qrcode::QrCode::new("otpauth://totp/Public?secret=MY")?;
            let side = (code.width() + 8) * 5;
            let mut pixels = zeroize::Zeroizing::new(vec![255; side * side]);
            for y in 0..code.width() {
                for x in 0..code.width() {
                    if code[(x, y)] == qrcode::Color::Dark {
                        for dy in 0..5 {
                            for dx in 0..5 {
                                pixels[((y + 4) * 5 + dy) * side + (x + 4) * 5 + dx] = 0;
                            }
                        }
                    }
                }
            }
            let review = crate::qr::decode_pixels(
                side as u32,
                side as u32,
                crate::qr::PixelLayout::Gray8,
                pixels,
            )?;
            assert_eq!(review.supported_count(), 1);
            let task = CaptureTask::queued(None);
            let gate = Gate {
                shared: task.shared.clone(),
                watch: listener.watch(),
                deadline: Instant::now() + std::time::Duration::from_secs(60),
            };
            let barrier = Arc::new(std::sync::Barrier::new(2));
            let producer_barrier = barrier.clone();
            let producer = std::thread::spawn(move || {
                producer_barrier.wait();
                gate.publish(CaptureOutcome::Review(review));
            });
            barrier.wait();
            task.cancel();
            producer.join().map_err(|_| "producer failed")?;
            assert!(matches!(task.poll(), Some(CaptureOutcome::Cancelled)));
        }
        Ok(())
    }
    #[cfg(windows)]
    #[test]
    fn queued_result_cannot_outlive_consumed_native_lock_or_idle_deadline() -> Result<(), AppError>
    {
        let listener = crate::platform::lifecycle::Lifecycle::start(|| {})?;
        let mut task = CaptureTask::queued(Some(CaptureOutcome::DecodeFailure(QrError::NoQr)));
        task.watch = Some(listener.watch());
        listener.notify_lock_for_test();
        assert!(listener.take_lock());
        assert!(matches!(task.poll(), Some(CaptureOutcome::Locked)));
        let mut task = CaptureTask::queued(Some(CaptureOutcome::DecodeFailure(QrError::NoQr)));
        task.deadline = Some(Instant::now());
        assert!(matches!(task.poll(), Some(CaptureOutcome::Locked)));
        Ok(())
    }

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
