use bevy::{prelude::*, sprite_render::AlphaMode2d};
use rantzsoft_spatial2d::components::Scale2D;

use super::helpers::*;
use crate::effect::effects::shockwave::system::*;

// =========================================================================
// Part D: animate_shockwave
// =========================================================================

/// Behavior 12: `animate_shockwave` sets `Scale2D.x = current * 2.0` and
/// `Scale2D.y = current * 2.0`.
#[test]
fn animate_shockwave_scales_by_diameter() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, animate_shockwave);

    let entity = app
        .world_mut()
        .spawn((
            ShockwaveRadius {
                current: 48.0,
                max: 96.0,
            },
            Scale2D::default(),
        ))
        .id();

    tick(&mut app);

    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("entity should have Scale2D");
    // current * 2.0 = 48.0 * 2.0 = 96.0
    assert!(
        (scale.x - 96.0).abs() < f32::EPSILON,
        "Scale2D.x should be 96.0 (48.0 * 2.0), got {}",
        scale.x
    );
    assert!(
        (scale.y - 96.0).abs() < f32::EPSILON,
        "Scale2D.y should be 96.0 (48.0 * 2.0), got {}",
        scale.y
    );
}

/// Behavior 12 edge case: `current = 0.0` should produce `Scale2D` (0.0, 0.0),
/// NOT panic. This is why we write fields directly instead of using
/// `Scale2D::new` (which panics on zero).
#[test]
fn animate_shockwave_zero_radius_does_not_panic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, animate_shockwave);

    let entity = app
        .world_mut()
        .spawn((
            ShockwaveRadius {
                current: 0.0,
                max: 96.0,
            },
            Scale2D::default(),
        ))
        .id();

    tick(&mut app);

    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("entity should have Scale2D");
    assert!(
        scale.x.abs() < f32::EPSILON,
        "Scale2D.x should be 0.0 at zero radius, got {}",
        scale.x
    );
    assert!(
        scale.y.abs() < f32::EPSILON,
        "Scale2D.y should be 0.0 at zero radius, got {}",
        scale.y
    );
}

// -- Regression: animate_shockwave does not produce NaN when max is zero --

/// When both `current` and `max` are `0.0`, the progress calculation
/// `current / max` yields `NaN`. The alpha must remain finite (no NaN).
#[test]
fn animate_shockwave_zero_max_radius_does_not_produce_nan_alpha() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<ColorMaterial>>();
    app.add_systems(FixedUpdate, animate_shockwave);

    // Create a material and store the handle
    let mat_handle = {
        let mut materials = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
        materials.add(ColorMaterial {
            color: Color::linear_rgba(0.0, 4.0, 4.0, 0.9),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })
    };

    let entity = app
        .world_mut()
        .spawn((
            ShockwaveRadius {
                current: 0.0,
                max: 0.0,
            },
            Scale2D { x: 1.0, y: 1.0 },
            MeshMaterial2d(mat_handle.clone()),
        ))
        .id();

    tick(&mut app);

    // Scale2D should be (0.0, 0.0): current * 2.0 = 0.0
    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        scale.x.abs() < f32::EPSILON,
        "Scale2D.x should be 0.0 when current is 0.0, got {}",
        scale.x
    );
    assert!(
        scale.y.abs() < f32::EPSILON,
        "Scale2D.y should be 0.0 when current is 0.0, got {}",
        scale.y
    );

    // Alpha must be finite (not NaN) — 0.0/0.0 produces NaN in the
    // progress calculation, which propagates to alpha.
    let materials = app.world().resource::<Assets<ColorMaterial>>();
    let material = materials
        .get(mat_handle.id())
        .expect("material should still exist");
    let linear = material.color.to_linear();
    assert!(
        linear.alpha.is_finite(),
        "material alpha must be finite (not NaN), got {} — \
         likely caused by 0.0 / 0.0 in progress calculation",
        linear.alpha
    );
}

// =========================================================================
// Part F: Shockwave Visual Effect (VFX)
// =========================================================================

