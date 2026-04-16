//! Portal behavior — builder sugar, definition dispatch, and `CellBehavior` variant.
//!
//! Tests exercise `.portal(tier_offset)`, `.with_behavior(CellBehavior::Portal { .. })`,
//! and `.definition(&def)` against the `spawn_inner()` match arm. They assert
//! the `PortalCell` marker and `PortalConfig { tier_offset }`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::{
        behaviors::portal::components::{PortalCell, PortalConfig},
        components::VolatileCell,
        definition::CellBehavior,
    },
    prelude::*,
    shared::death_pipeline::invulnerable::Invulnerable,
};

// ── Behavior 1: CellBehavior::Portal variant exists ────────────────────────

#[test]
fn cell_behavior_portal_deserializes_from_ron() {
    let ron_str = "Portal(sub_node_tier_offset: 2)";
    let result: CellBehavior = ron::de::from_str(ron_str).expect("should deserialize Portal");
    assert_eq!(
        result,
        CellBehavior::Portal {
            sub_node_tier_offset: 2,
        }
    );
}

#[test]
fn cell_behavior_portal_negative_offset_deserializes_from_ron() {
    let ron_str = "Portal(sub_node_tier_offset: -3)";
    let result: CellBehavior =
        ron::de::from_str(ron_str).expect("should deserialize negative offset");
    assert_eq!(
        result,
        CellBehavior::Portal {
            sub_node_tier_offset: -3,
        }
    );
}

#[test]
fn cell_behavior_portal_zero_offset_deserializes_from_ron() {
    let ron_str = "Portal(sub_node_tier_offset: 0)";
    let result: CellBehavior = ron::de::from_str(ron_str).expect("should deserialize zero offset");
    assert_eq!(
        result,
        CellBehavior::Portal {
            sub_node_tier_offset: 0,
        }
    );
}

// ── Behavior 2: CellTypeDefinition with Portal passes validation ───────────

#[test]
fn definition_with_portal_behavior_passes_validation() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Portal {
        sub_node_tier_offset: 1,
    }]);
    assert!(
        def.validate().is_ok(),
        "Portal behavior should pass validation"
    );
}

#[test]
fn definition_with_portal_i32_min_passes_validation() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Portal {
        sub_node_tier_offset: i32::MIN,
    }]);
    assert!(
        def.validate().is_ok(),
        "Portal with i32::MIN tier offset should pass validation"
    );
}

#[test]
fn definition_with_portal_i32_max_passes_validation() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Portal {
        sub_node_tier_offset: i32::MAX,
    }]);
    assert!(
        def.validate().is_ok(),
        "Portal with i32::MAX tier offset should pass validation"
    );
}

// ── Behavior 3: Builder .portal() inserts PortalCell marker ────────────────

#[test]
fn spawn_with_portal_sugar_inserts_marker() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .portal(2)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<PortalCell>(entity).is_some(),
        "entity should have PortalCell marker"
    );
}

#[test]
fn spawn_with_portal_negative_offset_inserts_marker() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .portal(-1)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<PortalCell>(entity).is_some(),
        "entity should have PortalCell marker for negative offset"
    );
}

// ── Behavior 4: Builder .portal() inserts PortalConfig ─────────────────────

#[test]
fn spawn_with_portal_sugar_inserts_config() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .portal(3)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let config = world
        .get::<PortalConfig>(entity)
        .expect("entity should have PortalConfig");
    assert_eq!(
        *config,
        PortalConfig { tier_offset: 3 },
        "tier_offset should be 3"
    );
}

#[test]
fn spawn_with_portal_zero_offset_inserts_config() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .portal(0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let config = world
        .get::<PortalConfig>(entity)
        .expect("entity should have PortalConfig for zero offset");
    assert_eq!(
        *config,
        PortalConfig { tier_offset: 0 },
        "tier_offset should be 0"
    );
}

// ── Behavior 5: .with_behavior(Portal) matches .portal() sugar ─────────────

