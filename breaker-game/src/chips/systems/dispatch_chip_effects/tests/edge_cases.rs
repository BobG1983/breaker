use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    chips::{
        components::*,
        definition::{ChipDefinition, Rarity},
        inventory::ChipInventory,
        resources::ChipRegistry,
    },
    effect::{
        definition::{Effect, EffectChains, EffectNode, ImpactTarget, RootEffect, Target, Trigger},
        effects::*,
    },
    ui::messages::ChipSelected,
};

// =========================================================================
// Dispatch: unknown chip silently skipped
// =========================================================================

#[test]
fn unknown_chip_silently_skipped() {
    let mut app = test_app();
    app.init_resource::<ChipInventory>();

    app.world_mut().spawn(Bolt);
    app.world_mut().spawn(Breaker);

    send_chip_selected(&mut app, "Nonexistent");
    tick(&mut app);

    let inventory = app.world().resource::<ChipInventory>();
    assert_eq!(
        inventory.total_held(),
        0,
        "ChipInventory should remain empty for unknown chip"
    );
    assert!(
        app.world_mut()
            .query::<&Piercing>()
            .iter(app.world())
            .next()
            .is_none(),
        "no passive components should exist for unknown chip"
    );
}

// =========================================================================
// Dispatch: triggered chip adds no passive components
// =========================================================================

#[test]
fn triggered_chip_adds_no_passive_components() {
    let mut app = test_app();

    app.world_mut().spawn(Bolt);
    let breaker = app
        .world_mut()
        .spawn((Breaker, EffectChains::default()))
        .id();
    app.world_mut()
        .resource_mut::<ChipRegistry>()
        .insert(ChipDefinition {
            name: "Surge".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Rare,
            max_stacks: 1,
            effects: vec![RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(Effect::SpawnBolts {
                        count: 1,
                        lifespan: None,
                        inherit: false,
                    })],
                }],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "Surge");
    tick(&mut app);

    // No passive components on bolt or breaker
    assert!(
        app.world_mut()
            .query::<&Piercing>()
            .iter(app.world())
            .next()
            .is_none(),
        "triggered chip should not insert Piercing"
    );
    assert!(
        app.world_mut()
            .query::<&DamageBoost>()
            .iter(app.world())
            .next()
            .is_none(),
        "triggered chip should not insert DamageBoost"
    );
    assert!(
        app.world_mut()
            .query::<&BoltSpeedBoost>()
            .iter(app.world())
            .next()
            .is_none(),
        "triggered chip should not insert BoltSpeedBoost"
    );

    // But breaker EffectChains should have the triggered chain
    let chains = app.world().get::<EffectChains>(breaker).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "triggered chain should be in breaker EffectChains"
    );
}

// =========================================================================
// Dispatch: Once and Until nodes push to chains
// =========================================================================

#[test]
fn once_node_pushes_to_chains() {
    let mut app = test_app();

    let bolt = app.world_mut().spawn((Bolt, EffectChains::default())).id();
    app.world_mut()
        .resource_mut::<ChipRegistry>()
        .insert(ChipDefinition {
            name: "OnceChip".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Rare,
            max_stacks: 1,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Once(vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::DamageBoost(1.5))],
                }])],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "OnceChip");
    tick(&mut app);

    let chains = app.world().get::<EffectChains>(bolt).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "Once node should be pushed to bolt EffectChains"
    );
    assert!(
        matches!(&chains.0[0].1, EffectNode::Once(_)),
        "pushed chain should be a Once node"
    );
}

#[test]
fn until_node_pushes_to_chains() {
    let mut app = test_app();

    let bolt = app.world_mut().spawn((Bolt, EffectChains::default())).id();
    app.world_mut()
        .resource_mut::<ChipRegistry>()
        .insert(ChipDefinition {
            name: "UntilChip".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Rare,
            max_stacks: 1,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Until {
                    until: Trigger::BoltLost,
                    then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
                }],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "UntilChip");
    tick(&mut app);

    let chains = app.world().get::<EffectChains>(bolt).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "Until node should be pushed to bolt EffectChains"
    );
    assert!(
        matches!(&chains.0[0].1, EffectNode::Until { .. }),
        "pushed chain should be an Until node"
    );
}

// =========================================================================
// Dispatch: unhandled target logs warning and does not panic
// =========================================================================

