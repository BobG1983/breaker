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
///
/// **Installer pattern for callers**: when installing `Lifespan` onto an
/// already-spawned entity in the SAME `FixedUpdate` pass that this system
/// runs in (no ordering edge between the installer and this system), the
/// `Lifespan` insert is deferred and invisible to this system on the install
/// tick. The installer should pre-subtract one `Time::<Fixed>::delta_secs()`
/// from the initial `Lifespan::remaining` to compensate for the dropped
/// tick. Callers that spawn a fresh entity carrying `Lifespan` (e.g., the
/// `Bolt` builder's chip-effect path) do NOT need this compensation because
/// the entity itself is invisible until the next flush.
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
