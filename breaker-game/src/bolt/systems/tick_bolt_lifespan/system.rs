//! System to tick bolt lifespan and dispatch on expiry via `LifetimeEndBehavior`.

use bevy::prelude::*;

use crate::{
    bolt::components::{LifetimeEndBehavior, PhantomBolt},
    prelude::*,
    shared::Lifespan,
};

type BoltLifespanQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Lifespan,
        Option<&'static LifetimeEndBehavior>,
    ),
    (With<Bolt>, Without<Birthing>),
>;

/// Ticks [`Lifespan`] on bolt entities and dispatches on expiry:
///
/// - `LifetimeEndBehavior::Despawn` (or absent) → emit [`DespawnEntity`]
/// - `LifetimeEndBehavior::RevertToNormalBolt` → call [`PhantomBolt::become_normal`]
pub(crate) fn tick_bolt_lifespan(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut query: BoltLifespanQuery,
    mut despawn_writer: MessageWriter<DespawnEntity>,
) {
    let dt = time.delta_secs();
    for (entity, mut lifespan, behavior) in &mut query {
        lifespan.remaining -= dt;
        if lifespan.remaining <= 0.0 {
            match behavior {
                Some(LifetimeEndBehavior::RevertToNormalBolt) => {
                    PhantomBolt::become_normal(&mut commands, entity);
                }
                Some(LifetimeEndBehavior::Despawn) | None => {
                    despawn_writer.write(DespawnEntity { entity });
                }
            }
        }
    }
}
