//! Drift hazard — ambient wind pushes bolts in a telegraphed direction.
//! Direction changes every `period_secs`; magnitude scales linearly with
//! stack count. The hazard emits one `ApplyBoltForce` message per active
//! Bolt each tick; the bolt domain's `apply_bolt_forces` consumer drains
//! the messages and writes `force * dt` to each bolt's `Velocity2D`.

use bevy::prelude::*;
use rand::Rng;

use crate::{
    bolt::{components::Bolt, messages::ApplyBoltForce, sets::BoltSystems},
    mutators::hazards::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::rng::HazardRng,
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
/// direction via the seeded `HazardRng` — deterministic initial wind for
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
/// `HazardRng` and resets the timer to `period_secs`. `drift_apply_force`
/// emits one `ApplyBoltForce { bolt, force }` per active Bolt each tick;
/// the bolt domain's `apply_bolt_forces` consumer (running in
/// `BoltSystems::ApplyForces`) drains the messages and writes
/// `force * dt` to each bolt's `Velocity2D`. The chain is ordered
/// `.before(BoltSystems::ApplyForces)` so the consumer drains the
/// emitter's messages within the same `FixedUpdate` tick.
pub(crate) fn wire(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (drift_update_wind, drift_apply_force)
            .chain()
            .before(BoltSystems::ApplyForces)
            .run_if(hazard_active(HazardKind::Drift))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Ticks the wind timer; when it expires, rolls a new random unit vector
/// via `HazardRng` (seeded for deterministic replay) and resets the timer
/// to `period_secs`. Early-returns (no-op) when any of `DriftConfig`,
/// `DriftWind`, or `HazardRng` is absent from the world — the gate in
/// `wire` ensures they are present during normal play.
pub(crate) fn drift_update_wind(
    time: Res<Time<Fixed>>,
    config: Option<Res<DriftConfig>>,
    wind: Option<ResMut<DriftWind>>,
    rng: Option<ResMut<HazardRng>>,
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

/// Emits one `ApplyBoltForce { bolt, force }` per active Bolt each tick.
/// `force = wind.direction * force_magnitude(stacks)` in world-units/s²
/// (acceleration). The bolt-domain consumer (`apply_bolt_forces`) drains
/// the messages, sums per-bolt forces, and performs the single `force * dt`
/// velocity write. The emitter MUST NOT pre-multiply by dt.
///
/// Early-returns (no messages emitted) when:
/// - `DriftConfig` is absent,
/// - `DriftWind` is absent, or
/// - `force_magnitude(stacks) <= 0.0` (zero stacks, zero base, or
///   negative tuning — silenced uniformly).
pub(crate) fn drift_apply_force(
    active: Res<ActiveHazards>,
    config: Option<Res<DriftConfig>>,
    wind: Option<Res<DriftWind>>,
    bolts: Query<Entity, With<Bolt>>,
    mut writer: MessageWriter<ApplyBoltForce>,
) {
    let (Some(config), Some(wind)) = (config, wind) else {
        return;
    };
    let stacks = active.stacks(HazardKind::Drift);
    let force_magnitude = config.force_magnitude(stacks);
    if force_magnitude <= 0.0 {
        return;
    }
    let force = wind.direction * force_magnitude;
    for bolt in &bolts {
        writer.write(ApplyBoltForce { bolt, force });
    }
}
