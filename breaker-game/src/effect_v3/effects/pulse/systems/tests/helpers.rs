use std::{collections::HashSet, time::Duration};

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::system::*;
use crate::{
    cells::components::Cell,
    effect_v3::effects::pulse::components::*,
    shared::{
        death_pipeline::{DamageDealt, Dead},
        test_utils::TestAppBuilder,
    },
};

pub(super) fn damage_test_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_system(FixedUpdate, apply_pulse_damage)
        .build()
}

/// `tick_pulse_ring` uses `Res<Time>`, which only resolves to `Time<Fixed>`
/// when the system is scheduled inside `FixedUpdate`. Registering it in
/// `Update` would silently use a zero-delta virtual clock.
pub(super) fn tick_test_app() -> App {
    TestAppBuilder::new()
        .with_system(FixedUpdate, tick_pulse_ring)
        .build()
}

pub(super) fn despawn_test_app() -> App {
    TestAppBuilder::new()
        .with_system(FixedUpdate, despawn_finished_pulse_ring)
        .build()
}

/// `tick_pulse` test app — exercises the emitter tick (Section F).
pub(super) fn emitter_test_app() -> App {
    TestAppBuilder::new()
        .with_system(FixedUpdate, tick_pulse)
        .build()
}

pub(super) fn tick_with_dt(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}

pub(super) fn spawn_cell(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut().spawn((Cell, Position2D(pos))).id()
}

pub(super) fn spawn_dead_cell(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut().spawn((Cell, Position2D(pos), Dead)).id()
}

pub(super) fn spawn_pulse_ring_no_chip(
    app: &mut App,
    pos: Vec2,
    radius: f32,
    base_dmg: f32,
    dmg_mult: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Position2D(pos),
            PulseRingRadius(radius),
            PulseRingBaseDamage(base_dmg),
            PulseRingDamageMultiplier(dmg_mult),
            PulseRingDamaged(HashSet::new()),
        ))
        .id()
}
