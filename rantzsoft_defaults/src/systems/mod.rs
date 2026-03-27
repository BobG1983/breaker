//! Generic systems for seeding and propagating config resources from defaults
//! assets.

pub use fns::*;

mod fns;

#[cfg(test)]
#[allow(clippy::expect_used, reason = "tests panic on failure")]
mod tests;
