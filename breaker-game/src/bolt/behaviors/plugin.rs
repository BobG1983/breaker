//! `BoltBehaviorsPlugin` — wires the overclock evaluation engine.

use bevy::prelude::*;

use super::{
    active::ActiveOverclocks,
    bridges::{
        bridge_overclock_bolt_lost, bridge_overclock_breaker_impact, bridge_overclock_bump,
        bridge_overclock_cell_destroyed, bridge_overclock_cell_impact,
        bridge_overclock_wall_impact,
    },
    effects::handle_shockwave,
};
use crate::{
    behaviors::BehaviorSystems, breaker::BreakerSystems, physics::PhysicsSystems,
    shared::PlayingState,
};

/// Plugin for the bolt overclock evaluation engine.
///
/// Registers `ActiveOverclocks` resource and bridge systems.
///
/// `OverclockEffectFired` is dispatched via `commands.trigger()` and
/// consumed by observers — no `add_event` registration needed.
pub(crate) struct BoltBehaviorsPlugin;

impl Plugin for BoltBehaviorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveOverclocks>()
            .add_observer(handle_shockwave)
            .add_systems(
                FixedUpdate,
                (
                    bridge_overclock_bump
                        .after(BreakerSystems::GradeBump)
                        .after(BehaviorSystems::Bridge),
                    bridge_overclock_cell_impact
                        .after(PhysicsSystems::BreakerCollision)
                        .after(BehaviorSystems::Bridge),
                    bridge_overclock_breaker_impact
                        .after(PhysicsSystems::BreakerCollision)
                        .after(BehaviorSystems::Bridge),
                    bridge_overclock_wall_impact
                        .after(PhysicsSystems::BreakerCollision)
                        .after(BehaviorSystems::Bridge),
                    bridge_overclock_cell_destroyed.after(BehaviorSystems::Bridge),
                    bridge_overclock_bolt_lost
                        .after(PhysicsSystems::BoltLost)
                        .after(BehaviorSystems::Bridge),
                )
                    .run_if(in_state(PlayingState::Active)),
            );
    }
}
