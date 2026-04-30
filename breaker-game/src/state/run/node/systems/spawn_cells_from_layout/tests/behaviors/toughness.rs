use bevy::prelude::*;

use super::{
    super::{super::spawning::spawn_cells_from_layout, helpers::*},
    helpers::*,
};
use crate::{
    cells::{
        CellTypeDefinition,
        definition::{CellBehavior, GuardedBehavior, Toughness},
        resources::{CellConfig, CellTypeRegistry},
    },
    prelude::*,
    state::run::{
        node::{ActiveNodeLayout, definition::NodePool, messages::CellsSpawned},
        resources::{NodeAssignment, NodeOutcome, NodeSequence},
    },
};

// ── Part L: Toughness-based HP computation in spawn system ─────────────

/// Creates a registry with a guarded cell type ("Gu") that has Tough toughness
/// and `guardian_hp_fraction: 0.5`, plus "S" (Standard) and "T" (Tough).
fn toughness_registry_with_guarded() -> CellTypeRegistry {
    let mut registry = test_registry(); // already has "S" (Standard), "T" (Tough)
    registry.insert(
        "Gu".to_owned(),
        CellTypeDefinition {
            id:                "guarded".to_owned(),
            alias:             "Gu".to_owned(),
            toughness:         Toughness::Tough,
            color_rgb:         [1.0, 0.8, 0.2],
            required_to_clear: true,
            damage_hdr_base:   4.0,
            damage_green_min:  0.2,
            damage_blue_range: 0.4,
            damage_blue_base:  0.2,
            behaviors:         Some(vec![CellBehavior::Guarded(GuardedBehavior {
                guardian_hp_fraction: 0.5,
                guardian_color_rgb:   [0.5, 0.8, 1.0],
                slide_speed:          30.0,
            })]),
            effects:           None,
        },
    );
    // "gu" is the guardian child cell type consumed by the guarded parent
    registry.insert(
        "gu".to_owned(),
        CellTypeDefinition {
            id:                "guardian".to_owned(),
            alias:             "gu".to_owned(),
            toughness:         Toughness::Weak,
            color_rgb:         [0.5, 0.8, 1.0],
            required_to_clear: false,
            damage_hdr_base:   4.0,
            damage_green_min:  0.2,
            damage_blue_range: 0.4,
            damage_blue_base:  0.2,
            behaviors:         None,
            effects:           None,
        },
    );
    registry
}

