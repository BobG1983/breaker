use bevy::prelude::*;

use super::*;
use crate::effect::{
    armed::ArmedEffects,
    definition::{Effect, EffectChains, EffectNode, EffectTarget, ImpactTarget, Trigger},
    triggers::test_helpers::{CapturedShockwaveFired, capture_shockwave_fired},
};

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

// =========================================================================
// H1a: evaluate_entity_chains preserves non-matching chains
// =========================================================================

/// System wrapper: evaluates entity chains for `Trigger::Bump` on all entities
/// with `EffectChains`.
fn sys_evaluate_bump(mut query: Query<&mut EffectChains>, mut commands: Commands) {
    for mut chains in &mut query {
        evaluate_entity_chains(&mut chains, Trigger::Bump, vec![], &mut commands);
    }
}

#[test]
fn evaluate_entity_chains_preserves_non_matching_chains() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<CapturedShockwaveFired>()
        .add_observer(capture_shockwave_fired)
        .add_systems(FixedUpdate, sys_evaluate_bump);

    // Entity with 2 chains: When(Bump) and When(Impact(Cell))
    let entity = app
        .world_mut()
        .spawn(EffectChains(vec![
            (
                None,
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                },
            ),
            (
                None,
                EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::test_shockwave(32.0))],
                },
            ),
        ]))
        .id();

    tick(&mut app);

    // ShockwaveFired(64.0) should fire for the Bump match
    let captured = app.world().resource::<CapturedShockwaveFired>();
    assert_eq!(
        captured.0.len(),
        1,
        "only the Bump chain should fire — got {}",
        captured.0.len()
    );
    assert!(
        (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
        "should fire shockwave with base_range 64.0"
    );

    // EffectChains is the permanent source of truth — BOTH chains are preserved
    // (matching chains fire but are NOT consumed; only Once wrappers are consumed)
    let chains = app.world().get::<EffectChains>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        2,
        "both chains should be retained (EffectChains is permanent) — got {}",
        chains.0.len()
    );
}

// =========================================================================
// H1b: evaluate_entity_chains unwraps Once(When) on matching trigger
// =========================================================================

fn sys_evaluate_bolt_lost(mut query: Query<&mut EffectChains>, mut commands: Commands) {
    for mut chains in &mut query {
        evaluate_entity_chains(&mut chains, Trigger::BoltLost, vec![], &mut commands);
    }
}

#[test]
fn evaluate_entity_chains_unwraps_once_on_matching_trigger() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<CapturedShockwaveFired>()
        .add_observer(capture_shockwave_fired)
        .add_systems(FixedUpdate, sys_evaluate_bolt_lost);

    // Entity with Once([When(BoltLost, [Do(Shockwave(64.0))])])
    let entity = app
        .world_mut()
        .spawn(EffectChains(vec![(
            None,
            EffectNode::Once(vec![EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }]),
        )]))
        .id();

    tick(&mut app);

    let captured = app.world().resource::<CapturedShockwaveFired>();
    assert_eq!(
        captured.0.len(),
        1,
        "ShockwaveFired should fire when Once(When(BoltLost)) matches BoltLost trigger"
    );
    assert!(
        (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
        "should fire shockwave with base_range 64.0"
    );

    // EffectChains should be empty (Once consumed)
    let chains = app.world().get::<EffectChains>(entity).unwrap();
    assert!(
        chains.0.is_empty(),
        "Once node should be consumed after matching — got {} entries",
        chains.0.len()
    );
}

// =========================================================================
// H1c: evaluate_entity_chains preserves Once(When) on non-matching trigger
// =========================================================================

#[test]
fn evaluate_entity_chains_preserves_once_on_non_matching_trigger() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<CapturedShockwaveFired>()
        .add_observer(capture_shockwave_fired)
        .add_systems(FixedUpdate, sys_evaluate_bump);

    // Entity with Once([When(BoltLost, [Do(Shockwave(64.0))])])
    let entity = app
        .world_mut()
        .spawn(EffectChains(vec![(
            None,
            EffectNode::Once(vec![EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }]),
        )]))
        .id();

    // sys_evaluate_bump evaluates against Trigger::Bump — should NOT match BoltLost
    tick(&mut app);

    let captured = app.world().resource::<CapturedShockwaveFired>();
    assert!(
        captured.0.is_empty(),
        "no ShockwaveFired should fire when trigger does not match Once contents"
    );

    let chains = app.world().get::<EffectChains>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "Once node should be preserved when trigger does not match — got {}",
        chains.0.len()
    );
}

