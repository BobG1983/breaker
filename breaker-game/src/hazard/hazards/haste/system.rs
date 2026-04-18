//! Haste hazard — multiplies bolt speed via the `SpeedBoostConfig` stack.
//! Stacking adds linear percentage on top of the base. Reconciles every
//! tick so the bolt stack always reflects the current hazard stack count.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    bolt::components::Bolt,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

/// Source tag used on the bolt's `EffectStack<SpeedBoostConfig>`. Shared
/// between the apply system and teardown so Haste can reconcile its own
/// entries without disturbing chip- or protocol-owned boosts.
const HASTE_SOURCE: &str = "hazard:haste";

/// Per-run tuning extracted from [`HazardTuning::Haste`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct HasteConfig {
    /// Added speed percentage at stack 1 (e.g. `20.0` → +20%).
    pub(crate) base_percent:      f32,
    /// Additional speed percentage per stack beyond the first.
    pub(crate) per_level_percent: f32,
}

impl HasteConfig {
    /// Multiplicative speed factor for the given stack count. Returns 1.0
    /// when `stacks == 0` (hazard inactive).
    #[must_use]
    pub(crate) fn multiplier(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 1.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        let percent = self.per_level_percent.mul_add(extra, self.base_percent);
        1.0 + percent / 100.0
    }
}

/// Inserts `HasteConfig` from `HazardTuning::Haste { base_percent,
/// per_level_percent }`. Called each time the player picks Haste; last
/// write wins (overwrites any prior `HasteConfig`; stack count is owned
/// by `ActiveHazards`, not the config). Warns and no-ops on a non-Haste
/// tuning variant, leaving any existing `HasteConfig` intact.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Haste {
        base_percent,
        per_level_percent,
    } = *tuning
    else {
        warn!("haste::activate called with non-Haste tuning");
        return;
    };
    commands.insert_resource(HasteConfig {
        base_percent,
        per_level_percent,
    });
}

/// Registers `haste_apply_speed` in `FixedUpdate`, gated on
/// `hazard_active(HazardKind::Haste)` AND `in_state(NodeState::Playing)`.
/// No `DeathPipelineSystems` ordering constraints — Haste does not
/// participate in the heal / damage pipeline; it operates on the shared
/// effect system via `EffectStack<SpeedBoostConfig>`. The system
/// reconciles each Bolt's `EffectStack<SpeedBoostConfig>` with a single
/// source-`hazard:haste` entry reflecting the current stack count's
/// multiplier.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        haste_apply_speed
            .run_if(hazard_active(HazardKind::Haste))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Reconciles each Bolt's `EffectStack<SpeedBoostConfig>` so that exactly
/// one entry with source [`HASTE_SOURCE`] exists, carrying the current
/// multiplier. Bolts without a stack yet are given one this tick; the
/// entry is pushed on the same tick. Idempotent across ticks.
pub(crate) fn haste_apply_speed(
    active: Res<ActiveHazards>,
    config: Option<Res<HasteConfig>>,
    mut bolts: Query<(Entity, Option<&mut EffectStack<SpeedBoostConfig>>), With<Bolt>>,
    mut commands: Commands,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Haste);
    let multiplier = config.multiplier(stacks);
    let entry = SpeedBoostConfig {
        multiplier: OrderedFloat(multiplier),
    };

    for (entity, stack) in &mut bolts {
        if let Some(mut stack) = stack {
            stack.retain_by_source(HASTE_SOURCE);
            stack.push(HASTE_SOURCE.to_owned(), entry.clone());
        } else {
            let mut fresh = EffectStack::<SpeedBoostConfig>::default();
            fresh.push(HASTE_SOURCE.to_owned(), entry.clone());
            commands.entity(entity).insert(fresh);
        }
    }
}
