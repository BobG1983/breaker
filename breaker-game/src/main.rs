//! Brickbreaker — a roguelite Arkanoid clone.

/// Entry point.
fn main() {
    let mut app = breaker::app::build_app();

    #[cfg(feature = "dev")]
    breaker::app::apply_dev_flags(&mut app);

    app.run();
}
