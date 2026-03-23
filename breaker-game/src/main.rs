//! Brickbreaker — a roguelite Arkanoid clone.

// Suppress the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Entry point.
fn main() {
    let mut app = breaker::app::build_app();

    #[cfg(feature = "dev")]
    breaker::app::apply_dev_flags(&mut app);

    app.run();
}
