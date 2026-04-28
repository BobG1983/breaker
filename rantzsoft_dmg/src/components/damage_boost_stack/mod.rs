//! `DamageBoostStack` component — a `Vec`-backed collection of dealer-side
//! outgoing-damage multipliers. Dealer-side companion of `VulnerableStack`
//! (damage dealt rather than damage received). Append-semantic (no
//! de-duplication of duplicate sources); callers pair each persistent
//! multiplier with a `SourceId` so they can later retract only their own
//! contribution, and one-shot multipliers are drained on the next aggregate
//! call.

mod component;

#[cfg(test)]
mod tests;

pub use component::DamageBoostStack;
