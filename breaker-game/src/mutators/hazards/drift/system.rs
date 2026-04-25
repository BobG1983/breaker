//! Drift hazard — ambient wind pushes bolts in a telegraphed direction.
//! Direction changes every `period_secs`; magnitude scales linearly with
//! stack count. The hazard writes directly to each Bolt's `Velocity2D`
//! each tick (force × dt), keeping the implementation self-contained.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::components::Bolt,
    mutators::hazards::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

/// Per-run tuning extracted from [`HazardTuning::Drift`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DriftConfig {
    /// Force magnitude (world-space units / second²) at stack 1.
    pub(crate) force:           f32,
    /// Seconds between direction changes. Does not scale with stacks.
    pub(crate) period_secs:     f32,
    /// Additional force magnitude per stack beyond the first.
    pub(crate) per_level_force: f32,
}

impl DriftConfig {
    /// Effective force magnitude (world-units per second²) for the given stack
    /// count. Returns `0.0` when `stacks == 0` (hazard inactive). For
    /// `stacks >= 1`, returns `force + per_level_force * (stacks - 1)` —
    /// linear scaling: stack 1 is `force`, stack 2 is `force + per_level_force`,
    /// stack N is `force + per_level_force * (N - 1)`. `const fn` so config
    /// consumers can evaluate at compile time.
    #[must_use]
    pub(crate) const fn force_magnitude(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        self.per_level_force.mul_add(extra, self.force)
    }
}

/// Current wind state. There is one global wind direction affecting all
/// bolts; individual bolts experience different outcomes only via their
/// existing velocities.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DriftWind {
    pub(crate) direction: Vec2,
    /// Seconds remaining until the next direction change.
    pub(crate) timer:     f32,
}

impl Default for DriftWind {
    fn default() -> Self {
        Self {
            direction: Vec2::X,
            timer:     0.0,
        }
    }
}

/// Inserts `DriftConfig` and `DriftWind` from `HazardTuning::Drift { force,
/// period_secs, per_level_force }`. Called each time the player picks Drift
/// from `hazards::activate`; last write wins (overwrites any prior
/// `DriftConfig` and resets `DriftWind` to `{ direction: Vec2::X,
/// timer: 0.0 }` so the first `FixedUpdate` tick re-rolls a fresh
/// direction via the seeded `GameRng` — deterministic initial wind for
/// replay and scenario reproducibility). Warns and no-ops on a non-Drift
/// tuning variant, leaving any existing resources intact.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Drift {
        force,
        period_secs,
        per_level_force,
    } = *tuning
    else {
        warn!("drift::activate called with non-Drift tuning");
        return;
    };
    commands.insert_resource(DriftConfig {
        force,
        period_secs,
        per_level_force,
    });
    commands.insert_resource(DriftWind {
        direction: Vec2::X,
        // Timer starts at 0 so the first tick re-rolls direction via the
        // seeded RNG — deterministic initial wind for replay/scenarios.
        timer:     0.0,
    });
}

/// Registers the Drift `FixedUpdate` chain: `drift_update_wind` →
/// `drift_apply_force`, gated on `hazard_active(HazardKind::Drift)` AND
/// `in_state(NodeState::Playing)`. `drift_update_wind` ticks the
/// `DriftWind.timer` down; on expiry rolls a new unit-vector direction via
/// `GameRng` and resets the timer to `period_secs`. `drift_apply_force`
/// adds `direction * force_magnitude(stacks) * dt` to every Bolt's
/// `Velocity2D`. No `DmgSystems` ordering — Drift operates on
/// bolt `Velocity2D` directly (pending the `ApplyBoltForce` pipeline in
/// Commit 5).
pub(crate) fn wire(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (drift_update_wind, drift_apply_force)
            .chain()
            .run_if(hazard_active(HazardKind::Drift))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Ticks the wind timer; when it expires, rolls a new random unit vector
/// via `GameRng` (seeded for deterministic replay) and resets the timer
/// to `period_secs`. Early-returns (no-op) when any of `DriftConfig`,
/// `DriftWind`, or `GameRng` is absent from the world — the gate in
/// `wire` ensures they are present during normal play.
pub(crate) fn drift_update_wind(
    time: Res<Time<Fixed>>,
    config: Option<Res<DriftConfig>>,
    wind: Option<ResMut<DriftWind>>,
    rng: Option<ResMut<GameRng>>,
) {
    let (Some(config), Some(mut wind), Some(mut rng)) = (config, wind, rng) else {
        return;
    };
    wind.timer -= time.delta_secs();
    if wind.timer > 0.0 {
        return;
    }
    let angle = rng.0.random_range(0.0..std::f32::consts::TAU);
    wind.direction = Vec2::new(angle.cos(), angle.sin());
    wind.timer = config.period_secs;
}

/// Adds `direction × force × dt` to every Bolt's `Velocity2D` each tick.
/// Early-returns when `DriftConfig` or `DriftWind` is absent, or when
/// `force_magnitude(stacks) <= 0.0` (stack count is zero) — avoids writing
/// a zero-length impulse to every bolt in the common inactive path.
pub(crate) fn drift_apply_force(
    time: Res<Time<Fixed>>,
    active: Res<ActiveHazards>,
    config: Option<Res<DriftConfig>>,
    wind: Option<Res<DriftWind>>,
    mut bolts: Query<&mut Velocity2D, With<Bolt>>,
) {
    let (Some(config), Some(wind)) = (config, wind) else {
        return;
    };
    let stacks = active.stacks(HazardKind::Drift);
    let force = config.force_magnitude(stacks);
    if force <= 0.0 {
        return;
    }
    let impulse = wind.direction * force * time.delta_secs();
    for mut velocity in &mut bolts {
        velocity.0 += impulse;
    }
}
