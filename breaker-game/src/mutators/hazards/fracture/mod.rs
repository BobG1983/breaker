//! Fracture hazard — destroyed cells spawn low-HP debris cells in
//! adjacent positions. Stacking increases the debris count per death
//! (count only; debris HP stays at 1). Positions are world-space offsets
//! from the victim; the cells domain resolves any overlap naturally.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, wire};
