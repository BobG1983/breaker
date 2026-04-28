use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::system::apply_damage_boosts;
use crate::{SourceId, messages::DamageDealt, traits::Dmgable};

#[derive(Component)]
pub(super) struct TestT;
impl Dmgable for TestT {}

/// Compares two `f32` values for "equality" without tripping
/// `clippy::float_cmp`. Replicated per file per the test spec
/// copy-paste policy.
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

/// Build a bare App with `MinimalPlugins`, `Messages<DamageDealt<TestT>>`
/// registered, and `apply_damage_boosts::<TestT>` scheduled in
/// `FixedUpdate`. No `RantzDmgPlugin` — we exercise this system in
/// isolation.
pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<DamageDealt<TestT>>();
    app.add_systems(FixedUpdate, apply_damage_boosts::<TestT>);
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

pub(super) fn drain_messages(app: &mut App) -> Vec<DamageDealt<TestT>> {
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .drain()
        .collect()
}

pub(super) fn mk_msg(dealer: Option<Entity>, amount: f32) -> DamageDealt<TestT> {
    DamageDealt::<TestT> {
        dealer,
        attributed_to: None,
        target: Entity::PLACEHOLDER,
        amount,
        source: None,
        _marker: PhantomData,
    }
}

/// Constructs a `DamageDealt` with an explicit `attributed_to` override. Dealer-only
/// `mk_msg` above always sets `attributed_to: None`, so this variant is needed by
/// attribution tests that need both `dealer` and `attributed_to` set.
pub(super) fn mk_msg_full(
    dealer: Option<Entity>,
    attributed_to: Option<Entity>,
    amount: f32,
) -> DamageDealt<TestT> {
    DamageDealt::<TestT> {
        dealer,
        attributed_to,
        target: Entity::PLACEHOLDER,
        amount,
        source: None,
        _marker: PhantomData,
    }
}

/// Constructs a `DamageDealt` with an explicit emission `source`. Used by the
/// system-level source-filter coverage to verify that `apply_damage_boosts`
/// passes `msg.source.as_ref()` into both aggregate calls so filtered entries
/// scope correctly.
pub(super) fn mk_msg_with_source(
    dealer: Option<Entity>,
    source: Option<SourceId>,
    amount: f32,
) -> DamageDealt<TestT> {
    DamageDealt::<TestT> {
        dealer,
        attributed_to: None,
        target: Entity::PLACEHOLDER,
        amount,
        source,
        _marker: PhantomData,
    }
}
