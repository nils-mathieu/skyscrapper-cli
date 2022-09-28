use std::sync::atomic::{AtomicBool, Ordering};

static OCCURED: AtomicBool = AtomicBool::new(false);

/// Initializes the CTRL+C handler.
pub fn initialize() {
    ctrlc::set_handler(|| OCCURED.store(true, Ordering::Relaxed)).unwrap();
}

/// Returns whether the interrupt signal has been recieved.
#[inline]
pub fn occured() -> bool {
    OCCURED.load(Ordering::Relaxed)
}
