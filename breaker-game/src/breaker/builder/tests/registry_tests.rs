use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use crate::breaker::{definition::BreakerDefinition, registry::BreakerRegistry};

// ── Behavior 51: BreakerRegistry extensions() returns &["breaker.ron"] ──

#[test]
fn extensions_returns_breaker_ron() {
    assert_eq!(
        BreakerRegistry::extensions(),
        &["breaker.ron"],
        "extensions() should return [\"breaker.ron\"]"
    );
    // Negative: old extension should NOT be present
    assert!(
        !BreakerRegistry::extensions().contains(&"bdef.ron"),
        "extensions() should NOT contain \"bdef.ron\""
    );
}

// ── Behavior 52: BreakerRegistry seed() populates from expanded definitions ──

#[test]
fn seed_populates_from_expanded_definitions() {
    let def_a: BreakerDefinition =
        ron::de::from_str(r#"(name: "TestBreaker", width: 150.0, max_speed: 700.0, effects: [])"#)
            .expect("test RON should parse");
    let def_b: BreakerDefinition =
        ron::de::from_str(r#"(name: "OtherBreaker", width: 120.0, max_speed: 1000.0, effects: [])"#)
            .expect("test RON should parse");

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<BreakerDefinition>();

    let pairs: Vec<_> = {
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        vec![
            {
                let h = assets.add(def_a.clone());
                (h.id(), def_a)
            },
            {
                let h = assets.add(def_b.clone());
                (h.id(), def_b)
            },
        ]
    };

    let mut registry = BreakerRegistry::default();
    registry.seed(&pairs);

    let test_def = registry.get("TestBreaker").unwrap();
    assert!((test_def.width - 150.0).abs() < f32::EPSILON);
    assert!((test_def.max_speed - 700.0).abs() < f32::EPSILON);

    let other_def = registry.get("OtherBreaker").unwrap();
    assert!((other_def.width - 120.0).abs() < f32::EPSILON);
    assert!((other_def.max_speed - 1000.0).abs() < f32::EPSILON);
}

// ── Behavior 53: BreakerRegistry asset_dir() still returns "breakers" ──

#[test]
fn asset_dir_returns_breakers() {
    assert_eq!(
        BreakerRegistry::asset_dir(),
        "breakers",
        "asset_dir() should return \"breakers\""
    );
}
