//! Erosion hazard — the Breaker slowly shrinks each tick; bumps restore
//! a fraction of the lost width. Shrink rate scales linearly with stack
//! count; restoration fractions stay fixed. Width is applied to the
//! Breaker's [`EffectStack<SizeBoostConfig>`] under a dedicated source
//! tag, mirroring the pattern used by Haste for bolt speed.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    breaker::{
        components::Breaker,
        messages::{BumpGrade, BumpPerformed},
    },
    effect_v3::{effects::SizeBoostConfig, stacking::EffectStack},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

/// Source tag for Erosion's entry on the Breaker's
/// [`EffectStack<SizeBoostConfig>`]. Keeps Erosion's scaling isolated
/// from chip/protocol size boosts so each can be reconciled independently.
const EROSION_SOURCE: &str = "hazard:erosion";

/// Per-run tuning extracted from [`HazardTuning::Erosion`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct ErosionConfig {
    /// Full-width fractions lost per second at stack 1. Scales linearly
    /// with stack count (stack 3 shrinks 3× as fast).
    pub(crate) shrink_rate:      f32,
    /// Floor on `width_fraction`; the breaker never shrinks below this.
    pub(crate) min_width_frac:   f32,
    /// Fraction of *lost* width restored on an Early/Late bump.
    pub(crate) restore_nonwhiff: f32,
    /// Fraction of *lost* width restored on a Perfect bump.
    pub(crate) restore_perfect:  f32,
}

/// Live per-run state — tracks the Breaker's current scale as a fraction
/// of its pristine width. 1.0 = full, `min_width_frac` = floor.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct ErosionState {
    pub(crate) width_fraction: f32,
}

impl Default for ErosionState {
    fn default() -> Self {
        Self {
            width_fraction: 1.0,
        }
    }
}

/// Inserts `ErosionConfig` and `ErosionState` from `HazardTuning::Erosion`.
/// Called each time the player picks Erosion; last write wins (overwrites
/// any prior config; state resets to full width 1.0). Warns and no-ops on
/// a non-Erosion tuning variant, leaving any existing resources intact.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Erosion {
        shrink_rate,
        min_width_frac,
        restore_nonwhiff,
        restore_perfect,
    } = *tuning
    else {
        warn!("erosion::activate called with non-Erosion tuning");
        return;
    };
    commands.insert_resource(ErosionConfig {
        shrink_rate,
        min_width_frac,
        restore_nonwhiff,
        restore_perfect,
    });
    commands.insert_resource(ErosionState::default());
}

/// Registers the Erosion `FixedUpdate` systems.
///
/// Non-reader systems — gated on `hazard_active(HazardKind::Erosion)`
/// AND `in_state(NodeState::Playing)`:
/// - `erosion_shrink` — ticks `width_fraction` down by
///   `shrink_rate * stacks * dt`, before `erosion_restore`.
/// - `erosion_apply_width` — reconciles the Breaker's
///   `EffectStack<SizeBoostConfig>` with a single
///   source-"hazard:erosion" entry, after `erosion_restore`.
///
/// Reader system — intentionally ungated at the tuple level:
/// - `erosion_restore` — restores width on `BumpPerformed` messages
///   per bump grade. Holds `MessageReader<BumpPerformed>`. Enforces the
///   gate in-body via `reader.clear()` + return when inactive, draining
///   the buffer every tick. A `.run_if(...)` gate suppresses execution
///   but does NOT advance the reader cursor, so messages buffered
///   during gated-off frames would get retroactively consumed the tick
///   the gate opens.
///
/// Ordering: `erosion_shrink` → `erosion_restore` → `erosion_apply_width`.
/// No `DeathPipelineSystems` ordering — Erosion operates on the shared
/// effect system, not the heal pipeline.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (erosion_shrink, erosion_apply_width)
            .run_if(hazard_active(HazardKind::Erosion))
            .run_if(in_state(NodeState::Playing)),
    )
    .add_systems(
        FixedUpdate,
        erosion_restore
            .after(erosion_shrink)
            .before(erosion_apply_width),
    );
}

