//! Overcharge hazard — the Bolt gains speed per cell destroyed within a
//! single bump cycle (between consecutive [`BumpPerformed`] events), and
//! resets on the next bump. Per-kill multiplier scales linearly with
//! stack count. The effective multiplier is the per-kill multiplier
//! raised to the cycle's kill count, applied through the Bolt's
//! [`EffectStack<SpeedBoostConfig>`] under a dedicated source tag.

use bevy::{platform::collections::HashMap, prelude::*};
use ordered_float::OrderedFloat;

use crate::{
    bolt::components::Bolt,
    breaker::messages::BumpPerformed,
    cells::components::Cell,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::death_pipeline::Destroyed,
};

/// Source tag for Overcharge's entry on the Bolt's
/// [`EffectStack<SpeedBoostConfig>`]. Dedicated so Overcharge can swap
/// its entry on kill/reset without disturbing other speed contributions.
const OVERCHARGE_SOURCE: &str = "hazard:overcharge";

/// Per-run tuning extracted from [`HazardTuning::Overcharge`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct OverchargeConfig {
    /// Per-kill speed fraction at stack 1 (e.g. `0.05` → +5% per kill).
    pub(crate) base_frac:      f32,
    /// Additional per-kill speed fraction per stack beyond the first.
    pub(crate) per_level_frac: f32,
}

impl OverchargeConfig {
    /// Per-kill speed multiplier for the given stack count. Returns 1.0 when
    /// `stacks == 0` (hazard inactive), **even when `base_frac > 0`**. For
    /// `stacks >= 1`, returns `1.0 + base_frac + per_level_frac * (stacks - 1)`.
    /// Applied compounding per kill in `overcharge_apply_speed`.
    #[must_use]
    pub(crate) fn per_kill_multiplier(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 1.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        1.0 + self.per_level_frac.mul_add(extra, self.base_frac)
    }
}

/// Tracks the number of cells a given bolt has destroyed since its last
/// bump. Reset to 0 on [`BumpPerformed`]. Missing entries are treated as 0.
#[derive(Component, Debug, Default, Clone, Copy)]
pub(crate) struct OverchargeKillCount(pub(crate) u32);

/// Inserts `OverchargeConfig` from `HazardTuning::Overcharge { base_frac,
/// per_level_frac }`. Called each time the player picks Overcharge from
/// `hazards::activate`; last write wins (overwrites any prior
/// `OverchargeConfig`; per-bolt kill tracking is a separate component,
/// untouched here). Warns and no-ops on a non-Overcharge tuning variant,
/// leaving any existing `OverchargeConfig` intact. Does not mutate
/// `ActiveHazards` — the caller owns stack accounting.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Overcharge {
        base_frac,
        per_level_frac,
    } = *tuning
    else {
        warn!("overcharge::activate called with non-Overcharge tuning");
        return;
    };
    commands.insert_resource(OverchargeConfig {
        base_frac,
        per_level_frac,
    });
}

/// Registers the Overcharge `FixedUpdate` systems.
///
/// Reader systems — intentionally ungated at the tuple level. They hold
/// `MessageReader<...>` and enforce the `ActiveHazards` /
/// `NodeState::Playing` gate in-body via an immediate `reader.clear()` +
/// return when inactive. A `.run_if(...)` gate suppresses execution but
/// does NOT advance the reader cursor, so messages buffered during
/// gated-off frames would get retroactively consumed the tick the gate
/// opens:
/// - `overcharge_count_kills` — reads `Destroyed<Cell>` and increments
///   the killer bolt's `OverchargeKillCount`.
/// - `overcharge_reset_on_bump` — reads `BumpPerformed` and zeroes the
///   count.
///
/// Non-reader system — gated on `hazard_active(HazardKind::Overcharge)`
/// AND `in_state(NodeState::Playing)`:
/// - `overcharge_apply_speed` — reconciles the Bolt's
///   `EffectStack<SpeedBoostConfig>` with a single
///   source-`"hazard:overcharge"` entry.
///
/// Ordering: `overcharge_count_kills` → `overcharge_reset_on_bump` →
/// `overcharge_apply_speed`. No `DeathPipelineSystems` ordering —
/// Overcharge operates on the shared effect system, not the heal /
/// damage pipeline.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        overcharge_apply_speed
            .after(overcharge_reset_on_bump)
            .run_if(hazard_active(HazardKind::Overcharge))
            .run_if(in_state(NodeState::Playing)),
    )
    .add_systems(
        FixedUpdate,
        (overcharge_count_kills, overcharge_reset_on_bump).chain(),
    );
}

