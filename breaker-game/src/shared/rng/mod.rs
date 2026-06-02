//! Deterministic RNG for gameplay randomness.

pub(crate) mod rng_types;

#[cfg(test)]
mod tests;

pub(crate) use rng_types::seed_node_rng;
pub use rng_types::{
    BoltRng, ChipRng, ChipSelectCount, EffectBaseSeed, EffectEventCounter, FxRng, GameRng,
    HazardRng, NodeGenRng, NodeSequenceRng, ProtocolRng, SeedNodeRngSystems, derive_seed,
    derive_seed_named,
};
