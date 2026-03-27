use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    chips::{
        components::*,
        definition::{ChipDefinition, Rarity},
        inventory::ChipInventory,
        resources::ChipCatalog,
    },
    effect::definition::{Effect, EffectChains, EffectNode, ImpactTarget, RootEffect, Target, Trigger},
};

// =========================================================================
// Dispatch via RootEffect::On — passive Do children
// =========================================================================

#[test]
fn passive_piercing_via_root_effect() {
    let mut app = test_app();

    app.world_mut().spawn(Bolt);
    app.world_mut()
        .resource_mut::<ChipCatalog>()
        .insert(ChipDefinition {
            name: "Piercing Shot".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(Effect::Piercing(1))],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "Piercing Shot");
    tick(&mut app);

    let piercing = app
        .world_mut()
        .query::<&Piercing>()
        .iter(app.world())
        .next()
        .expect("bolt should have Piercing component after RootEffect::On(Bolt, [Do(Piercing)])");
    assert_eq!(piercing.0, 1);
}

#[test]
fn passive_multiple_do_leaves_fire_all() {
    let mut app = test_app();

    app.world_mut().spawn(Bolt);
    app.world_mut()
        .resource_mut::<ChipCatalog>()
        .insert(ChipDefinition {
            name: "MultiPassive".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![
                    EffectNode::Do(Effect::Piercing(1)),
                    EffectNode::Do(Effect::DamageBoost(0.5)),
                ],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "MultiPassive");
    tick(&mut app);

    assert!(
        app.world_mut()
            .query::<&Piercing>()
            .iter(app.world())
            .next()
            .is_some(),
        "bolt should have Piercing from On(Bolt, [Do(Piercing), Do(DamageBoost)])"
    );
    assert!(
        app.world_mut()
            .query::<&DamageBoost>()
            .iter(app.world())
            .next()
            .is_some(),
        "bolt should have DamageBoost from On(Bolt, [Do(Piercing), Do(DamageBoost)])"
    );
}

// =========================================================================
// Dispatch via RootEffect::On — triggered chain push to EffectChains
// =========================================================================

#[test]
fn triggered_chain_pushes_to_breaker_chains() {
    let mut app = test_app();

    let breaker = app
        .world_mut()
        .spawn((Breaker, EffectChains::default()))
        .id();
    app.world_mut()
        .resource_mut::<ChipCatalog>()
        .insert(ChipDefinition {
            name: "Surge".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Rare,
            max_stacks: 1,
            effects: vec![RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "Surge");
    tick(&mut app);

    let chains = app.world().get::<EffectChains>(breaker).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "triggered chain should be pushed to breaker entity EffectChains"
    );
    assert_eq!(chains.0[0].0, Some("Surge".to_owned()));
}

#[test]
fn triggered_chain_pushes_to_bolt_chains() {
    let mut app = test_app();

    let bolt = app.world_mut().spawn((Bolt, EffectChains::default())).id();
    app.world_mut()
        .resource_mut::<ChipCatalog>()
        .insert(ChipDefinition {
            name: "Impact Chip".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Rare,
            max_stacks: 1,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "Impact Chip");
    tick(&mut app);

    let chains = app.world().get::<EffectChains>(bolt).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "triggered chain should be pushed to bolt entity EffectChains"
    );
    assert_eq!(chains.0[0].0, Some("Impact Chip".to_owned()));
}

// =========================================================================
// Dispatch: mixed passive + triggered in separate RootEffects
// =========================================================================

#[test]
fn mixed_passive_and_triggered_in_separate_roots() {
    let mut app = test_app();

    app.world_mut().spawn(Bolt);
    let breaker = app
        .world_mut()
        .spawn((Breaker, EffectChains::default()))
        .id();
    app.world_mut()
        .resource_mut::<ChipCatalog>()
        .insert(ChipDefinition {
            name: "Hybrid".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Rare,
            max_stacks: 1,
            effects: vec![
                RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::Do(Effect::Piercing(1))],
                },
                RootEffect::On {
                    target: Target::Breaker,
                    then: vec![EffectNode::When {
                        trigger: Trigger::PerfectBump,
                        then: vec![EffectNode::Do(Effect::SpawnBolts {
                            count: 1,
                            lifespan: None,
                            inherit: false,
                        })],
                    }],
                },
            ],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "Hybrid");
    tick(&mut app);

    // Bolt should have Piercing(1) from the passive Do
    let piercing = app
        .world_mut()
        .query::<&Piercing>()
        .iter(app.world())
        .next()
        .expect("bolt should have Piercing from On(Bolt, [Do(Piercing)])");
    assert_eq!(piercing.0, 1);

    // Breaker EffectChains should have the triggered chain
    let chains = app.world().get::<EffectChains>(breaker).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "breaker should have 1 triggered chain from On(Breaker, [When(PerfectBump, ...)])"
    );
}

