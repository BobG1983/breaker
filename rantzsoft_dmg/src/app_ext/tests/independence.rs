use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    super::system::*,
    helpers::{OtherT, TestT, app_with_plugin, assert_f32_eq, tick},
};
use crate::{Hp, messages::DamageDealt};

// ── Behavior 149: TestT and OtherT are independent ──

#[test]
fn test_t_and_other_t_pipelines_are_independent() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>().register_dmgable::<OtherT>();

    let target = app.world_mut().spawn((OtherT, Hp::new(10.0))).id();

    // Send message in TestT queue referencing an OtherT target — skipped.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer: None,
            attributed_to: None,
            target,
            amount: 10.0,
            source: None,
            _marker: PhantomData,
        });
    // Send via OtherT queue — applied.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<OtherT>>>()
        .write(DamageDealt::<OtherT> {
            dealer: None,
            attributed_to: None,
            target,
            amount: 10.0,
            source: None,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(target).unwrap().current, 0.0);
}
