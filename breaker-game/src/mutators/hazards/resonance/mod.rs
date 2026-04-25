//! Resonance hazard module — production code lives in `system`, tests in
//! `tests/`. Parent `hazards::mod` references `resonance::activate` and
//! `resonance::wire` directly — both are re-exported below.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, wire};
