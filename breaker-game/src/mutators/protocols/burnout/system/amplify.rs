//! Burnout — System 3: amplify per-bolt cell damage on `BurnoutDamageBoost`
//! consume.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{config::BurnoutConfig, on_bump::BurnoutDamageBoost};
use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    mutators::protocols::{definition::ProtocolKind, resources::ActiveProtocols},
    prelude::*,
};

// ── System 3 — burnout_amplify_damage ───────────────────────────────────────

/// Consumes `BoltImpactCell` messages. On a bolt carrying
/// `BurnoutDamageBoost`, emits an amplified `DamageDealt<Cell>` with
/// `amount = base_damage * boost.multiplier` and
/// `source = Some(SourceId::protocol(Burnout).build())` — gated on
/// `amount > 0.0`. Removes the
/// `BurnoutDamageBoost` from the bolt unconditionally (single-shot, even when
/// emission is gated off by the `amount > 0.0` guard).
///
/// Pierce guard: `bolt_cell_collision` can emit multiple `BoltImpactCell`
/// messages for the same bolt in a single frame (pierce-through). The
/// `BurnoutDamageBoost` removal is deferred via `commands`, so subsequent
/// iterations in the same invocation would otherwise still see the boost.
/// `amplified_this_frame` tracks already-amplified bolts within THIS
/// invocation so only ONE amplified emit is produced per boost.
///
/// Harness-safe: if `BurnoutConfig` is absent, clears the reader and returns
/// so buffered messages do not leak into a later frame that does have the
/// resource. Boost is NOT consumed in this path.
///
/// Gated in-body: this system now runs every `FixedUpdate` tick. When
/// Burnout is not active or `NodeState` is not `Playing`, it drains the
/// `MessageReader` via `reader.clear()` and returns so buffered
/// `BoltImpactCell` messages cannot be retroactively amplified when the
/// protocol activates on a later frame.
pub(crate) fn burnout_amplify_damage(
    mut reader: MessageReader<BoltImpactCell>,
    config: Option<Res<BurnoutConfig>>,
    active_protocols: Option<Res<ActiveProtocols>>,
    node_state: Option<Res<State<NodeState>>>,
    bolts: Query<(&BurnoutDamageBoost, Option<&BoltBaseDamage>)>,
    mut commands: Commands,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
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
    if config.is_none() {
        reader.clear();
        return;
    }
    // `Vec` (not `HashSet`) because cardinality is almost always 0 or 1: a
    // single-bolt game with one `BoltImpactCell` message per frame. Linear
    // `contains` beats hash overhead at this scale.
    let mut amplified_this_frame: Vec<Entity> = Vec::with_capacity(1);
    for msg in reader.read() {
        if amplified_this_frame.contains(&msg.bolt) {
            continue;
        }
        let Ok((boost, base_opt)) = bolts.get(msg.bolt) else {
            continue;
        };
        let base_damage = base_opt.map_or(DEFAULT_BOLT_BASE_DAMAGE, |b| b.0);
        let amount = base_damage * boost.multiplier;
        if amount > 0.0 {
            damage_writer.write(DamageDealt::<Cell> {
                dealer: Some(msg.bolt),
                attributed_to: None,
                target: msg.cell,
                amount,
                source: Some(SourceId::protocol(ProtocolKind::Burnout).build()),
                _marker: PhantomData,
            });
        }
        // Unconditionally consume the boost — single-shot regardless of the
        // `amount > 0.0` emission gate.
        if let Ok(mut entity) = commands.get_entity(msg.bolt) {
            entity.remove::<BurnoutDamageBoost>();
        }
        amplified_this_frame.push(msg.bolt);
    }
}
