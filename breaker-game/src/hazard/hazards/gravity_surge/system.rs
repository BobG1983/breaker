//! `GravitySurge` hazard — destroyed cells spawn short-lived gravity wells
//! that pull the bolt. Stacking increases both the duration of each well
//! and its pull strength (linear in duration, fractional in strength).
//! Forces are applied directly to each Bolt's `Velocity2D` each tick and
//! are clamped at short range to prevent numeric blow-up.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};
use rantzsoft_stateflow::CleanupOnExit;

use crate::{
    bolt::components::Bolt,
    cells::components::Cell,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::death_pipeline::Destroyed,
};

/// Distance floor (world units) below which the pull force magnitude is
/// clamped. Prevents divide-by-zero when a bolt sits on top of a well.
const MIN_PULL_DISTANCE: f32 = 20.0;

/// Per-run tuning extracted from [`HazardTuning::GravitySurge`].
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct GravitySurgeConfig {
    /// Seconds each well lives at stack 1.
    pub(crate) base_duration_secs:      f32,
    /// Additional seconds per stack beyond the first.
    pub(crate) per_level_duration_secs: f32,
    /// Pull strength (force magnitude at `MIN_PULL_DISTANCE`) at stack 1.
    pub(crate) base_strength:           f32,
    /// Fractional per-stack increase applied multiplicatively to strength
    /// with sqrt diminishing returns: `strength * (1 + frac * sqrt(s-1))`.
    pub(crate) per_level_strength_frac: f32,
}

impl GravitySurgeConfig {
    /// Well lifetime (seconds) for the given stack count. Returns `0.0` when
    /// `stacks == 0` (hazard inactive). For `stacks >= 1`, returns
    /// `base_duration_secs + per_level_duration_secs * (stacks - 1)` —
    /// linear scaling: stack 1 is `base_duration_secs`, stack 2 is
    /// `base_duration_secs + per_level_duration_secs`, stack N is
    /// `base_duration_secs + per_level_duration_secs * (N - 1)`. `const fn`
    /// so config consumers can evaluate at compile time. Uses `mul_add` on
    /// `per_level_duration_secs.mul_add(extra, base_duration_secs)` for
    /// numerical stability.
    #[must_use]
    pub(crate) const fn duration_secs(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        self.per_level_duration_secs
            .mul_add(extra, self.base_duration_secs)
    }

    /// Pull strength (force magnitude evaluated at `MIN_PULL_DISTANCE`) for
    /// the given stack count. Returns `0.0` when `stacks == 0` (hazard
    /// inactive), **even when `base_strength > 0`**. For `stacks >= 1`,
    /// returns `base_strength * (1.0 + per_level_strength_frac * sqrt(s - 1))`
    /// where `s = stacks`. Square-root-in-stack diminishing returns: stack 1
    /// multiplier is `1.0`, stack 2 is `1.0 + per_level_strength_frac * 1.0`,
    /// stack 5 is `1.0 + per_level_strength_frac * 2.0`. Not `const fn`
    /// because `f32::sqrt` is not `const`.
    #[must_use]
    pub(crate) fn strength(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        let multiplier = self.per_level_strength_frac.mul_add(extra.sqrt(), 1.0);
        self.base_strength * multiplier
    }
}

/// A short-lived gravity well entity. Spawned at the position of a
/// destroyed cell. The `Position2D` on the same entity stores the world
/// location; `strength` is baked in at spawn time so post-spawn stack
/// changes don't retroactively alter older wells.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct GravityWell {
    pub(crate) strength:  f32,
    pub(crate) remaining: f32,
}

/// Inserts `GravitySurgeConfig` from `HazardTuning::GravitySurge {
/// base_duration_secs, per_level_duration_secs, base_strength,
/// per_level_strength_frac }`. Called each time the player picks
/// `GravitySurge` from `hazards::activate`; last write wins (overwrites any
/// prior `GravitySurgeConfig`; stack count is owned by `ActiveHazards`,
/// not the config; existing `GravityWell` entities are NOT retroactively
/// rescaled — their `strength` and `remaining` are baked in at spawn
/// time). Warns and no-ops on a non-GravitySurge tuning variant, leaving
/// any existing `GravitySurgeConfig` intact.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::GravitySurge {
        base_duration_secs,
        per_level_duration_secs,
        base_strength,
        per_level_strength_frac,
    } = *tuning
    else {
        warn!("gravity_surge::activate called with non-GravitySurge tuning");
        return;
    };
    commands.insert_resource(GravitySurgeConfig {
        base_duration_secs,
        per_level_duration_secs,
        base_strength,
        per_level_strength_frac,
    });
}

