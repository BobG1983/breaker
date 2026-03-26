use bevy::prelude::*;

use super::super::*;

// =========================================================================
// C7 Wave 1 Part E: EffectEntity rename + new EffectTarget (behaviors 26-30)
// =========================================================================

#[test]
fn effect_entity_is_queryable_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(EffectEntity).id();
    assert!(
        app.world().entity(entity).contains::<EffectEntity>(),
        "EffectEntity should be queryable via With<EffectEntity>"
    );
}

#[test]
fn effect_target_entity_variant() {
    let entity = Entity::PLACEHOLDER;
    let target = EffectTarget::Entity(entity);
    match target {
        EffectTarget::Entity(e) => assert_eq!(e, entity),
        other @ EffectTarget::Location(_) => {
            panic!("expected EffectTarget::Entity, got {other:?}")
        }
    }
}

#[test]
fn effect_target_location_variant() {
    let target = EffectTarget::Location(Vec2::new(100.0, 200.0));
    match target {
        EffectTarget::Location(pos) => {
            assert_eq!(pos, Vec2::new(100.0, 200.0));
        }
        other @ EffectTarget::Entity(_) => {
            panic!("expected EffectTarget::Location, got {other:?}")
        }
    }
}

#[test]
fn effect_target_location_zero_is_valid() {
    let target = EffectTarget::Location(Vec2::ZERO);
    assert_eq!(target, EffectTarget::Location(Vec2::ZERO));
}

#[test]
fn effect_target_empty_vec_is_valid() {
    let targets: Vec<EffectTarget> = Vec::new();
    assert!(targets.is_empty());
}

#[test]
fn effect_target_multiple_entities() {
    let targets = [
        EffectTarget::Entity(Entity::PLACEHOLDER),
        EffectTarget::Entity(Entity::PLACEHOLDER),
    ];
    assert_eq!(targets.len(), 2);
}

// =========================================================================
// Target expansion — new variants (Part C)
// =========================================================================

#[test]
fn target_cell_deserializes() {
    let t: Target = ron::de::from_str("Cell").expect("Target::Cell RON should parse");
    assert_eq!(t, Target::Cell);
}

#[test]
fn target_wall_deserializes() {
    let t: Target = ron::de::from_str("Wall").expect("Target::Wall RON should parse");
    assert_eq!(t, Target::Wall);
}

#[test]
fn target_all_cells_deserializes() {
    let t: Target = ron::de::from_str("AllCells").expect("Target::AllCells RON should parse");
    assert_eq!(t, Target::AllCells);
}
