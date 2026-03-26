//! Thin dispatcher: reads [`ChipSelected`] messages, looks up the chip in the
//! [`ChipRegistry`], and dispatches effects via `RootEffect::On` target routing.

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    chips::{inventory::ChipInventory, resources::ChipRegistry},
    effect::{
        definition::{EffectChains, EffectNode, RootEffect, Target},
        typed_events::fire_passive_event,
    },
    ui::messages::ChipSelected,
};

/// Reads [`ChipSelected`] messages, looks up the chip definition in the
/// [`ChipRegistry`], and dispatches effects via `RootEffect::On` target routing.
///
/// For each `On { target, then }` node in the chip's effects:
/// - `Do(eff)` children fire as passive events immediately
/// - `When`/`Once`/`Until`/`On` children push to the target entity's `EffectChains`
pub(crate) fn dispatch_chip_effects(
    mut reader: MessageReader<ChipSelected>,
    registry: Option<Res<ChipRegistry>>,
    mut inventory: Option<ResMut<ChipInventory>>,
    mut breaker_query: Query<&mut EffectChains, With<Breaker>>,
    mut bolt_query: Query<&mut EffectChains, (With<Bolt>, Without<Breaker>)>,
    mut commands: Commands,
) {
    let Some(registry) = registry else {
        return;
    };
    for msg in reader.read() {
        let Some(chip) = registry.get(&msg.name) else {
            debug!("chip not found in registry: {}", msg.name);
            continue;
        };
        if let Some(inv) = inventory.as_mut() {
            let _ = inv.add_chip(&msg.name, chip);
        }
        for root in &chip.effects {
            let RootEffect::On { target, then } = root;
            for child in then {
                match child {
                    EffectNode::Do(eff) => {
                        fire_passive_event(
                            eff.clone(),
                            chip.max_stacks,
                            msg.name.clone(),
                            &mut commands,
                        );
                    }
                    node => {
                        let entry = (Some(msg.name.clone()), node.clone());
                        match target {
                            Target::Bolt => {
                                for mut chains in &mut bolt_query {
                                    chains.0.push(entry.clone());
                                }
                            }
                            Target::Breaker => {
                                for mut chains in &mut breaker_query {
                                    chains.0.push(entry.clone());
                                }
                            }
                            _ => {
                                warn!("dispatch_chip_effects: unhandled target {:?}", target);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
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
            definition::{
                Effect, EffectChains, EffectNode, ImpactTarget, RootEffect, Target, Trigger,
            },
            effects::*,
        },
        ui::messages::ChipSelected,
    };

    // ---------------------------------------------------------------------------
    // Test infrastructure
    // ---------------------------------------------------------------------------

    /// Resource holding an optional [`ChipSelected`] message to be sent once.
    #[derive(Resource)]
    struct PendingChipSelected(Option<ChipSelected>);

    /// Helper system: writes the pending message once, then clears it.
    fn enqueue_chip_selected(
        mut pending: ResMut<PendingChipSelected>,
        mut writer: MessageWriter<ChipSelected>,
    ) {
        if let Some(msg) = pending.0.take() {
            writer.write(msg);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ChipSelected>()
            .init_resource::<ChipRegistry>()
            .add_systems(
                Update,
                (enqueue_chip_selected, dispatch_chip_effects).chain(),
            )
            .add_observer(handle_piercing)
            .add_observer(handle_damage_boost)
            .add_observer(handle_bolt_speed_boost)
            .add_observer(handle_chain_hit)
            .add_observer(handle_bolt_size_boost)
            .add_observer(handle_width_boost)
            .add_observer(handle_breaker_speed_boost)
            .add_observer(handle_bump_force_boost)
            .add_observer(handle_tilt_control_boost);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn send_chip_selected(app: &mut App, name: &str) {
        app.insert_resource(PendingChipSelected(Some(ChipSelected {
            name: name.to_owned(),
        })));
    }

    // =========================================================================
    // Dispatch via RootEffect::On — passive Do children
    // =========================================================================

    #[test]
    fn passive_piercing_via_root_effect() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
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
            .expect(
                "bolt should have Piercing component after RootEffect::On(Bolt, [Do(Piercing)])",
            );
        assert_eq!(piercing.0, 1);
    }

    #[test]
    fn passive_multiple_do_leaves_fire_all() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
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
            .resource_mut::<ChipRegistry>()
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
            .resource_mut::<ChipRegistry>()
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
            .resource_mut::<ChipRegistry>()
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
            .resource_mut::<ChipRegistry>()
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
            .resource_mut::<ChipRegistry>()
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
        use crate::effect::effects::ramping_damage::{
            handle_ramping_damage, RampingDamageState,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ChipSelected>()
            .init_resource::<ChipRegistry>()
            .add_observer(handle_ramping_damage)
            .add_systems(
                Update,
                (enqueue_chip_selected, dispatch_chip_effects).chain(),
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
}