/// Shrinks `ErosionState.width_fraction` by `shrink_rate * stacks * dt`
/// each tick, clamped to `min_width_frac`. No-op at stack 0 or with no
/// config resource (defensive; the `run_if` should prevent that).
pub(crate) fn erosion_shrink(
    time: Res<Time<Fixed>>,
    active: Res<ActiveHazards>,
    config: Option<Res<ErosionConfig>>,
    state: Option<ResMut<ErosionState>>,
) {
    let (Some(config), Some(mut state)) = (config, state) else {
        return;
    };
    let stacks = active.stacks(HazardKind::Erosion);
    if stacks == 0 {
        return;
    }
    let delta = config.shrink_rate * stacks as f32 * time.delta_secs();
    if delta <= 0.0 {
        return;
    }
    state.width_fraction = (state.width_fraction - delta).max(config.min_width_frac);
}

/// Restores width when a bump lands. Perfect bumps restore
/// `restore_perfect × lost`; Early/Late bumps restore `restore_nonwhiff ×
/// lost`. Width is clamped to 1.0. Whiffs (absence of `BumpPerformed`)
/// contribute nothing.
///
/// Gated in-body: this system runs every `FixedUpdate` tick. When
/// Erosion is not active or `NodeState` is not `Playing`, it drains the
/// `MessageReader` via `reader.clear()` and returns so buffered
/// `BumpPerformed` messages cannot leak retroactively when the hazard
/// activates on a later frame.
pub(crate) fn erosion_restore(
    mut reader: MessageReader<BumpPerformed>,
    active_hazards: Option<Res<ActiveHazards>>,
    node_state: Option<Res<State<NodeState>>>,
    config: Option<Res<ErosionConfig>>,
    state: Option<ResMut<ErosionState>>,
) {
    if active_hazards
        .as_ref()
        .is_none_or(|ah| !ah.is_active(HazardKind::Erosion))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    let (Some(config), Some(mut state)) = (config, state) else {
        reader.clear();
        return;
    };
    for bump in reader.read() {
        let lost = (1.0 - state.width_fraction).max(0.0);
        if lost <= 0.0 {
            continue;
        }
        let fraction = match bump.grade {
            BumpGrade::Perfect => config.restore_perfect,
            BumpGrade::Early | BumpGrade::Late => config.restore_nonwhiff,
        };
        if fraction <= 0.0 {
            continue;
        }
        // `restore_perfect > 1.0` is legal config — cap to the actual lost
        // amount so a single bump can't overshoot beyond pristine.
        let restore = (lost * fraction).min(lost);
        state.width_fraction = (state.width_fraction + restore).min(1.0);
    }
}

/// Reconciles each Breaker's [`EffectStack<SizeBoostConfig>`] so that
/// exactly one entry with source [`EROSION_SOURCE`] exists, carrying the
/// current `width_fraction` multiplier. Follows the Haste pattern: new
/// stacks are inserted this tick. Idempotent across ticks.
pub(crate) fn erosion_apply_width(
    state: Option<Res<ErosionState>>,
    mut breakers: Query<(Entity, Option<&mut EffectStack<SizeBoostConfig>>), With<Breaker>>,
    mut commands: Commands,
) {
    let Some(state) = state else { return };
    let entry = SizeBoostConfig {
        multiplier: OrderedFloat(state.width_fraction),
    };

    for (entity, stack) in &mut breakers {
        if let Some(mut stack) = stack {
            stack.retain_by_source(EROSION_SOURCE);
            stack.push(EROSION_SOURCE.to_owned(), entry.clone());
        } else {
            let mut fresh = EffectStack::<SizeBoostConfig>::default();
            fresh.push(EROSION_SOURCE.to_owned(), entry.clone());
            commands.entity(entity).insert(fresh);
        }
    }
}
