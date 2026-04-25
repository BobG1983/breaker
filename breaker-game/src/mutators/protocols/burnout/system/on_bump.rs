//! Burnout — System 2: consume `BumpPerformed`, install `BurnoutDamageBoost`,
//! reset heat, dispatch shockwave.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{config::BurnoutConfig, update_heat::BurnoutHeat};
use crate::{
    effect_v3::{
        commands::EffectCommandsExt, effects::shockwave::ShockwaveConfig, types::EffectType,
    },
    mutators::protocols::{definition::ProtocolKind, resources::ActiveProtocols},
    prelude::*,
};

// ── BurnoutDamageBoost ──────────────────────────────────────────────────────

/// Per-bolt single-shot amplified-damage marker inserted by `burnout_on_bump`
/// on a mega-bump consume and consumed by `burnout_amplify_damage` on the
/// bolt's next cell impact.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct BurnoutDamageBoost {
    /// Multiplier carried by this boost (copied from
    /// `BurnoutConfig::full_heat_damage_multiplier` at insert time).
    pub multiplier: f32,
}

// ── System 2 — burnout_on_bump ──────────────────────────────────────────────

/// Consumes `BumpPerformed` messages. On a breaker with
/// `BurnoutHeat::mega_bump_charged == true`, installs `BurnoutDamageBoost` on
/// the bolt, resets the breaker's `BurnoutHeat` (heat 0.0, charge cleared),
/// and dispatches a shockwave at the breaker's position via
/// `commands.fire_effect`.
///
/// Skips (`continue`) when:
/// - `msg.bolt` is `None` (spectator bump).
/// - Breaker lookup fails (despawned or missing `BurnoutHeat`).
/// - `mega_bump_charged` is `false`.
///
/// Harness-safe: if `BurnoutConfig` is absent, clears the reader and returns
/// so buffered messages do not leak into a later frame that does have the
/// resource.
///
/// Gated in-body: this system now runs every `FixedUpdate` tick. When
/// Burnout is not active or `NodeState` is not `Playing`, it drains the
/// `MessageReader` via `reader.clear()` and returns so buffered
/// `BumpPerformed` messages cannot leak retroactively when the protocol
/// activates on a later frame.
pub(crate) fn burnout_on_bump(
    mut reader: MessageReader<BumpPerformed>,
    config: Option<Res<BurnoutConfig>>,
    active_protocols: Option<Res<ActiveProtocols>>,
    node_state: Option<Res<State<NodeState>>>,
    mut breakers: Query<&mut BurnoutHeat, With<Breaker>>,
    mut commands: Commands,
) {
    if active_protocols
        .as_ref()
        .is_none_or(|ap| !ap.contains(ProtocolKind::Burnout))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    let Some(config) = config else {
        reader.clear();
        return;
    };
    for msg in reader.read() {
        let Some(bolt) = msg.bolt else { continue };
        let Ok(mut heat) = breakers.get_mut(msg.breaker) else {
            continue;
        };
        if !heat.mega_bump_charged {
            continue;
        }
        // Install damage boost on the bolt (despawned bolt tolerated).
        if let Ok(mut entity) = commands.get_entity(bolt) {
            entity.insert(BurnoutDamageBoost {
                multiplier: config.full_heat_damage_multiplier,
            });
        }
        // Consume charge — reset heat state to default (heat + timer + charge).
        heat.heat = 0.0;
        heat.still_timer = 0.0;
        heat.mega_bump_charged = false;
        // Dispatch shockwave at breaker's position via fire_effect — the
        // shockwave's `Fireable::fire` impl snapshots Position2D itself.
        commands.fire_effect(
            msg.breaker,
            EffectType::Shockwave(ShockwaveConfig {
                base_range:      OrderedFloat(64.0),
                range_per_level: OrderedFloat(16.0),
                stacks:          1,
                speed:           OrderedFloat(200.0),
            }),
            SourceId::protocol(ProtocolKind::Burnout)
                .action("shockwave")
                .build()
                .0
                .into_owned(),
        );
    }
}
