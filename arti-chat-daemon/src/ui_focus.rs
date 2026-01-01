//! Has static variable UI_FOCUSSED indicating if the user has the app open + focussed.
//! We use this to only show a notification when the user is not actively using the app.

use std::sync::atomic::{AtomicBool, Ordering};

/// State representing if the user is focussing on the UI.
pub static UI_FOCUSSED: AtomicBool = AtomicBool::new(true);

/// Update state.
pub fn set_focussed(val: bool) {
    UI_FOCUSSED.store(val, Ordering::Relaxed);
}

/// Get state.
pub fn is_focussed() -> bool {
    UI_FOCUSSED.load(Ordering::Relaxed)
}
