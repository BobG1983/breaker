//! Shared resources used across multiple domain plugins.

use bevy::prelude::*;

/// Optional seed for deterministic RNG at run start.
///
/// `None` means random (OS entropy). `Some(n)` seeds the [`GameRng`] with
/// the given value for deterministic replays.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSeed(pub Option<u64>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_seed_default_is_none() {
        let seed = RunSeed::default();
        assert_eq!(seed.0, None);
    }

    #[test]
    fn run_seed_some_holds_value() {
        let seed = RunSeed(Some(12345));
        assert_eq!(seed.0, Some(12345));
    }
}
