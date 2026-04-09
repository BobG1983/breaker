//! System to capture the run seed into `RunStats` at node start.

use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    shared::{GameRng, RunSeed},
    state::run::resources::RunStats,
};

/// Captures the [`RunSeed`] value into [`RunStats::seed`] on the first node.
///
/// If [`RunSeed`] is `None`, generates a random seed from [`GameRng`].
/// Only captures once (skips if `seed` is already non-zero).
pub(crate) fn capture_run_seed(
    seed: Res<RunSeed>,
    mut stats: ResMut<RunStats>,
    mut rng: ResMut<GameRng>,
) {
    if stats.seed != 0 {
        return;
    }
    if let Some(n) = seed.0 {
        stats.seed = n;
    } else {
        let generated: u64 = rng.0.random();
        stats.seed = generated;
        rng.0 = ChaCha8Rng::seed_from_u64(stats.seed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::test_utils::TestAppBuilder;

    fn test_app() -> App {
        TestAppBuilder::new()
            .with_resource::<RunStats>()
            .with_resource::<GameRng>()
            .with_resource::<RunSeed>()
            .with_system(Update, capture_run_seed)
            .build()
    }

    #[test]
    fn captures_specific_seed_into_stats() {
        let mut app = test_app();
        app.insert_resource(RunSeed(Some(42)));
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.seed, 42,
            "RunStats.seed should be 42 when RunSeed is Some(42)"
        );
    }

    #[test]
    fn only_captures_seed_once() {
        let mut app = test_app();
        app.insert_resource(RunSeed(Some(42)));
        app.update();

        // Change the RunSeed and run again
        app.insert_resource(RunSeed(Some(99)));
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.seed, 42,
            "RunStats.seed should remain 42 (not overwritten on second node)"
        );
    }

    #[test]
    fn generates_random_seed_when_run_seed_is_none() {
        let mut app = test_app();
        // RunSeed defaults to None
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_ne!(
            stats.seed, 0,
            "RunStats.seed should be non-zero when generated from RNG"
        );
    }
}
