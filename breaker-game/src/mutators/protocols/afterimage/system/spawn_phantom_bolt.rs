use bevy::prelude::*;

use super::config::AfterimageConfig;
use crate::{
    bolt::components::{Bolt, LifetimeEndBehavior, PhantomBolt, PhantomDedupKey},
    breaker::{components::PhantomBreaker, messages::BumpGrade},
    mutators::protocols::{definition::ProtocolKind, systems::ProtocolGate},
    prelude::*,
    shared::{Lifespan, PhantomFlicker},
};

// ── System 4 — afterimage_spawn_phantom_bolt ───────────────────────────────

/// On a Perfect `BumpPerformed` against a `PhantomBreaker`, mutates the REAL
/// bolt entity in place into a phantom: calls `Bolt::become_phantom` (which
/// inserts `PhantomBolt`, `PhantomDedupKey::Bolt(real_bolt)`, and empty
/// `PhantomDamagedCells`) and then inserts `Lifespan`,
/// `LifetimeEndBehavior::RevertToNormalBolt`, and `PhantomFlicker::default()`
/// on the same entity.
///
/// Lifespan is installed at `phantom_bolt_duration - dt` so the install-tick
/// "counts" against lifetime — `tick_bolt_lifespan` runs in the same
/// `FixedUpdate` pass as the install but cannot see the still-deferred
/// `Lifespan`, so it would otherwise grant the phantom a free tick.
///
/// No new entity is spawned. If the real bolt already carries `PhantomBolt`,
/// the message is consumed without further action — duration does NOT reset.
/// The `tick_bolt_lifespan` system (Wave 3A) reverts the bolt at expiry via
/// `PhantomBolt::become_normal`.
pub(crate) fn afterimage_spawn_phantom_bolt(
    mut reader: MessageReader<BumpPerformed>,
    config: Option<Res<AfterimageConfig>>,
    time: Res<Time<Fixed>>,
    gate: ProtocolGate,
    phantom_breakers: Query<(), With<PhantomBreaker>>,
    real_bolts: Query<Has<PhantomBolt>, With<Bolt>>,
    mut commands: Commands,
) {
    if gate.is_closed_for(ProtocolKind::Afterimage) {
        reader.clear();
        return;
    }
    let Some(config) = config else {
        reader.clear();
        return;
    };
    if phantom_breakers.is_empty() {
        reader.clear();
        return;
    }
    let install_remaining = config.phantom_bolt_duration - time.delta_secs();
    for msg in reader.read() {
        if msg.grade != BumpGrade::Perfect {
            continue;
        }
        let Some(real_bolt) = msg.bolt else {
            continue;
        };
        if !phantom_breakers.contains(msg.breaker) {
            continue;
        }
        let Ok(already_phantom) = real_bolts.get(real_bolt) else {
            continue;
        };
        if already_phantom {
            continue;
        }
        Bolt::become_phantom(&mut commands, real_bolt, PhantomDedupKey::Bolt(real_bolt));
        commands.entity(real_bolt).insert((
            Lifespan {
                remaining: install_remaining,
            },
            LifetimeEndBehavior::RevertToNormalBolt,
            PhantomFlicker::default(),
        ));
    }
}