// =========================================================================
// Dispatch: bare Do fires passive, not pushed to chains
// =========================================================================

#[test]
fn bare_do_fires_passive_not_pushed_to_chains() {
    let mut app = test_app();

    app.world_mut().spawn(Bolt);
    let breaker = app
        .world_mut()
        .spawn((Breaker, EffectChains::default()))
        .id();
    app.world_mut()
        .resource_mut::<ChipCatalog>()
        .insert(ChipDefinition {
            name: "BareLeaf".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(Effect::Piercing(1))],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "BareLeaf");
    tick(&mut app);

    // Bolt should have Piercing(1)
    let piercing = app
        .world_mut()
        .query::<&Piercing>()
        .iter(app.world())
        .next()
        .expect("bolt should have Piercing from bare Do passive dispatch");
    assert_eq!(piercing.0, 1);

    // Breaker EffectChains should NOT have the leaf
    let chains = app.world().get::<EffectChains>(breaker).unwrap();
    assert!(
        chains.0.is_empty(),
        "bare Do leaf should fire passive, NOT be pushed to entity EffectChains"
    );
}

// =========================================================================
// Dispatch: inventory updated on selection
// =========================================================================

#[test]
fn inventory_updated_on_selection() {
    let mut app = test_app();
    app.init_resource::<ChipInventory>();

    app.world_mut().spawn(Bolt);
    app.world_mut()
        .resource_mut::<ChipCatalog>()
        .insert(ChipDefinition {
            name: "Piercing Shot".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(Effect::Piercing(1))],
            }],
            ingredients: None,
            template_name: None,
        });

    send_chip_selected(&mut app, "Piercing Shot");
    tick(&mut app);

    let inventory = app.world().resource::<ChipInventory>();
    assert_eq!(
        inventory.stacks("Piercing Shot"),
        1,
        "ChipInventory should track the selected chip at 1 stack"
    );
}

// =========================================================================
// H5: Same chip selected twice pushes two entries to EffectChains
// =========================================================================

#[test]
fn same_chip_selected_twice_pushes_two_entries() {
    let mut app = test_app();

    let breaker = app
        .world_mut()
        .spawn((Breaker, EffectChains::default()))
        .id();

    app.world_mut()
        .resource_mut::<ChipCatalog>()
        .insert(ChipDefinition {
            name: "Surge".to_owned(),
            description: "test".to_owned(),
            rarity: Rarity::Rare,
            max_stacks: 3,
            effects: vec![RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }],
            }],
            ingredients: None,
            template_name: None,
        });

    // First selection
    send_chip_selected(&mut app, "Surge");
    tick(&mut app);

    let chains = app.world().get::<EffectChains>(breaker).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "first selection should push 1 entry to breaker EffectChains"
    );

    // Second selection
    send_chip_selected(&mut app, "Surge");
    tick(&mut app);

    let chains = app.world().get::<EffectChains>(breaker).unwrap();
    assert_eq!(
        chains.0.len(),
        2,
        "second selection of same chip should push another entry — got {}",
        chains.0.len()
    );

    // Both entries should be When(PerfectBump) with chip name "Surge"
    for (i, (chip_name, node)) in chains.0.iter().enumerate() {
        assert_eq!(
            chip_name.as_deref(),
            Some("Surge"),
            "entry {i} chip_name should be Some(\"Surge\")"
        );
        assert!(
            matches!(
                node,
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    ..
                }
            ),
            "entry {i} should be When(PerfectBump)"
        );
    }
}
