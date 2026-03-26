use bevy::prelude::*;

use super::super::*;

// =========================================================================
// C7 Wave 1 Part D: EffectChains component (behaviors 23-25)
// =========================================================================

#[test]
fn effect_chains_default_is_empty() {
    let chains = EffectChains::default();
    assert!(chains.0.is_empty());
}

#[test]
fn effect_chains_stores_mixed_node_types() {
    let chains = EffectChains(vec![
        (
            None,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(Effect::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                })],
            },
        ),
        (None, EffectNode::Do(Effect::Piercing(1))),
    ]);
    assert_eq!(chains.0.len(), 2);
}

#[test]
fn effect_chains_single_do_is_valid() {
    let chains = EffectChains(vec![(None, EffectNode::Do(Effect::Piercing(1)))]);
    assert_eq!(chains.0.len(), 1, "single Do node in chains is valid");
}

#[test]
fn effect_chains_is_queryable_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(EffectChains::default()).id();
    let found = app
        .world()
        .entity(entity)
        .get::<EffectChains>()
        .expect("EffectChains should be queryable as Component");
    assert!(found.0.is_empty());
}

#[test]
fn effect_chains_not_present_returns_none() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn_empty().id();
    assert!(
        app.world().entity(entity).get::<EffectChains>().is_none(),
        "entity without EffectChains should return None"
    );
}
