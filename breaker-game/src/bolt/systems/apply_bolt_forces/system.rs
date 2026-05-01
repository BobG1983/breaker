//! Consumer system — drains `ApplyBoltForce` messages and applies velocity deltas.

use bevy::prelude::*;

use crate::{bolt::messages::ApplyBoltForce, prelude::*};

/// Drains `ApplyBoltForce` messages and applies velocity deltas to bolts.
///
/// Forces are summed per bolt before the `dt` multiply, so multiple producers
/// per tick compose correctly without repeated scalar application.
pub(crate) fn apply_bolt_forces(
    mut reader: MessageReader<ApplyBoltForce>,
    time: Res<Time<Fixed>>,
    mut bolts: Query<&mut Velocity2D, With<Bolt>>,
    mut accum: Local<Vec<(Entity, Vec2)>>,
) {
    let mut iter = reader.read();
    // Fast-path: skips HashMap allocation when the message queue is empty.
    let Some(first) = iter.next() else { return };
    accum.clear();

    accum.push((first.bolt, first.force));
    for msg in iter {
        match accum.iter_mut().find(|(e, _)| *e == msg.bolt) {
            Some((_, f)) => *f += msg.force,
            None => accum.push((msg.bolt, msg.force)),
        }
    }

    let dt = time.delta_secs();
    for (entity, force_sum) in &*accum {
        if let Ok(mut velocity) = bolts.get_mut(*entity) {
            velocity.0 += *force_sum * dt;
        }
    }
}
