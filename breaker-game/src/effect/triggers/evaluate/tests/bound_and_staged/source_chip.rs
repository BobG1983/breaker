//! Tests for `EffectSourceChip` threading through `evaluate_bound_effects` and
//! `evaluate_staged_effects`.

use bevy::prelude::*;

use super::helpers::*;
use crate::effect::{core::EffectSourceChip, effects::explode::ExplodeRequest};

fn evaluate_source_chip_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, sys_evaluate_bound_for_bump);
    app.add_systems(Update, sys_evaluate_staged_for_bump);
    app
}

#[test]
fn evaluate_bound_effects_threads_chip_name_as_source_chip_to_fire_effect() {
    let mut app = evaluate_source_chip_test_app();

    let node = when_do(
        Trigger::Bump,
        EffectKind::Explode {
            range: 60.0,
            damage: 2.0,
        },
    );

    app.world_mut().spawn((
        BoundEffects(vec![("resonance".into(), node)]),
        StagedEffects::default(),
        Transform::from_xyz(50.0, 50.0, 0.0),
    ));

    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<&EffectSourceChip, With<ExplodeRequest>>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip (on ExplodeRequest)"
    );
    assert_eq!(
        results[0].0,
        Some("resonance".to_string()),
        "evaluate_bound_effects should thread chip_name 'resonance' to fire_effect"
    );
}

#[test]
fn evaluate_bound_effects_empty_chip_name_produces_none() {
    let mut app = evaluate_source_chip_test_app();

    let node = when_do(
        Trigger::Bump,
        EffectKind::Explode {
            range: 60.0,
            damage: 2.0,
        },
    );

    app.world_mut().spawn((
        BoundEffects(vec![(String::new(), node)]),
        StagedEffects::default(),
        Transform::from_xyz(50.0, 50.0, 0.0),
    ));

    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<&EffectSourceChip, With<ExplodeRequest>>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0, None,
        "empty chip_name should produce EffectSourceChip(None)"
    );
}

#[test]
fn evaluate_staged_effects_threads_chip_name_as_source_chip_to_fire_effect() {
    let mut app = evaluate_source_chip_test_app();

    let node = when_do(
        Trigger::Bump,
        EffectKind::Explode {
            range: 60.0,
            damage: 2.0,
        },
    );

    app.world_mut().spawn((
        StagedEffects(vec![("zapper".into(), node)]),
        Transform::from_xyz(50.0, 50.0, 0.0),
    ));

    app.update();
    app.update();

    let mut query = app.world_mut().query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip (on ExplodeRequest)"
    );
    assert_eq!(
        results[0].0,
        Some("zapper".to_string()),
        "evaluate_staged_effects should thread chip_name 'zapper' to fire_effect"
    );
}

#[test]
fn once_node_preserves_chip_name_through_to_fire_effect() {
    // Once([When(Bump, Do(Explode))]) in StagedEffects -- chip_name should
    // be threaded through Once dispatch to the inner fire_effect call.
    let mut app = evaluate_source_chip_test_app();

    let inner_when = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![EffectNode::Do(EffectKind::Explode {
            range: 60.0,
            damage: 2.0,
        })],
    };
    let once_node = EffectNode::Once(vec![inner_when]);

    app.world_mut().spawn((
        StagedEffects(vec![("once_chip".into(), once_node)]),
        Transform::from_xyz(50.0, 50.0, 0.0),
    ));

    app.update();
    app.update();

    let mut query = app.world_mut().query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0,
        Some("once_chip".to_string()),
        "Once node should preserve chip_name through to fire_effect"
    );
}
