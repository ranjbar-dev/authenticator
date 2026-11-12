//! Per-token test-only actual save admission gate; absent from production builds.
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
#[derive(Default)]
pub(crate) struct CommitGate {
    state: Mutex<(bool, bool)>, // entered, released
    changed: Condvar,
}
impl CommitGate {
    pub(crate) fn hold_admitted_action(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.0 = true;
        self.changed.notify_all();
        while !state.1 {
            state = self.changed.wait(state).unwrap_or_else(|e| e.into_inner());
        }
    }
    pub(crate) fn wait_entered(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        while !state.0 {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return false;
            }
            state = self
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
        true
    }
    pub(crate) fn release(&self) {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        self.changed.notify_all();
    }
    pub(crate) fn is_released(&self) -> bool {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).1
    }
    pub(crate) fn release_on_drop(self: &Arc<Self>) -> ReleaseOnDrop {
        ReleaseOnDrop(self.clone())
    }
}
/// Must live INSIDE a scoped-thread closure so assertion unwinding releases the
/// gate before scope joins. This prevents a regression from hanging CI.
pub(crate) struct ReleaseOnDrop(Arc<CommitGate>);
impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        self.0.release();
    }
}