#[test]
fn unhandled_target_logs_and_does_not_panic() {
    let mut app = test_app();

    let bolt = app.world_mut().spawn((Bolt, EffectChains::default())).id();
    let breaker = app
        .world_mut()
        .spawn((Breaker, EffectChains::default()))
        .id();
    app.world_mut()
        .resource_mut::<ChipRegistry>()
        .insert(ChipDefinition {
            name: "AllBoltsChip".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 1,
            effects: vec![RootEffect::On {
                target: Target::AllBolts,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(Effect::Piercing(1))],
                }],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "AllBoltsChip");
    tick(&mut app);

    // No Piercing component should be on any entity
    assert!(
        app.world_mut()
            .query::<&Piercing>()
            .iter(app.world())
            .next()
            .is_none(),
        "unhandled target should not insert Piercing on any entity"
    );

    // No EffectChains entries should have been pushed
    let bolt_chains = app.world().get::<EffectChains>(bolt).unwrap();
    assert!(
        bolt_chains.0.is_empty(),
        "unhandled target should not push chains to bolt EffectChains"
    );
    let breaker_chains = app.world().get::<EffectChains>(breaker).unwrap();
    assert!(
        breaker_chains.0.is_empty(),
        "unhandled target should not push chains to breaker EffectChains"
    );
}

// =========================================================================
// H6: Passive dispatch fires correct typed events for all passive effect types
// =========================================================================

#[test]
fn passive_chain_hit_fires_and_applies_component() {
    let mut app = test_app();

    let bolt = app.world_mut().spawn(Bolt).id();
    app.world_mut()
        .resource_mut::<ChipRegistry>()
        .insert(ChipDefinition {
            name: "Chain Hit".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(Effect::ChainHit(1))],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "Chain Hit");
    tick(&mut app);

    let chain_hit = app
        .world()
        .entity(bolt)
        .get::<ChainHit>()
        .expect("bolt should have ChainHit component after passive ChainHit dispatch");
    assert_eq!(
        chain_hit.0, 1,
        "ChainHit should be 1 per stack — got {}",
        chain_hit.0
    );
}

#[test]
fn passive_bump_force_fires_and_applies_component() {
    let mut app = test_app();

    let breaker = app.world_mut().spawn(Breaker).id();
    app.world_mut()
        .resource_mut::<ChipRegistry>()
        .insert(ChipDefinition {
            name: "Bump Force".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(Effect::BumpForce(10.0))],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "Bump Force");
    tick(&mut app);

    let bump_force = app
        .world()
        .entity(breaker)
        .get::<BumpForceBoost>()
        .expect("breaker should have BumpForceBoost after passive BumpForce dispatch");
    assert!(
        (bump_force.0 - 10.0).abs() < f32::EPSILON,
        "BumpForceBoost should be 10.0 — got {}",
        bump_force.0
    );
}

#[test]
fn passive_tilt_control_fires_and_applies_component() {
    let mut app = test_app();

    let breaker = app.world_mut().spawn(Breaker).id();
    app.world_mut()
        .resource_mut::<ChipRegistry>()
        .insert(ChipDefinition {
            name: "Tilt Control".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(Effect::TiltControl(0.1))],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "Tilt Control");
    tick(&mut app);

    let tilt = app
        .world()
        .entity(breaker)
        .get::<TiltControlBoost>()
        .expect("breaker should have TiltControlBoost after passive TiltControl dispatch");
    assert!(
        (tilt.0 - 0.1).abs() < f32::EPSILON,
        "TiltControlBoost should be 0.1 — got {}",
        tilt.0
    );
}

#[test]
fn passive_ramping_damage_fires_and_applies_component() {
    use crate::effect::effects::ramping_damage::{RampingDamageState, handle_ramping_damage};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<ChipSelected>()
        .init_resource::<ChipRegistry>()
        .add_observer(handle_ramping_damage)
        .add_systems(
            Update,
            (enqueue_chip_selected, super::super::dispatch_chip_effects).chain(),
        );

    let bolt = app.world_mut().spawn(Bolt).id();
    app.world_mut()
        .resource_mut::<ChipRegistry>()
        .insert(ChipDefinition {
            name: "Ramping".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Uncommon,
            max_stacks: 2,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(Effect::RampingDamage {
                    bonus_per_hit: 0.02,
                })],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "Ramping");
    tick(&mut app);

    let state = app
        .world()
        .entity(bolt)
        .get::<RampingDamageState>()
        .expect("bolt should have RampingDamageState after passive RampingDamage dispatch");
    assert!(
        (state.bonus_per_hit - 0.02).abs() < f32::EPSILON,
        "bonus_per_hit should be 0.02 — got {}",
        state.bonus_per_hit
    );
    assert!(
        (state.current_bonus - 0.0).abs() < f32::EPSILON,
        "current_bonus should start at 0.0 — got {}",
        state.current_bonus
    );
}