/// Behavior V1: Observer spawns entity with `Mesh2d` and
/// `MeshMaterial2d<ColorMaterial>` in addition to all existing components.
#[test]
fn observer_spawns_shockwave_with_mesh_and_material() {
    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 50.0, 100.0);

    trigger_shockwave(&mut app, bolt, 96.0, 400.0);

    let sw = get_shockwave_entity(&mut app);
    let world = app.world();

    // Visual components: Mesh2d and MeshMaterial2d<ColorMaterial>
    assert!(
        world.get::<Mesh2d>(sw).is_some(),
        "shockwave entity should have Mesh2d component"
    );
    assert!(
        world.get::<MeshMaterial2d<ColorMaterial>>(sw).is_some(),
        "shockwave entity should have MeshMaterial2d<ColorMaterial> component"
    );

    // Existing components still present (additive, not replacing)
    assert!(
        world.get::<ShockwaveRadius>(sw).is_some(),
        "ShockwaveRadius should still be present"
    );
    assert!(
        world.get::<ShockwaveSpeed>(sw).is_some(),
        "ShockwaveSpeed should still be present"
    );
    assert!(
        world.get::<ShockwaveDamage>(sw).is_some(),
        "ShockwaveDamage should still be present"
    );
    assert!(
        world.get::<ShockwaveAlreadyHit>(sw).is_some(),
        "ShockwaveAlreadyHit should still be present"
    );
    assert!(
        world
            .get::<rantzsoft_spatial2d::components::Position2D>(sw)
            .is_some(),
        "Position2D should still be present"
    );
    assert!(
        world.get::<Scale2D>(sw).is_some(),
        "Scale2D should still be present"
    );
    assert!(
        world.get::<crate::shared::GameDrawLayer>(sw).is_some(),
        "GameDrawLayer should still be present"
    );
    assert!(
        world.get::<crate::shared::CleanupOnNodeExit>(sw).is_some(),
        "CleanupOnNodeExit should still be present"
    );
    assert!(
        world
            .get::<rantzsoft_spatial2d::components::Spatial2D>(sw)
            .is_some(),
        "Spatial2D should still be present"
    );
}

/// Behavior V2: Material uses HDR emissive cyan with
/// `AlphaMode2d::Blend`.
#[test]
fn observer_spawns_shockwave_with_hdr_emissive_material() {
    use bevy::sprite_render::AlphaMode2d;

    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 0.0, 0.0);

    trigger_shockwave(&mut app, bolt, 96.0, 400.0);

    let sw = get_shockwave_entity(&mut app);

    let mat_handle = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(sw)
        .expect("shockwave should have MeshMaterial2d<ColorMaterial>");
    let materials = app.world().resource::<Assets<ColorMaterial>>();
    let material = materials
        .get(mat_handle.id())
        .expect("material asset should exist");

    // Alpha mode must be Blend for transparency
    assert_eq!(
        material.alpha_mode,
        AlphaMode2d::Blend,
        "material alpha_mode should be AlphaMode2d::Blend"
    );

    // Color should be HDR emissive cyan: linear_rgba(0.0, 4.0, 4.0, 0.9)
    let linear = material.color.to_linear();
    assert!(
        linear.red.abs() < 0.01,
        "red channel should be 0.0, got {}",
        linear.red
    );
    assert!(
        (linear.green - 4.0).abs() < 0.01,
        "green channel should be 4.0, got {}",
        linear.green
    );
    assert!(
        (linear.blue - 4.0).abs() < 0.01,
        "blue channel should be 4.0, got {}",
        linear.blue
    );
    assert!(
        (linear.alpha - 0.9).abs() < 0.01,
        "alpha should be 0.9, got {}",
        linear.alpha
    );

    // Edge case: alpha must NOT be 1.0
    assert!(
        (linear.alpha - 1.0).abs() > 0.01,
        "starting alpha must NOT be 1.0 (should be 0.9)"
    );
}

/// Behavior V3: `animate_shockwave` fades alpha based on expansion
/// progress. At 50% expanded, alpha = 0.9 * (1.0 - 0.5) = 0.45.
#[test]
fn animate_shockwave_fades_alpha_at_half_expansion() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<ColorMaterial>>();
    app.add_systems(FixedUpdate, animate_shockwave);

    // Create a material with starting color
    let mut materials = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
    let mat_handle = materials.add(ColorMaterial {
        color: Color::linear_rgba(0.0, 4.0, 4.0, 0.9),
        alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
        ..Default::default()
    });

    let entity = app
        .world_mut()
        .spawn((
            ShockwaveRadius {
                current: 48.0,
                max: 96.0,
            },
            Scale2D::default(),
            MeshMaterial2d(mat_handle),
        ))
        .id();

    tick(&mut app);

    // Scale2D should be current * 2.0 = 96.0
    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("entity should have Scale2D");
    assert!(
        (scale.x - 96.0).abs() < f32::EPSILON,
        "Scale2D.x should be 96.0 (48.0 * 2.0), got {}",
        scale.x
    );
    assert!(
        (scale.y - 96.0).abs() < f32::EPSILON,
        "Scale2D.y should be 96.0 (48.0 * 2.0), got {}",
        scale.y
    );

    // Alpha should be 0.9 * (1.0 - 48.0/96.0) = 0.9 * 0.5 = 0.45
    let mat_handle = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(entity)
        .unwrap();
    let materials = app.world().resource::<Assets<ColorMaterial>>();
    let material = materials.get(mat_handle.id()).unwrap();
    let linear = material.color.to_linear();
    assert!(
        (linear.alpha - 0.45).abs() < 0.01,
        "alpha should be ~0.45 at 50% expansion, got {}",
        linear.alpha
    );

    // HDR color channels should remain unchanged
    assert!(
        linear.red.abs() < 0.01,
        "red should remain 0.0, got {}",
        linear.red
    );
    assert!(
        (linear.green - 4.0).abs() < 0.01,
        "green should remain 4.0, got {}",
        linear.green
    );
    assert!(
        (linear.blue - 4.0).abs() < 0.01,
        "blue should remain 4.0, got {}",
        linear.blue
    );
}

