use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::system::handle_kill;
use crate::{
    messages::{DespawnEntity, Destroyed, KillYourself},
    traits::Dmgable,
};

#[derive(Component)]
pub(super) struct TestT;
impl Dmgable for TestT {}

#[track_caller]
pub(super) fn assert_f32_eq(actual: f32, expected: f32) {
    if expected.is_infinite() {
        assert!(
            actual.is_infinite() && actual.is_sign_positive() == expected.is_sign_positive(),
            "expected {expected}, got {actual}"
        );
    } else {
        assert!(
            (actual - expected).abs() < f32::EPSILON,
            "expected {expected}, got {actual}"
        );
    }
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<KillYourself<TestT>>();
    app.add_message::<Destroyed<TestT>>();
    app.add_message::<DespawnEntity>();
    app.add_systems(FixedUpdate, handle_kill::<TestT>);
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn enqueue_kill(app: &mut App, msg: KillYourself<TestT>) {
    app.world_mut()
        .resource_mut::<Messages<KillYourself<TestT>>>()
        .write(msg);
}

pub(super) fn drain_destroyed(app: &mut App) -> Vec<Destroyed<TestT>> {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect()
}

pub(super) fn drain_despawn(app: &mut App) -> Vec<DespawnEntity> {
    app.world_mut()
        .resource_mut::<Messages<DespawnEntity>>()
        .drain()
        .collect()
}

pub(super) fn mk_kill(victim: Entity, killer: Option<Entity>) -> KillYourself<TestT> {
    KillYourself::<TestT> {
        victim,
        killer,
        _marker: PhantomData,
    }
}
