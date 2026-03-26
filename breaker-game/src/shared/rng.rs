//! Deterministic RNG for gameplay randomness.

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Deterministic RNG for gameplay randomness.
///
/// Initialized at app start with a fixed seed (deterministic for tests).
/// Reseeded at run start by `reset_run_state` using [`RunSeed`]
/// (user-controlled) or OS entropy when no seed is set.
#[derive(Resource)]
pub struct GameRng(pub ChaCha8Rng);

impl GameRng {
    /// Creates a `GameRng` with a specific seed. Useful for tests.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::from_seed(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_rng_from_seed_is_deterministic() {
        use rand::Rng;
        let mut rng1 = GameRng::from_seed(42);
        let mut rng2 = GameRng::from_seed(42);
        let v1: f32 = rng1.0.random();
        let v2: f32 = rng2.0.random();
        assert!((v1 - v2).abs() < f32::EPSILON);
    }
}