/// Behavior V3 edge case: At 0% expansion (current=0.0), alpha should
/// remain at 0.9 (no fade yet) and `Scale2D` should be (0.0, 0.0).
#[test]
fn animate_shockwave_no_fade_at_zero_progress() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<ColorMaterial>>();
    app.add_systems(FixedUpdate, animate_shockwave);

    let mut materials = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
    let mat_handle = materials.add(ColorMaterial {
        color: Color::linear_rgba(0.0, 4.0, 4.0, 0.9),
        alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
        ..Default::default()
    });

    let entity = app
        .world_mut()
        .spawn((
            ShockwaveRadius {
                current: 0.0,
                max: 96.0,
            },
            Scale2D::default(),
            MeshMaterial2d(mat_handle),
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        scale.x.abs() < f32::EPSILON,
        "Scale2D.x should be 0.0 at zero radius, got {}",
        scale.x
    );
    assert!(
        scale.y.abs() < f32::EPSILON,
        "Scale2D.y should be 0.0 at zero radius, got {}",
        scale.y
    );

    let mat_handle = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(entity)
        .unwrap();
    let materials = app.world().resource::<Assets<ColorMaterial>>();
    let material = materials.get(mat_handle.id()).unwrap();
    let linear = material.color.to_linear();
    assert!(
        (linear.alpha - 0.9).abs() < 0.01,
        "alpha should remain 0.9 at 0% progress, got {}",
        linear.alpha
    );
}

/// Behavior V4: At 100% expansion (current == max), alpha reaches ~0.0.
#[test]
fn animate_shockwave_alpha_reaches_zero_at_max_radius() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<ColorMaterial>>();
    app.add_systems(FixedUpdate, animate_shockwave);

    let mut materials = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
    let mat_handle = materials.add(ColorMaterial {
        color: Color::linear_rgba(0.0, 4.0, 4.0, 0.9),
        alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
        ..Default::default()
    });

    let entity = app
        .world_mut()
        .spawn((
            ShockwaveRadius {
                current: 96.0,
                max: 96.0,
            },
            Scale2D::default(),
            MeshMaterial2d(mat_handle),
        ))
        .id();

    tick(&mut app);

    // Scale2D = current * 2.0 = 192.0
    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 192.0).abs() < f32::EPSILON,
        "Scale2D.x should be 192.0 (96.0 * 2.0), got {}",
        scale.x
    );
    assert!(
        (scale.y - 192.0).abs() < f32::EPSILON,
        "Scale2D.y should be 192.0 (96.0 * 2.0), got {}",
        scale.y
    );

    // Alpha should be 0.9 * (1.0 - 96.0/96.0) = 0.0
    let mat_handle = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(entity)
        .unwrap();
    let materials = app.world().resource::<Assets<ColorMaterial>>();
    let material = materials.get(mat_handle.id()).unwrap();
    let linear = material.color.to_linear();
    assert!(
        linear.alpha.abs() < 0.01,
        "alpha should be ~0.0 at 100% expansion, got {}",
        linear.alpha
    );
}

/// Behavior V4 edge case: current slightly exceeding max — alpha clamps
/// to 0.0, does not go negative.
#[test]
fn animate_shockwave_alpha_clamps_when_exceeding_max() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<ColorMaterial>>();
    app.add_systems(FixedUpdate, animate_shockwave);

    let mut materials = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
    let mat_handle = materials.add(ColorMaterial {
        color: Color::linear_rgba(0.0, 4.0, 4.0, 0.9),
        alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
        ..Default::default()
    });

    let entity = app
        .world_mut()
        .spawn((
            ShockwaveRadius {
                current: 100.0,
                max: 96.0,
            },
            Scale2D::default(),
            MeshMaterial2d(mat_handle),
        ))
        .id();

    tick(&mut app);

    let mat_handle = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(entity)
        .unwrap();
    let materials = app.world().resource::<Assets<ColorMaterial>>();
    let material = materials.get(mat_handle.id()).unwrap();
    let linear = material.color.to_linear();
    assert!(
        linear.alpha >= 0.0,
        "alpha must not go negative when current exceeds max, got {}",
        linear.alpha
    );
    assert!(
        linear.alpha.abs() < 0.01,
        "alpha should clamp to ~0.0 when exceeding max, got {}",
        linear.alpha
    );
}

