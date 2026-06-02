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

// ── Seed derivation helpers ─────────────────────────────────────────────────

/// Mixes `parent` and `discriminator` into a deterministic `u64` child seed.
#[must_use]
pub const fn derive_seed(parent: u64, discriminator: u64) -> u64 {
    let mut h = parent
        .wrapping_add(discriminator)
        .wrapping_add(0x9E37_79B9_7F4A_7C15);
    h = h.wrapping_mul(0x517c_c1b7_2722_0a95);
    h ^ (h >> 32)
}

/// Mixes `parent` and a string `name` into a deterministic `u64` child seed.
#[must_use]
pub const fn derive_seed_named(parent: u64, name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut name_hash: u64 = 0;
    let mut i = 0;
    while i < bytes.len() {
        name_hash = name_hash.wrapping_mul(31).wrapping_add(bytes[i] as u64);
        i += 1;
    }
    derive_seed(parent, name_hash)
}

// ── New per-channel RNG resources ────────────────────────────────────────────

/// FX-visuals RNG — seeded from OS entropy at app startup (cosmetic-only).
#[derive(Resource)]
pub struct FxRng(pub ChaCha8Rng);

impl FxRng {
    /// Creates an `FxRng` seeded from a specific value. Used in tests.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

impl Default for FxRng {
    fn default() -> Self {
        Self(ChaCha8Rng::from_os_rng())
    }
}

/// Run-level node-sequence randomness. Used only by
/// `generate_node_sequence_system` (Wave 2E) to determine the per-tier node
/// ordering and shuffle. Seeded once at run start from
/// `derive_seed_named(run_seed, "node_sequence")`.
#[derive(Resource)]
pub struct NodeSequenceRng(pub ChaCha8Rng);

impl NodeSequenceRng {
    /// Creates a `NodeSequenceRng` from the given seed.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

impl Default for NodeSequenceRng {
    fn default() -> Self {
        Self::from_seed(0)
    }
}

/// Per-node node-generation randomness.
#[derive(Resource)]
pub struct NodeGenRng(pub ChaCha8Rng);

impl NodeGenRng {
    /// Creates a `NodeGenRng` from the given seed.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

impl Default for NodeGenRng {
    fn default() -> Self {
        Self::from_seed(0)
    }
}

/// Per-node bolt physics/angle randomness.
#[derive(Resource)]
pub struct BoltRng(pub ChaCha8Rng);

impl BoltRng {
    /// Creates a `BoltRng` from the given seed.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

impl Default for BoltRng {
    fn default() -> Self {
        Self::from_seed(0)
    }
}

/// Per-chip-event randomness.
#[derive(Resource)]
pub struct ChipRng(pub ChaCha8Rng);

impl ChipRng {
    /// Creates a `ChipRng` from the given seed.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

impl Default for ChipRng {
    fn default() -> Self {
        Self::from_seed(0)
    }
}

/// Per-protocol-event randomness.
#[derive(Resource)]
pub struct ProtocolRng(pub ChaCha8Rng);

impl ProtocolRng {
    /// Creates a `ProtocolRng` from the given seed.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

impl Default for ProtocolRng {
    fn default() -> Self {
        Self::from_seed(0)
    }
}

/// Per-hazard-event randomness.
#[derive(Resource)]
pub struct HazardRng(pub ChaCha8Rng);

impl HazardRng {
    /// Creates a `HazardRng` from the given seed.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }
}

impl Default for HazardRng {
    fn default() -> Self {
        Self::from_seed(0)
    }
}

/// Counter of effect events within the current node. Reset to 0 on
/// `OnEnter(NodeState::Loading)` by `seed_node_rng`.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EffectEventCounter(pub u64);

/// One-time effect base seed, derived from `run_seed` at run start by
/// `reset_run_state`. Stable across node transitions.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EffectBaseSeed(pub u64);

/// Number of chip selections made so far in the current run.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChipSelectCount(pub u32);

// ── SystemSet for OnEnter(NodeState::Loading) ordering ───────────────────────

/// Ordering set for `OnEnter(NodeState::Loading)` systems.
///
/// `PrepareSeed → Seed → ConsumeSeed`.
#[derive(bevy::ecs::schedule::SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeedNodeRngSystems {
    /// Runs first — populates `RunStats.seed` (contains `capture_run_seed`).
    PrepareSeed,
    /// Runs after `PrepareSeed` — seeds per-node RNG resources (contains
    /// `seed_node_rng`).
    Seed,
    /// Runs after `Seed` — consumes the seeded resources (contains
    /// `setup_run`, `reset_bolt`).
    ConsumeSeed,
}

/// Seeds `BoltRng`, `NodeGenRng`, and resets `EffectEventCounter` for the
/// current node, reading from `RunStats.seed` and `NodeOutcome.node_index`.
pub(crate) fn seed_node_rng(
    run_stats: Res<crate::state::run::resources::RunStats>,
    node_outcome: Res<crate::state::run::resources::NodeOutcome>,
    mut commands: Commands,
) {
    let node_seed = derive_seed(run_stats.seed, u64::from(node_outcome.node_index));
    let bolt_seed = derive_seed_named(node_seed, "bolt");
    let node_gen_seed = derive_seed_named(node_seed, "node_gen");

    commands.insert_resource(BoltRng::from_seed(bolt_seed));
    commands.insert_resource(NodeGenRng::from_seed(node_gen_seed));
    commands.insert_resource(EffectEventCounter(0));
}
