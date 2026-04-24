//! Renewal hazard — every living cell carries a countdown timer; on
//! expiry the cell heals back to its `starting` HP and the timer resets.
//! Each stack shortens the timer multiplicatively with diminishing
//! returns: `duration = base * (1 - frac)^(stacks - 1)`.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    cells::components::Cell,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

/// Per-run tuning extracted from [`HazardTuning::Renewal`].
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct RenewalConfig {
    /// Countdown duration at stack 1 (seconds).
    pub(crate) base_period_secs:         f32,
    /// Per-stack fractional reduction applied multiplicatively (diminishing).
    pub(crate) per_level_reduction_frac: f32,
}

impl RenewalConfig {
    /// Timer duration for the given stack count. `base * (1 - frac) ^
    /// (stacks - 1)`. Returns 0.0 at stack 0.
    #[must_use]
    pub(crate) fn duration_secs(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1).cast_signed();
        let factor = (1.0 - self.per_level_reduction_frac).max(0.0);
        self.base_period_secs * factor.powi(extra)
    }
}

/// Per-cell countdown for Renewal. Ticks down each tick; on expiry the
/// cell heals to its `starting` HP and the timer resets using the
/// *current* stack count's duration.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct RenewalTimer {
    /// Seconds remaining until the next regen cycle. Ticks down in
    /// `renewal_tick`; on reaching `<= 0.0` the cell emits a heal and the
    /// field is reset to `duration_secs(current_stacks)`.
    pub(crate) remaining: f32,
}

/// Inserts `RenewalConfig` from `HazardTuning::Renewal`. Called each time
/// the player picks Renewal; last write wins (overwrites any prior config).
/// Warns and no-ops on a non-Renewal tuning variant.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Renewal {
        base_period_secs,
        per_level_reduction_frac,
    } = *tuning
    else {
        warn!("renewal::activate called with non-Renewal tuning");
        return;
    };
    commands.insert_resource(RenewalConfig {
        base_period_secs,
        per_level_reduction_frac,
    });
}

/// Registers the Renewal `FixedUpdate` chain: `renewal_attach_timers`
/// runs first to stamp a `RenewalTimer` on every living cell that lacks
/// one, then `renewal_tick` ticks every timer and emits one
/// `HealDealt<Cell>` per damaged cell on expiry. `renewal_tick` is
/// ordered `.after(DmgSystems::ApplyKill)` (so it never acts
/// on cells that died this tick) and `.before(DmgSystems::ApplyHeal)`
/// (so the emitted heals feed `apply_heal::<Cell>` in the same tick).
/// Both systems gated by `hazard_active(Renewal)` and
/// `NodeState::Playing`.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            renewal_attach_timers,
            renewal_tick
                .after(DmgSystems::ApplyKill)
                .in_set(DmgSystems::EmitHeal),
        )
            .chain()
            .run_if(hazard_active(HazardKind::Renewal))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Attaches a `RenewalTimer` to every Cell that lacks one. Handles both
/// node-start (cells spawned first) and dynamically-spawned cells (e.g.
/// Echo Cells / Fracture).
pub(crate) fn renewal_attach_timers(
    active: Res<ActiveHazards>,
    config: Option<Res<RenewalConfig>>,
    cells: Query<Entity, (With<Cell>, Without<RenewalTimer>)>,
    mut commands: Commands,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Renewal);
    let duration = config.duration_secs(stacks);
    if duration <= 0.0 {
        return;
    }
    for entity in &cells {
        commands.entity(entity).insert(RenewalTimer {
            remaining: duration,
        });
    }
}

/// Ticks each cell's `RenewalTimer` down; on expiry heals the cell back
/// to `starting` HP and resets `remaining` to the current duration.
pub(crate) fn renewal_tick(
    time: Res<Time<Fixed>>,
    active: Res<ActiveHazards>,
    config: Option<Res<RenewalConfig>>,
    mut cells: Query<(Entity, &mut RenewalTimer, &Hp), With<Cell>>,
    mut writer: MessageWriter<HealDealt<Cell>>,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Renewal);
    let duration = config.duration_secs(stacks);
    // Consistent with `renewal_attach_timers`: if the current duration is
    // 0.0 (e.g. `per_level_reduction_frac >= 1.0` at stack >= 2), Renewal
    // is functionally inactive. Without this guard a zero-reset would
    // re-expire every tick, spamming heals at frame rate.
    if duration <= 0.0 {
        return;
    }
    let dt = time.delta_secs();

    for (entity, mut timer, hp) in &mut cells {
        if hp.current <= 0.0 {
            continue;
        }
        timer.remaining -= dt;
        if timer.remaining > 0.0 {
            continue;
        }
        let missing = hp.starting - hp.current;
        if missing > 0.0 {
            writer.write(HealDealt::<Cell> {
                healer:        None,
                attributed_to: None,
                target:        entity,
                amount:        missing,
                cap:           HealCap::Starting,
                source:        Some(SourceId::from("hazard:renewal")),
                _marker:       PhantomData,
            });
        }
        timer.remaining = duration;
    }
}
