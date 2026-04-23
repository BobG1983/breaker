//! Shared helpers for `cross_t_isolation` sub-files.
//!
//! Registers two distinct `Dmgable` marker types (`T1`, `T2`) used to pin
//! per-`T` pipeline isolation across all behaviors 11–15.

use bevy::prelude::*;
use rantzsoft_dmg::{Dmgable, RantzDmgAppExt, RantzDmgPlugin};

#[derive(Component)]
pub(super) struct T1;
impl Dmgable for T1 {}

#[derive(Component)]
pub(super) struct T2;
impl Dmgable for T2 {}

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

pub(super) fn app_with_both_types() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<T1>().register_dmgable::<T2>();
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