/// For each `Destroyed<Cell>` whose killer is a Bolt, increment that
/// bolt's [`OverchargeKillCount`]. Inserts the component on first kill.
/// Kills whose killer is `None` (environmental) or is not a Bolt are
/// skipped silently.
///
/// Gated in-body: this system runs every `FixedUpdate` tick. When
/// Overcharge is not active or `NodeState` is not `Playing`, it drains
/// the `MessageReader` via `reader.clear()` and returns so buffered
/// `Destroyed<Cell>` messages cannot leak retroactively when the hazard
/// activates on a later frame.
pub(crate) fn overcharge_count_kills(
    mut reader: MessageReader<Destroyed<Cell>>,
    active_hazards: Option<Res<ActiveHazards>>,
    node_state: Option<Res<State<NodeState>>>,
    bolts: Query<Entity, With<Bolt>>,
    mut counts: Query<&mut OverchargeKillCount>,
    mut commands: Commands,
) {
    if active_hazards
        .as_ref()
        .is_none_or(|ah| !ah.is_active(HazardKind::Overcharge))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    // Tally kills per bolt first so multiple kills in one tick don't
    // race against pending Commands inserts.
    let mut batch: HashMap<Entity, u32> = HashMap::default();
    for destroyed in reader.read() {
        let Some(killer) = destroyed.killer else {
            continue;
        };
        if !bolts.contains(killer) {
            continue;
        }
        *batch.entry(killer).or_insert(0) += 1;
    }

    for (bolt, kills) in batch {
        if let Ok(mut count) = counts.get_mut(bolt) {
            count.0 = count.0.saturating_add(kills);
        } else {
            commands.entity(bolt).insert(OverchargeKillCount(kills));
        }
    }
}

/// Resets a bolt's kill count on every `BumpPerformed` for that bolt.
/// Missing [`OverchargeKillCount`] is a no-op — zero is zero. Messages
/// with `bolt: None` (no identified bumping bolt) are skipped.
///
/// Gated in-body: this system runs every `FixedUpdate` tick. When
/// Overcharge is not active or `NodeState` is not `Playing`, it drains
/// the `MessageReader` via `reader.clear()` and returns so buffered
/// `BumpPerformed` messages cannot leak retroactively when the hazard
/// activates on a later frame.
pub(crate) fn overcharge_reset_on_bump(
    mut reader: MessageReader<BumpPerformed>,
    active_hazards: Option<Res<ActiveHazards>>,
    node_state: Option<Res<State<NodeState>>>,
    mut counts: Query<&mut OverchargeKillCount>,
) {
    if active_hazards
        .as_ref()
        .is_none_or(|ah| !ah.is_active(HazardKind::Overcharge))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    for bump in reader.read() {
        let Some(bolt) = bump.bolt else { continue };
        if let Ok(mut count) = counts.get_mut(bolt) {
            count.0 = 0;
        }
    }
}

/// Aggregated query over Bolts carrying an [`OverchargeKillCount`] and
/// (optionally) a [`SpeedBoostConfig`] effect stack.
type BoltSpeedQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static OverchargeKillCount>,
        Option<&'static mut EffectStack<SpeedBoostConfig>>,
    ),
    With<Bolt>,
>;

/// Reconciles each Bolt's [`EffectStack<SpeedBoostConfig>`] so that
/// exactly one entry with source [`OVERCHARGE_SOURCE`] carries
/// `per_kill_multiplier ^ kill_count`. Bolts with zero kills get no
/// entry (the reconcile retains no stale entry either). Idempotent.
/// Early-returns when `OverchargeConfig` is absent; does not require
/// `ActiveHazards` to have stacks > 0 (the multiplier handles the
/// zero-stack case by collapsing to 1.0).
pub(crate) fn overcharge_apply_speed(
    active: Res<ActiveHazards>,
    config: Option<Res<OverchargeConfig>>,
    mut bolts: BoltSpeedQuery,
    mut commands: Commands,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Overcharge);
    let per_kill = config.per_kill_multiplier(stacks);

    for (entity, count, stack) in &mut bolts {
        let kills = count.map_or(0, |c| c.0);
        let multiplier = per_kill.powi(kills.cast_signed());

        if let Some(mut stack) = stack {
            stack.retain_by_source(OVERCHARGE_SOURCE);
            if kills > 0 {
                stack.push(
                    OVERCHARGE_SOURCE.to_owned(),
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(multiplier),
                    },
                );
            }
        } else if kills > 0 {
            let mut fresh = EffectStack::<SpeedBoostConfig>::default();
            fresh.push(
                OVERCHARGE_SOURCE.to_owned(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                },
            );
            commands.entity(entity).insert(fresh);
        }
    }
}