/// Behavior V5: Entity with stale `MeshMaterial2d` handle (asset removed)
/// — `Scale2D` still updates, no panic.
#[test]
fn animate_shockwave_handles_missing_material_gracefully() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<ColorMaterial>>();
    app.add_systems(FixedUpdate, animate_shockwave);

    // Create and immediately remove the material asset
    let mut materials = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
    let mat_handle = materials.add(ColorMaterial {
        color: Color::linear_rgba(0.0, 4.0, 4.0, 0.9),
        alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
        ..Default::default()
    });
    let handle_id = mat_handle.id();
    materials.remove(handle_id);

    let entity = app
        .world_mut()
        .spawn((
            ShockwaveRadius {
                current: 48.0,
                max: 96.0,
            },
            Scale2D::default(),
            MeshMaterial2d(mat_handle),
        ))
        .id();

    // Should not panic
    tick(&mut app);

    // Scale2D should still be updated
    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 96.0).abs() < f32::EPSILON,
        "Scale2D.x should still be 96.0 even with missing material, got {}",
        scale.x
    );
    assert!(
        (scale.y - 96.0).abs() < f32::EPSILON,
        "Scale2D.y should still be 96.0 even with missing material, got {}",
        scale.y
    );
}

/// Behavior V6: Entity without `MeshMaterial2d` — `Scale2D` still
/// updates, no panic. Backward compat with existing test entities.
#[test]
fn animate_shockwave_works_without_material_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<ColorMaterial>>();
    app.add_systems(FixedUpdate, animate_shockwave);

    let entity = app
        .world_mut()
        .spawn((
            ShockwaveRadius {
                current: 48.0,
                max: 96.0,
            },
            Scale2D::default(),
            // No MeshMaterial2d component
        ))
        .id();

    // Should not panic
    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 96.0).abs() < f32::EPSILON,
        "Scale2D.x should be 96.0 without MeshMaterial2d, got {}",
        scale.x
    );
    assert!(
        (scale.y - 96.0).abs() < f32::EPSILON,
        "Scale2D.y should be 96.0 without MeshMaterial2d, got {}",
        scale.y
    );
}

/// Behavior V7: Multiple shockwave entities have independent materials
/// — each fades its own alpha independently.
#[test]
fn multiple_shockwaves_have_independent_materials() {
    let mut app = test_app();
    let bolt_a = spawn_bolt(&mut app, 10.0, 10.0);
    let bolt_b = spawn_bolt(&mut app, 20.0, 20.0);

    // Spawn two shockwaves via separate triggers
    trigger_shockwave(&mut app, bolt_a, 96.0, 400.0);
    trigger_shockwave(&mut app, bolt_b, 96.0, 400.0);

    // Collect all shockwave entities
    let shockwaves: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveRadius>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        shockwaves.len(),
        2,
        "should have exactly 2 shockwave entities"
    );

    let sw_a = shockwaves[0];
    let sw_b = shockwaves[1];

    // Verify each has its own MeshMaterial2d handle
    let handle_a = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(sw_a)
        .expect("entity A should have MeshMaterial2d");
    let handle_b = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(sw_b)
        .expect("entity B should have MeshMaterial2d");

    // Handles must be different — not sharing a material asset
    assert_ne!(
        handle_a.id(),
        handle_b.id(),
        "shockwave entities must have independent material handles"
    );

    // Set different radii: A at 25%, B at 75%
    app.world_mut()
        .get_mut::<ShockwaveRadius>(sw_a)
        .unwrap()
        .current = 24.0;
    app.world_mut()
        .get_mut::<ShockwaveRadius>(sw_b)
        .unwrap()
        .current = 72.0;

    // Add animate_shockwave system and run
    app.add_systems(FixedUpdate, animate_shockwave);
    tick(&mut app);

    // Entity A: alpha = 0.9 * (1.0 - 24.0/96.0) = 0.9 * 0.75 = 0.675
    let handle_a = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(sw_a)
        .unwrap();
    let materials = app.world().resource::<Assets<ColorMaterial>>();
    let mat_a = materials.get(handle_a.id()).unwrap();
    let linear_a = mat_a.color.to_linear();
    assert!(
        (linear_a.alpha - 0.675).abs() < 0.01,
        "entity A alpha should be ~0.675 (25% progress), got {}",
        linear_a.alpha
    );

    // Entity B: alpha = 0.9 * (1.0 - 72.0/96.0) = 0.9 * 0.25 = 0.225
    let handle_b = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(sw_b)
        .unwrap();
    let mat_b = materials.get(handle_b.id()).unwrap();
    let linear_b = mat_b.color.to_linear();
    assert!(
        (linear_b.alpha - 0.225).abs() < 0.01,
        "entity B alpha should be ~0.225 (75% progress), got {}",
        linear_b.alpha
    );
}
