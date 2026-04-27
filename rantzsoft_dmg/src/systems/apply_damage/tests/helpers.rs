use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::system::apply_damage;
use crate::{messages::DamageDealt, traits::Dmgable};

#[derive(Component)]
pub(super) struct TestT;
impl Dmgable for TestT {}

#[derive(Component)]
pub(super) struct OtherT;
impl Dmgable for OtherT {}

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
    app.add_message::<DamageDealt<TestT>>();
    app.add_systems(FixedUpdate, apply_damage::<TestT>);
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn enqueue(app: &mut App, msg: DamageDealt<TestT>) {
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(msg);
}

pub(super) fn mk_msg(
    dealer: Option<Entity>,
    attributed_to: Option<Entity>,
    target: Entity,
    amount: f32,
) -> DamageDealt<TestT> {
    DamageDealt::<TestT> {
        dealer,
        attributed_to,
        target,
        amount,
        source: None,
        _marker: PhantomData,
    }
}