/// Registers the `GravitySurge` `FixedUpdate` chain: `spawn_gravity_wells`
/// → `gravity_well_pull` → `despawn_expired_gravity_wells`, gated on
/// `hazard_active(HazardKind::GravitySurge)` AND
/// `in_state(NodeState::Playing)`. `spawn_gravity_wells` reads
/// `Destroyed<Cell>` and spawns `(GravityWell, Position2D)` with
/// snapshotted `duration`/`strength`. `gravity_well_pull` ticks each
/// well's `remaining`, sums inverse-linear pulls clamped at
/// `MIN_PULL_DISTANCE`, and writes the accumulated impulse to every
/// Bolt's `Velocity2D`. `despawn_expired_gravity_wells` removes wells
/// whose `remaining` has dropped to or below zero. No
/// `DeathPipelineSystems` ordering — `GravitySurge` operates on bolt
/// `Velocity2D` directly as a **deferred architectural exception** not
/// yet covered by `docs/architecture/plugins.md` § `Velocity2D`
/// Cross-Domain Write Exception. Commit 5 / Wave 7 retrofits
/// `gravity_well_pull` to publish `ApplyBoltForce` messages instead,
/// at which point the bolt domain owns velocity integration and this
/// exception resolves.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            spawn_gravity_wells,
            gravity_well_pull,
            despawn_expired_gravity_wells,
        )
            .chain()
            .run_if(hazard_active(HazardKind::GravitySurge))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Spawns a `GravityWell` at every `Destroyed<Cell>` position. Duration
/// and strength are snapshotted at spawn time from current stack count —
/// post-spawn stack changes do NOT retroactively alter older wells.
/// Early-returns (draining the message reader via `reader.clear()`) when
/// `GravitySurgeConfig` is absent, or when the computed `duration <= 0.0`
/// or `strength <= 0.0` (zero-stack inactive path) — the drain prevents
/// stale `Destroyed<Cell>` messages from spawning wells on a later tick
/// after the hazard activates.
pub(crate) fn spawn_gravity_wells(
    mut reader: MessageReader<Destroyed<Cell>>,
    active: Res<ActiveHazards>,
    config: Option<Res<GravitySurgeConfig>>,
    mut commands: Commands,
) {
    let Some(config) = config else {
        reader.clear();
        return;
    };
    let stacks = active.stacks(HazardKind::GravitySurge);
    let duration = config.duration_secs(stacks);
    let strength = config.strength(stacks);
    if duration <= 0.0 || strength <= 0.0 {
        reader.clear();
        return;
    }
    for destroyed in reader.read() {
        commands.spawn((
            GravityWell {
                strength,
                remaining: duration,
            },
            Position2D(destroyed.victim_pos),
            // Node-end cleanup: stale wells must not leak across nodes.
            // Matches the Resonance precedent; a pre-existing leak bug.
            CleanupOnExit::<NodeState>::default(),
        ));
    }
}

/// Applies the summed pull from every active well to each bolt's
/// `Velocity2D`. Also ticks each well's `remaining` down. Pull falloff
/// is inverse-linear (`strength / distance`) with the distance clamped at
/// `MIN_PULL_DISTANCE` (20.0) to prevent numeric blow-up when a bolt sits
/// on top of a well. Collects a `Vec<(Vec2, f32)>` snapshot of still-live
/// wells after the tick-down pass so expired wells do not contribute to
/// this frame's pull. Early-returns when the snapshot is empty — no
/// active wells means no impulse work to do.
pub(crate) fn gravity_well_pull(
    time: Res<Time<Fixed>>,
    mut wells: Query<(&mut GravityWell, &Position2D)>,
    mut bolts: Query<(&Position2D, &mut Velocity2D), With<Bolt>>,
) {
    let dt = time.delta_secs();
    // Collect well snapshots and tick remaining.
    let mut snapshots: Vec<(Vec2, f32)> = Vec::new();
    for (mut well, pos) in &mut wells {
        well.remaining -= dt;
        if well.remaining > 0.0 {
            snapshots.push((pos.0, well.strength));
        }
    }
    if snapshots.is_empty() {
        return;
    }
    for (bolt_pos, mut vel) in &mut bolts {
        let mut accel = Vec2::ZERO;
        for (well_pos, strength) in &snapshots {
            let delta = *well_pos - bolt_pos.0;
            let distance = delta.length().max(MIN_PULL_DISTANCE);
            let direction = delta / distance;
            // Inverse-linear falloff, clamped at the distance floor.
            accel += direction * (*strength / distance);
        }
        vel.0 += accel * dt;
    }
}

/// Despawns wells whose `remaining` has dropped to or below zero. Runs
/// after `gravity_well_pull` in the chain. `gravity_well_pull` decrements
/// `remaining` by `dt` BEFORE snapshotting the active wells, so a well
/// entering the tick at `remaining == 0.0` decrements to negative, is
/// excluded from that tick's snapshot (no pull contribution), and is
/// despawned here. Early-returns implicitly when no wells are expired —
/// the query iteration simply finds none.
pub(crate) fn despawn_expired_gravity_wells(
    wells: Query<(Entity, &GravityWell)>,
    mut commands: Commands,
) {
    for (entity, well) in &wells {
        if well.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