#[test]
fn cell_hp_falls_back_to_default_base_hp_without_config() {
    let layout = NodeLayout {
        name:            "hp_mult_test".to_owned(),
        timer_secs:      60.0,
        cols:            1,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellsSpawned>()
        .init_resource::<CellConfig>()
        .init_resource::<PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(test_registry())
        .insert_resource(NodeOutcome {
            node_index: 0,
            ..Default::default()
        })
        .insert_resource(NodeSequence {
            assignments: vec![NodeAssignment {
                node_type:  NodeType::Active,
                tier_index: 0,

                timer_mult: 1.0,
            }],
        })
        .add_systems(Startup, spawn_cells_from_layout);
    app.update();

    // 'S' is Standard toughness — without ToughnessConfig, falls back to default_base_hp = 20.0
    let healths: Vec<&Hp> = app.world_mut().query::<&Hp>().iter(app.world()).collect();
    assert_eq!(healths.len(), 1);
    assert!(
        (healths[0].current - 20.0).abs() < f32::EPSILON,
        "cell current HP should be Standard default_base_hp = 20.0, got {}",
        healths[0].current
    );
    assert!(
        (healths[0].starting - 20.0).abs() < f32::EPSILON,
        "cell max HP should be Standard default_base_hp = 20.0, got {}",
        healths[0].starting
    );
}

#[test]
fn cell_hp_tough_falls_back_to_default_base_hp() {
    let layout = NodeLayout {
        name:            "hp_mult_one_test".to_owned(),
        timer_secs:      60.0,
        cols:            1,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("T")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellsSpawned>()
        .init_resource::<CellConfig>()
        .init_resource::<PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(test_registry())
        .insert_resource(NodeOutcome {
            node_index: 0,
            ..Default::default()
        })
        .insert_resource(NodeSequence {
            assignments: vec![NodeAssignment {
                node_type:  NodeType::Passive,
                tier_index: 0,

                timer_mult: 1.0,
            }],
        })
        .add_systems(Startup, spawn_cells_from_layout);
    app.update();

    // 'T' is Tough toughness — without ToughnessConfig, falls back to default_base_hp = 30.0
    let healths: Vec<&Hp> = app.world_mut().query::<&Hp>().iter(app.world()).collect();
    assert_eq!(healths.len(), 1);
    assert!(
        (healths[0].current - 30.0).abs() < f32::EPSILON,
        "cell current HP should be Tough default_base_hp = 30.0, got {}",
        healths[0].current
    );
    assert!(
        (healths[0].starting - 30.0).abs() < f32::EPSILON,
        "cell max HP should be Tough default_base_hp = 30.0, got {}",
        healths[0].starting
    );
}

// Behavior 36: Standard cell at tier 0, pos 0 → CellHealth { current: 20.0, max: 20.0 }
#[test]
fn spawn_standard_cell_tier0_pos0_has_correct_hp() {
    let layout = NodeLayout {
        name:            "toughness_test".to_owned(),
        timer_secs:      60.0,
        cols:            1,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    };
    let mut app = test_app_with_toughness(layout, test_registry(), 0, 0, false);
    app.update();

    let healths: Vec<&Hp> = app.world_mut().query::<&Hp>().iter(app.world()).collect();
    assert_eq!(healths.len(), 1, "should spawn exactly 1 cell");
    assert!(
        (healths[0].current - 20.0).abs() < 0.001,
        "Standard cell at tier 0, pos 0 should have HP 20.0, got {}",
        healths[0].current
    );
    assert!(
        (healths[0].starting - 20.0).abs() < 0.001,
        "Standard cell at tier 0, pos 0 should have max HP 20.0, got {}",
        healths[0].starting
    );
}

// Behavior 37: Tough cell at tier 3, pos 4 → CellHealth { current: ~62.208, max: ~62.208 }
#[test]
fn spawn_tough_cell_tier3_pos4_has_scaled_hp() {
    let layout = NodeLayout {
        name:            "toughness_scaled_test".to_owned(),
        timer_secs:      60.0,
        cols:            1,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("T")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    };
    let mut app = test_app_with_toughness(layout, test_registry(), 3, 4, false);
    app.update();

    let healths: Vec<&Hp> = app.world_mut().query::<&Hp>().iter(app.world()).collect();
    assert_eq!(healths.len(), 1, "should spawn exactly 1 cell");
    // Tough base=30.0, tier_scale(3,4) = 1.2^3 * (1.0 + 0.05*4) = 1.728 * 1.2 = 2.0736
    // HP = 30.0 * 2.0736 ≈ 62.208
    assert!(
        (healths[0].current - 62.208).abs() < 0.01,
        "Tough cell at tier 3, pos 4 should have HP ~62.208, got {}",
        healths[0].current
    );
    assert!(
        (healths[0].starting - 62.208).abs() < 0.01,
        "Tough cell at tier 3, pos 4 should have max HP ~62.208, got {}",
        healths[0].starting
    );
}

// Behavior 38: No ToughnessConfig → falls back to default_base_hp()
#[test]
fn spawn_cell_without_toughness_config_falls_back_to_default_base_hp() {
    let layout = NodeLayout {
        name:            "no_config_test".to_owned(),
        timer_secs:      60.0,
        cols:            1,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    };
    // Deliberately do NOT insert ToughnessConfig resource
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellsSpawned>()
        .init_resource::<CellConfig>()
        .init_resource::<PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(test_registry())
        .add_systems(Startup, spawn_cells_from_layout);
    app.update();

    let healths: Vec<&Hp> = app.world_mut().query::<&Hp>().iter(app.world()).collect();
    assert_eq!(healths.len(), 1, "should spawn exactly 1 cell");
    // Falls back to Standard.default_base_hp() = 20.0
    assert!(
        (healths[0].current - 20.0).abs() < 0.001,
        "Without ToughnessConfig, Standard cell should fall back to default_base_hp() = 20.0, got {}",
        healths[0].current
    );
}

// Behavior 39: Guarded cell at tier 0, pos 0, Tough, guardian_hp_fraction=0.5
//              → parent HP = 30.0, guardian HP = 30.0 * 0.5 = 15.0
#[test]
fn spawn_guarded_cell_tier0_pos0_guardian_hp_is_parent_times_fraction() {
    // Layout: 3x1 with guarded cell in center flanked by guardian children
    let layout = NodeLayout {
        name:            "guarded_hp_test".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("gu"), s("Gu"), s("gu")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    };
    let mut app = test_app_with_toughness(layout, toughness_registry_with_guarded(), 0, 0, false);
    app.update();

    // Find guardian cells (they should have GuardianMarker or lower HP)
    // The guarded parent has Tough toughness, HP = 30.0 at tier 0 pos 0
    // Guardian HP = 30.0 * 0.5 = 15.0
    let healths: Vec<f32> = app
        .world_mut()
        .query::<&Hp>()
        .iter(app.world())
        .map(|h| h.current)
        .collect();

    // There should be 3 entities: 1 parent + 2 guardians
    // (guardians are spawned by the guarded parent's builder)
    // Parent HP should be 30.0, each guardian HP should be 15.0
    let parent_hp = healths
        .iter()
        .copied()
        .filter(|&h| (h - 30.0).abs() < 0.001)
        .count();
    let guardian_hp = healths
        .iter()
        .copied()
        .filter(|&h| (h - 15.0).abs() < 0.001)
        .count();
    assert!(
        parent_hp >= 1,
        "should have at least 1 entity with parent HP 30.0 (Tough, tier 0, pos 0), got healths: {healths:?}"
    );
    assert!(
        guardian_hp >= 1,
        "should have at least 1 guardian entity with HP 15.0 (30.0 * 0.5), got healths: {healths:?}"
    );
}

// Behavior 40: Guardian HP scales with tier: tier 3, pos 0, Tough
//              → parent ≈ 51.84, guardian ≈ 25.92
#[test]
fn spawn_guarded_cell_tier3_pos0_guardian_hp_scales_with_tier() {
    let layout = NodeLayout {
        name:            "guarded_scaled_test".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("gu"), s("Gu"), s("gu")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    };
    let mut app = test_app_with_toughness(layout, toughness_registry_with_guarded(), 3, 0, false);
    app.update();

    // Tough base=30.0, tier_scale(3,0) = 1.2^3 = 1.728
    // Parent HP = 30.0 * 1.728 ≈ 51.84
    // Guardian HP = 51.84 * 0.5 ≈ 25.92
    let healths: Vec<f32> = app
        .world_mut()
        .query::<&Hp>()
        .iter(app.world())
        .map(|h| h.current)
        .collect();

    let parent_hp = healths
        .iter()
        .copied()
        .filter(|&h| (h - 51.84).abs() < 0.1)
        .count();
    let guardian_hp = healths
        .iter()
        .copied()
        .filter(|&h| (h - 25.92).abs() < 0.1)
        .count();
    assert!(
        parent_hp >= 1,
        "should have at least 1 entity with parent HP ~51.84 (Tough, tier 3, pos 0), got healths: {healths:?}"
    );
    assert!(
        guardian_hp >= 1,
        "should have at least 1 guardian entity with HP ~25.92 (51.84 * 0.5), got healths: {healths:?}"
    );
}

// Behavior 41: Boss guardian HP: tier 0, pos 0, Tough, is_boss=true
//              → parent = 90.0 (30.0 * 1.0 * 3.0 boss_mult), guardian = 45.0
#[test]
fn spawn_boss_guarded_cell_applies_boss_multiplier_before_guardian_fraction() {
    let layout = NodeLayout {
        name:            "boss_guarded_test".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("gu"), s("Gu"), s("gu")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    };
    let mut app = test_app_with_toughness(
        layout,
        toughness_registry_with_guarded(),
        0,
        0,
        true, // is_boss
    );
    app.update();

    // Tough base=30.0, boss_multiplier=3.0
    // Parent HP = 30.0 * 1.0 * 3.0 = 90.0
    // Guardian HP = 90.0 * 0.5 = 45.0
    let healths: Vec<f32> = app
        .world_mut()
        .query::<&Hp>()
        .iter(app.world())
        .map(|h| h.current)
        .collect();

    let parent_hp = healths
        .iter()
        .copied()
        .filter(|&h| (h - 90.0).abs() < 0.1)
        .count();
    let guardian_hp = healths
        .iter()
        .copied()
        .filter(|&h| (h - 45.0).abs() < 0.1)
        .count();
    assert!(
        parent_hp >= 1,
        "should have at least 1 entity with boss parent HP 90.0 (Tough, boss_mult 3.0), got healths: {healths:?}"
    );
    assert!(
        guardian_hp >= 1,
        "should have at least 1 guardian entity with HP 45.0 (90.0 * 0.5), got healths: {healths:?}"
    );
}

// Behavior 42: resolve_hp_mult no longer exists — HP computed from ToughnessConfig.
// This is a compile-time structural assertion: `resolve_hp_context` is the
// replacement function name. If `resolve_hp_mult` were still public, referencing
// `resolve_hp_context` would fail to compile.
// The function is private, so we cannot reference it directly. Instead, the
// behavioral tests above verify the correct HP computation path.