#[test]
fn spawn_with_behavior_portal_matches_portal_sugar() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Portal {
                sub_node_tier_offset: 2,
            })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<PortalCell>(entity).is_some(),
        "entity should have PortalCell marker via with_behavior"
    );
    let config = world
        .get::<PortalConfig>(entity)
        .expect("entity should have PortalConfig via with_behavior");
    assert_eq!(
        *config,
        PortalConfig { tier_offset: 2 },
        "tier_offset should be 2 via with_behavior"
    );
}

// ── Behavior 6: Definition-sourced Portal inserts markers and config ────────

#[test]
fn spawn_portal_through_definition_inserts_marker_and_config() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Portal {
        sub_node_tier_offset: -2,
    }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<PortalCell>(entity).is_some(),
        "definition-sourced Portal should insert PortalCell marker"
    );
    let config = world
        .get::<PortalConfig>(entity)
        .expect("definition-sourced Portal should insert PortalConfig");
    assert_eq!(
        *config,
        PortalConfig { tier_offset: -2 },
        "tier_offset should be -2 from definition"
    );
}

// ── Behavior 7: Cell without Portal has no portal components ────────────────

#[test]
fn spawn_without_portal_has_no_portal_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<PortalCell>(entity).is_none(),
        "entity should NOT have PortalCell marker"
    );
    assert!(
        world.get::<PortalConfig>(entity).is_none(),
        "entity should NOT have PortalConfig"
    );

    // Guard: prove the builder actually ran (prevents false-pass under a no-op stub).
    let hp = world
        .get::<Hp>(entity)
        .expect("entity should have Hp from builder");
    assert!((hp.current - 20.0).abs() < f32::EPSILON);
}

// ── Behavior 8: Portal coexists with other behaviors ────────────────────────

#[test]
fn spawn_portal_with_volatile_inserts_both_markers() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .portal(1)
            .volatile(25.0, 40.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<PortalCell>(entity).is_some(),
        "entity should have PortalCell marker"
    );
    assert!(
        world.get::<VolatileCell>(entity).is_some(),
        "entity should have VolatileCell marker"
    );
    let config = world
        .get::<PortalConfig>(entity)
        .expect("entity should have PortalConfig");
    assert_eq!(
        *config,
        PortalConfig { tier_offset: 1 },
        "tier_offset should be 1"
    );
    assert!(
        world.get::<Invulnerable>(entity).is_some(),
        "portal cell should have Invulnerable marker"
    );
}

#[test]
fn spawn_portal_with_regen_through_definition_inserts_both() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![
        CellBehavior::Portal {
            sub_node_tier_offset: 1,
        },
        CellBehavior::Regen { rate: 2.0 },
    ]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<PortalCell>(entity).is_some(),
        "entity should have PortalCell marker"
    );
    assert!(
        world.get::<RegenCell>(entity).is_some(),
        "entity should have RegenCell marker"
    );
    let regen_rate = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!((regen_rate.0 - 2.0).abs() < f32::EPSILON);
    let config = world
        .get::<PortalConfig>(entity)
        .expect("entity should have PortalConfig");
    assert_eq!(
        *config,
        PortalConfig { tier_offset: 1 },
        "tier_offset should be 1"
    );
}

// ── Behavior 5: Builder .portal() inserts Invulnerable ────────────────────

#[test]
fn spawn_with_portal_inserts_invulnerable() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .portal(1)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Invulnerable>(entity).is_some(),
        "portal cell should have Invulnerable marker"
    );
}

#[test]
fn spawn_portal_through_definition_inserts_invulnerable() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Portal {
        sub_node_tier_offset: -2,
    }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Invulnerable>(entity).is_some(),
        "definition-sourced portal should also have Invulnerable marker"
    );
}

#[test]
fn spawn_without_portal_has_no_invulnerable() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Invulnerable>(entity).is_none(),
        "non-portal cell should NOT have Invulnerable marker"
    );
    // Guard: prove builder actually ran
    let hp = world
        .get::<Hp>(entity)
        .expect("entity should have Hp from builder");
    assert!((hp.current - 20.0).abs() < f32::EPSILON);
}