// =========================================================================
// H1d: resolve_armed re-arms non-Do children
// =========================================================================

/// System wrapper: evaluates armed effects for `Trigger::Impact(Cell)` on all
/// entities with `ArmedEffects`.
fn sys_resolve_armed_impact_cell(
    mut query: Query<(Entity, &mut ArmedEffects)>,
    mut commands: Commands,
) {
    for (entity, mut armed) in &mut query {
        let targets = vec![EffectTarget::Entity(entity)];
        resolve_armed(
            &mut armed,
            Trigger::Impact(ImpactTarget::Cell),
            targets,
            &mut commands,
        );
    }
}

#[test]
fn resolve_armed_re_arms_non_do_children() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<CapturedShockwaveFired>()
        .add_observer(capture_shockwave_fired)
        .add_systems(FixedUpdate, sys_resolve_armed_impact_cell);

    // ArmedEffects with When(Impact(Cell), [When(CellDestroyed, [Do(Shockwave(64.0))])])
    let entity = app
        .world_mut()
        .spawn(ArmedEffects(vec![(
            None,
            EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::When {
                    trigger: Trigger::CellDestroyed,
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }],
            },
        )]))
        .id();

    tick(&mut app);

    // No ShockwaveFired — inner child is When, not Do
    let captured = app.world().resource::<CapturedShockwaveFired>();
    assert!(
        captured.0.is_empty(),
        "no ShockwaveFired should fire when inner child is When, not Do"
    );

    // ArmedEffects should have 1 re-armed entry: When(CellDestroyed, [Do(Shockwave(64.0))])
    let armed = app.world().get::<ArmedEffects>(entity).unwrap();
    assert_eq!(
        armed.0.len(),
        1,
        "should have 1 re-armed entry — got {}",
        armed.0.len()
    );
    assert!(
        matches!(
            &armed.0[0].1,
            EffectNode::When {
                trigger: Trigger::CellDestroyed,
                ..
            }
        ),
        "re-armed entry should be When(CellDestroyed, ...) — got {:?}",
        armed.0[0].1
    );
}

// =========================================================================
// H1e: resolve_armed retains non-matching entries
// =========================================================================

#[test]
fn resolve_armed_retains_non_matching_and_fires_matching() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<CapturedShockwaveFired>()
        .add_observer(capture_shockwave_fired)
        .add_systems(FixedUpdate, sys_resolve_armed_impact_cell);

    // ArmedEffects with 2 entries:
    // When(Impact(Cell), [Do(Shockwave(64.0))]) — matches
    // When(CellDestroyed, [Do(Shockwave(32.0))]) — does NOT match
    let entity = app
        .world_mut()
        .spawn(ArmedEffects(vec![
            (
                None,
                EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                },
            ),
            (
                None,
                EffectNode::When {
                    trigger: Trigger::CellDestroyed,
                    then: vec![EffectNode::Do(Effect::test_shockwave(32.0))],
                },
            ),
        ]))
        .id();

    tick(&mut app);

    // ShockwaveFired(64.0) should fire for the matching entry
    let captured = app.world().resource::<CapturedShockwaveFired>();
    assert_eq!(
        captured.0.len(),
        1,
        "only the Impact(Cell) match should fire — got {}",
        captured.0.len()
    );
    assert!(
        (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
        "should fire shockwave with base_range 64.0"
    );

    // ArmedEffects should retain the non-matching CellDestroyed entry
    let armed = app.world().get::<ArmedEffects>(entity).unwrap();
    assert_eq!(
        armed.0.len(),
        1,
        "non-matching CellDestroyed entry should be retained — got {}",
        armed.0.len()
    );
    assert!(
        matches!(
            &armed.0[0].1,
            EffectNode::When {
                trigger: Trigger::CellDestroyed,
                ..
            }
        ),
        "retained entry should be When(CellDestroyed, ...) — got {:?}",
        armed.0[0].1
    );
}
