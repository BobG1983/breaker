//! `DamageBoostStack` component — a `Vec`-backed collection of dealer-side
//! damage multipliers, append-semantic (no de-duplication of duplicate
//! sources). Callers pair a persistent multiplier with a `SourceId` so they
//! can later retract only their own contribution; one-shot multipliers are
//! drained on the next aggregate call.

mod component;

#[cfg(test)]
mod tests;

pub use component::DamageBoostStack;
