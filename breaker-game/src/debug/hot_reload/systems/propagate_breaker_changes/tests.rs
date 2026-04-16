//! Tests for breaker hot-reload propagation.
//!
//! Effect propagation tests (Behaviors 15-16) are deferred until all domains
//! use `effect_v3` `BoundEffects`. Only non-effect tests are active.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::MaxSpeed;

use super::system::*;
use crate::{
    breaker::{
        SelectedBreaker,
        components::{BreakerBaseY, BreakerReflectionSpread, DashTilt},
        definition::BreakerDefinition,
        registry::BreakerRegistry,
    },
    prelude::*,
};

fn test_app() -> App {
    TestAppBuilder::new()
        .with_resource::<BreakerRegistry>()
        .with_resource::<SelectedBreaker>()
        .with_system(Update, propagate_breaker_changes)
        .build()
}

fn make_test_def(name: &str, life_pool: Option<u32>) -> BreakerDefinition {
    ron::de::from_str(&format!(
        r#"(name: "{name}", life_pool: {lp}, bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))), salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))), effects: [])"#,
        lp = life_pool.map_or_else(|| "None".to_string(), |n| format!("Some({n})")),
    ))
    .expect("test RON should parse")
}

#[test]
fn registry_rebuilt_on_modified() {
    let mut app = test_app();
    let def = make_test_def("Test", Some(3));
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }
    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));
    app.update();
    app.update();
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let updated = make_test_def("Test", Some(5));
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }
    app.update();
    let registry = app.world().resource::<BreakerRegistry>();
    let rebuilt = registry.get("Test").unwrap();
    assert_eq!(rebuilt.life_pool, Some(5));
}

#[test]
fn hot_reload_stamps_max_speed_from_definition() {
    let mut app = test_app();
    let def = make_test_def("Test", Some(3));
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }
    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));
    let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.update();
    app.update();
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let mut updated = make_test_def("Test", Some(3));
        updated.max_speed = 800.0;
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }
    app.update();
    let max_speed = app.world().get::<MaxSpeed>(entity).unwrap();
    assert!(
        (max_speed.0 - 800.0).abs() < f32::EPSILON,
        "MaxSpeed should be updated to 800.0 from definition, got {}",
        max_speed.0
    );
}

#[test]
fn hot_reload_updates_reflection_spread_in_radians() {
    let mut app = test_app();
    let def = make_test_def("Test", None);
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }
    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));
    let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut()
        .entity_mut(entity)
        .insert((BreakerReflectionSpread(999.0), DashTilt(999.0)));
    app.update();
    app.update();
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let mut updated = make_test_def("Test", None);
        updated.reflection_spread = 60.0;
        updated.dash_tilt_angle = 20.0;
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }
    app.update();
    let spread = app.world().get::<BreakerReflectionSpread>(entity).unwrap();
    assert!(
        (spread.0 - 60.0_f32.to_radians()).abs() < 1e-5,
        "BreakerReflectionSpread should be 60 degrees in radians, got {}",
        spread.0
    );
    let tilt = app.world().get::<DashTilt>(entity).unwrap();
    assert!(
        (tilt.0 - 20.0_f32.to_radians()).abs() < 1e-5,
        "DashTilt should be 20 degrees in radians, got {}",
        tilt.0
    );
}

#[test]
fn hot_reload_updates_breaker_base_y_from_definition() {
    let mut app = test_app();
    let def = make_test_def("Test", None);
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }
    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));
    let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.update();
    app.update();
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let mut updated = make_test_def("Test", None);
        updated.y_position = -300.0;
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }
    app.update();
    let base_y = app.world().get::<BreakerBaseY>(entity).unwrap();
    assert!(
        (base_y.0 - (-300.0)).abs() < f32::EPSILON,
        "BreakerBaseY should be updated to -300.0, got {}",
        base_y.0
    );
}

// TODO(effect_v3 migration): Tests for Behaviors 15-16 (LivesCount/Hp propagation,
// BoundEffects rebuild from definition effects) are deferred. The hot-reload system
// currently has a TODO for effect tree propagation since entities still have old-domain
// BoundEffects components. These tests will be restored when all domains use effect_v3.
