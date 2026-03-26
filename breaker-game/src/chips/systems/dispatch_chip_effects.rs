//! Thin dispatcher: reads [`ChipSelected`] messages, looks up the chip in the
//! [`ChipRegistry`], and fires typed passive events for per-effect observers.

use bevy::prelude::*;
use tracing::debug;

use crate::{
    chips::{inventory::ChipInventory, resources::ChipRegistry},
    effect::{
        ActiveEffects,
        definition::{EffectNode, Trigger},
        typed_events::fire_passive_event,
    },
    ui::messages::ChipSelected,
};

/// Reads [`ChipSelected`] messages, looks up the chip definition in the
/// [`ChipRegistry`], and fires typed passive events for each selected chip.
///
/// Per-effect observers handle the actual stacking logic.
pub(crate) fn dispatch_chip_effects(
    mut reader: MessageReader<ChipSelected>,
    registry: Option<Res<ChipRegistry>>,
    mut inventory: Option<ResMut<ChipInventory>>,
    mut active_chains: Option<ResMut<ActiveEffects>>,
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
        if let Some(ref mut inv) = inventory {
            let _ = inv.add_chip(&msg.name, chip);
        }
        for effect in &chip.effects {
            match effect {
                EffectNode::When {
                    trigger: Trigger::OnSelected,
                    then,
                } => {
                    for child in then {
                        if let EffectNode::Do(eff) = child {
                            fire_passive_event(
                                eff.clone(),
                                chip.max_stacks,
                                msg.name.clone(),
                                &mut commands,
                            );
                        }
                    }
                }
                EffectNode::Do(eff) => {
                    fire_passive_event(
                        eff.clone(),
                        chip.max_stacks,
                        msg.name.clone(),
                        &mut commands,
                    );
                }
                // Any trigger-wrapper variant (When with non-OnSelected trigger, Until, Once)
                // is pushed to ActiveEffects for runtime evaluation by bridge systems.
                node => {
                    if let Some(ref mut active) = active_chains {
                        active.0.push((Some(msg.name.clone()), node.clone()));
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
            ActiveEffects,
            definition::{Effect, EffectNode, ImpactTarget, Target, Trigger},
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
            .init_resource::<ActiveEffects>()
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

    // ---------------------------------------------------------------------------
    // B3: OnSelected effects fire ChipEffectApplied for each inner leaf (29)
    // ---------------------------------------------------------------------------

    #[test]
    fn on_selected_piercing_inserts_on_bolt() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "Piercing Shot".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Common,
                max_stacks: 3,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnSelected,
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
            .expect("bolt should have Piercing component after OnSelected chip selected");
        assert_eq!(piercing.0, 1);
    }

    #[test]
    fn on_selected_multiple_leaves_fires_all() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "MultiEffect".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Common,
                max_stacks: 3,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnSelected,
                    then: vec![
                        EffectNode::Do(Effect::Piercing(1)),
                        EffectNode::Do(Effect::DamageBoost(0.5)),
                    ],
                }],
                ingredients: None,
                template_name: None,
            });

        send_chip_selected(&mut app, "MultiEffect");
        tick(&mut app);

        assert!(
            app.world_mut()
                .query::<&Piercing>()
                .iter(app.world())
                .next()
                .is_some(),
            "bolt should have Piercing from OnSelected with multiple leaves"
        );
        assert!(
            app.world_mut()
                .query::<&DamageBoost>()
                .iter(app.world())
                .next()
                .is_some(),
            "bolt should have DamageBoost from OnSelected with multiple leaves"
        );
    }

    // ---------------------------------------------------------------------------
    // B3: Triggered chain pushes to ActiveEffects (30)
    // ---------------------------------------------------------------------------

    #[test]
    fn triggered_chain_pushes_to_active_chains() {
        let mut app = test_app();

        let chain = EffectNode::When {
            trigger: Trigger::OnPerfectBump,
            then: vec![EffectNode::When {
                trigger: Trigger::OnImpact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::Shockwave {
                    base_range: 64.0,
                    range_per_level: 32.0,
                    stacks: 1,
                    speed: 400.0,
                })],
            }],
        };
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "Surge".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Rare,
                max_stacks: 1,
                effects: vec![chain.clone()],
                ingredients: None,
                template_name: None,
            });

        send_chip_selected(&mut app, "Surge");
        tick(&mut app);

        let active = app.world().resource::<ActiveEffects>();
        assert_eq!(
            active.0.len(),
            1,
            "triggered chain should be pushed to ActiveEffects"
        );
        assert_eq!(active.0[0].0, Some("Surge".to_owned()));
        assert_eq!(active.0[0].1, chain);
    }

    // ---------------------------------------------------------------------------
    // B3: dispatch_chip_effects updates ChipInventory (32)
    // ---------------------------------------------------------------------------

    #[test]
    fn dispatch_chip_effects_adds_chip_to_inventory_on_chip_selected() {
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
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnSelected,
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

    #[test]
    fn dispatch_chip_effects_does_not_add_inventory_entry_for_unknown_chip() {
        let mut app = test_app();
        app.init_resource::<ChipInventory>();

        send_chip_selected(&mut app, "Nonexistent");
        tick(&mut app);

        let inventory = app.world().resource::<ChipInventory>();
        assert_eq!(
            inventory.total_held(),
            0,
            "ChipInventory should remain empty for unknown chip"
        );
    }

    // ---------------------------------------------------------------------------
    // B3: Triggered chip adds no passive components (33)
    // ---------------------------------------------------------------------------

    #[test]
    fn triggered_chip_adds_no_passive_components() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut().spawn(Breaker);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "Surge".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Rare,
                max_stacks: 1,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnPerfectBump,
                    then: vec![EffectNode::Do(Effect::SpawnBolts {
                        count: 1,
                        lifespan: None,
                        inherit: false,
                    })],
                }],
                ingredients: None,
                template_name: None,
            });

        send_chip_selected(&mut app, "Surge");
        tick(&mut app);

        assert!(
            app.world_mut()
                .query::<&Piercing>()
                .iter(app.world())
                .next()
                .is_none()
        );
        assert!(
            app.world_mut()
                .query::<&DamageBoost>()
                .iter(app.world())
                .next()
                .is_none()
        );
        assert!(
            app.world_mut()
                .query::<&BoltSpeedBoost>()
                .iter(app.world())
                .next()
                .is_none()
        );
        assert!(
            app.world_mut()
                .query::<&ChainHit>()
                .iter(app.world())
                .next()
                .is_none()
        );
        assert!(
            app.world_mut()
                .query::<&BoltSizeBoost>()
                .iter(app.world())
                .next()
                .is_none()
        );
        assert!(
            app.world_mut()
                .query::<&WidthBoost>()
                .iter(app.world())
                .next()
                .is_none()
        );
        assert!(
            app.world_mut()
                .query::<&BreakerSpeedBoost>()
                .iter(app.world())
                .next()
                .is_none()
        );
        assert!(
            app.world_mut()
                .query::<&BumpForceBoost>()
                .iter(app.world())
                .next()
                .is_none()
        );
        assert!(
            app.world_mut()
                .query::<&TiltControlBoost>()
                .iter(app.world())
                .next()
                .is_none()
        );
    }

    // ---------------------------------------------------------------------------
    // B3: Bare leaf at top level is treated as immediate passive (34)
    // ---------------------------------------------------------------------------

    #[test]
    fn bare_leaf_fires_chip_effect_applied_not_active_chains() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "BareLeaf",
                EffectNode::Do(Effect::Piercing(1)),
                3,
            ));

        send_chip_selected(&mut app, "BareLeaf");
        tick(&mut app);

        // Bolt should have Piercing(1) from direct ChipEffectApplied
        let piercing = app
            .world_mut()
            .query::<&Piercing>()
            .iter(app.world())
            .next()
            .expect("bolt should have Piercing from bare leaf ChipEffectApplied");
        assert_eq!(piercing.0, 1);

        // ActiveEffects should NOT have the leaf
        let active = app.world().resource::<ActiveEffects>();
        assert!(
            active.0.is_empty(),
            "bare leaf should NOT be pushed to ActiveEffects"
        );
    }

    // ---------------------------------------------------------------------------
    // B3: OnSelected integration via registry (36)
    // ---------------------------------------------------------------------------

    #[test]
    fn on_selected_via_registry_inserts_piercing() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "Piercing Shot".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Common,
                max_stacks: 3,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnSelected,
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
            .expect("bolt should have Piercing via OnSelected from registry");
        assert_eq!(piercing.0, 1);
    }

    // ---------------------------------------------------------------------------
    // B3: OnSelected with empty inner vec is a no-op (37)
    // ---------------------------------------------------------------------------

    #[test]
    fn on_selected_empty_vec_is_noop() {
        let mut app = test_app();
        app.init_resource::<ChipInventory>();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "EmptyPassive".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Common,
                max_stacks: 1,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnSelected,
                    then: vec![],
                }],
                ingredients: None,
                template_name: None,
            });

        send_chip_selected(&mut app, "EmptyPassive");
        tick(&mut app);

        // No components inserted
        assert!(
            app.world_mut()
                .query::<&Piercing>()
                .iter(app.world())
                .next()
                .is_none()
        );
        // But inventory still tracks it
        let inventory = app.world().resource::<ChipInventory>();
        assert_eq!(inventory.stacks("EmptyPassive"), 1);
    }

    // ---------------------------------------------------------------------------
    // B3: Mixed effects: OnSelected + triggered chain on same chip (38)
    // ---------------------------------------------------------------------------

    #[test]
    fn mixed_effects_on_selected_and_triggered() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "Hybrid".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Rare,
                max_stacks: 1,
                effects: vec![
                    EffectNode::When {
                        trigger: Trigger::OnSelected,
                        then: vec![EffectNode::Do(Effect::Piercing(1))],
                    },
                    EffectNode::When {
                        trigger: Trigger::OnPerfectBump,
                        then: vec![EffectNode::Do(Effect::SpawnBolts {
                            count: 1,
                            lifespan: None,
                            inherit: false,
                        })],
                    },
                ],
                ingredients: None,
                template_name: None,
            });

        send_chip_selected(&mut app, "Hybrid");
        tick(&mut app);

        // Piercing from OnSelected dispatch
        let piercing = app
            .world_mut()
            .query::<&Piercing>()
            .iter(app.world())
            .next()
            .expect("bolt should have Piercing from OnSelected dispatch");
        assert_eq!(piercing.0, 1);

        // ActiveEffects has the triggered chain
        let active = app.world().resource::<ActiveEffects>();
        assert_eq!(
            active.0.len(),
            1,
            "triggered chain should be pushed to ActiveEffects"
        );
        assert_eq!(active.0[0].0, Some("Hybrid".to_owned()));
        assert_eq!(
            active.0[0].1,
            EffectNode::When {
                trigger: Trigger::OnPerfectBump,
                then: vec![EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false
                })]
            }
        );
    }

    // ---------------------------------------------------------------------------
    // B3: SpeedBoost Target::AllBolts via ChipEffectApplied is silent no-op (39)
    // ---------------------------------------------------------------------------

    #[test]
    fn speed_boost_all_bolts_via_on_selected_is_silent_noop() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut().spawn(Breaker);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "AllBoltsSpeed".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Common,
                max_stacks: 3,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnSelected,
                    then: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: Target::AllBolts,
                        multiplier: 1.1,
                    })],
                }],
                ingredients: None,
                template_name: None,
            });

        send_chip_selected(&mut app, "AllBoltsSpeed");
        tick(&mut app);

        assert!(
            app.world_mut()
                .query::<&BoltSpeedBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "no BoltSpeedBoost should exist for AllBolts passive"
        );
        assert!(
            app.world_mut()
                .query::<&BreakerSpeedBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "no BreakerSpeedBoost should exist for AllBolts passive"
        );
    }

    // ---------------------------------------------------------------------------
    // Existing integration tests rewritten with new types
    // ---------------------------------------------------------------------------

    #[test]
    fn piercing_stacks_from_1_to_2_via_bare_leaf() {
        let mut app = test_app();

        let bolt = app.world_mut().spawn((Bolt, Piercing(1))).id();
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Piercing Shot",
                EffectNode::Do(Effect::Piercing(1)),
                3,
            ));

        send_chip_selected(&mut app, "Piercing Shot");
        tick(&mut app);

        let piercing = app
            .world()
            .entity(bolt)
            .get::<Piercing>()
            .expect("bolt should still have Piercing component");
        assert_eq!(piercing.0, 2, "Piercing should stack from 1 to 2");
    }

    #[test]
    fn piercing_respects_max_stacks_cap() {
        let mut app = test_app();

        let bolt = app.world_mut().spawn((Bolt, Piercing(3))).id();
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Piercing Shot",
                EffectNode::Do(Effect::Piercing(1)),
                3,
            ));

        send_chip_selected(&mut app, "Piercing Shot");
        tick(&mut app);

        let piercing = app
            .world()
            .entity(bolt)
            .get::<Piercing>()
            .expect("bolt should still have Piercing component");
        assert_eq!(piercing.0, 3, "Piercing should not exceed max_stacks=3 cap");
    }

    #[test]
    fn width_boost_inserts_on_breaker_via_on_selected_size_boost() {
        let mut app = test_app();

        app.world_mut().spawn(Breaker);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "Wide Breaker".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Common,
                max_stacks: 3,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnSelected,
                    then: vec![EffectNode::Do(Effect::SizeBoost(Target::Breaker, 20.0))],
                }],
                ingredients: None,
                template_name: None,
            });

        send_chip_selected(&mut app, "Wide Breaker");
        tick(&mut app);

        let wb = app
            .world_mut()
            .query::<&WidthBoost>()
            .iter(app.world())
            .next()
            .expect("breaker should have WidthBoost component after OnSelected SizeBoost(Breaker)");
        assert!(
            (wb.0 - 20.0).abs() < f32::EPSILON,
            "WidthBoost should be 20.0, got {}",
            wb.0
        );
    }

    #[test]
    fn no_components_added_for_unknown_chip_name() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut().spawn(Breaker);

        send_chip_selected(&mut app, "Nonexistent");
        tick(&mut app);

        assert!(
            app.world_mut()
                .query::<&Piercing>()
                .iter(app.world())
                .next()
                .is_none(),
            "no Piercing should exist for unknown chip"
        );
        assert!(
            app.world_mut()
                .query::<&DamageBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "no DamageBoost should exist for unknown chip"
        );
    }

    // =========================================================================
    // B12b: dispatch_chip_effects dispatch patterns with EffectNode (behaviors 25-26)
    // These tests verify the EffectNode dispatch logic that dispatch_chip_effects
    // will use after migration. They exercise evaluate_node which fails
    // with todo!().
    // =========================================================================

    #[test]
    fn effect_node_on_selected_dispatches_leaf_effects() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        // dispatch_chip_effects matches on
        // EffectNode::When { trigger: OnSelected, then } and fires
        // passive events for each inner Do's Effect.
        let node = EffectNode::When {
            trigger: Trigger::OnSelected,
            then: vec![EffectNode::Do(Effect::Piercing(1))],
        };
        // Verify EffectNode structure and inner extraction
        match &node {
            EffectNode::When {
                trigger: Trigger::OnSelected,
                then,
            } => {
                assert_eq!(then.len(), 1);
                match &then[0] {
                    EffectNode::Do(effect) => {
                        assert_eq!(*effect, Effect::Piercing(1));
                    }
                    other => panic!("expected Do, got {other:?}"),
                }
            }
            other => panic!("expected When(OnSelected, _), got {other:?}"),
        }
        // evaluate_node should return NoMatch for OnSelected — it's handled
        // by dispatch_chip_effects, not by bridges
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(result, vec![NodeEvalResult::NoMatch]);
    }

    #[test]
    fn effect_node_on_selected_multiple_leaves_extracts_all() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        let node = EffectNode::When {
            trigger: Trigger::OnSelected,
            then: vec![
                EffectNode::Do(Effect::Piercing(1)),
                EffectNode::Do(Effect::DamageBoost(0.5)),
            ],
        };
        match &node {
            EffectNode::When {
                trigger: Trigger::OnSelected,
                then,
            } => {
                assert_eq!(then.len(), 2);
                assert_eq!(then[0], EffectNode::Do(Effect::Piercing(1)));
                assert_eq!(then[1], EffectNode::Do(Effect::DamageBoost(0.5)));
            }
            other => panic!("expected When(OnSelected, 2 children), got {other:?}"),
        }
        // OnSelected always returns NoMatch from evaluate_node (fails with todo!)
        let result = evaluate_node(TriggerKind::BumpSuccess, &node);
        assert_eq!(result, vec![NodeEvalResult::NoMatch]);
    }

    #[test]
    fn effect_node_trigger_wrapper_pushed_to_active_effects_pattern() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        // After migration, dispatch_chip_effects pushes non-OnSelected triggers
        // to ActiveEffects. Verify this EffectNode evaluates as expected.
        let node = EffectNode::When {
            trigger: Trigger::OnPerfectBump,
            then: vec![EffectNode::Do(Effect::SpawnBolts {
                count: 1,
                lifespan: None,
                inherit: false,
            })],
        };
        // Should NOT match OnSelected — bridge evaluation handles it
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::SpawnBolts {
                count: 1,
                lifespan: None,
                inherit: false
            })],
            "OnPerfectBump with Do(SpawnBolts) should fire on PerfectBump"
        );
    }

    #[test]
    fn effect_node_bare_leaf_pattern_for_direct_dispatch() {
        use crate::effect::{
            definition::{Effect, EffectNode},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        // A bare EffectNode::Do is treated as an immediate
        // passive — dispatched via fire_passive_event directly.
        let node = EffectNode::Do(Effect::Piercing(1));
        assert!(
            matches!(&node, EffectNode::Do(_)),
            "bare Do should be directly dispatchable"
        );
        // Verify it extracts correctly
        if let EffectNode::Do(effect) = &node {
            assert_eq!(*effect, Effect::Piercing(1));
        }
        // Bare leaf returns NoMatch from evaluate_node (fails with todo!)
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(result, vec![NodeEvalResult::NoMatch]);
    }

    // =========================================================================
    // B12c: dispatch fires typed passive events (behaviors 18-20)
    // =========================================================================

    #[derive(Resource, Default)]
    struct CapturedPiercingApplied(Vec<crate::effect::typed_events::PiercingApplied>);

    fn capture_piercing_applied(
        trigger: On<crate::effect::typed_events::PiercingApplied>,
        mut captured: ResMut<CapturedPiercingApplied>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedSizeBoostApplied(Vec<crate::effect::typed_events::SizeBoostApplied>);

    fn capture_size_boost_applied(
        trigger: On<crate::effect::typed_events::SizeBoostApplied>,
        mut captured: ResMut<CapturedSizeBoostApplied>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedSpeedBoostApplied(Vec<crate::effect::typed_events::SpeedBoostApplied>);

    fn capture_speed_boost_applied(
        trigger: On<crate::effect::typed_events::SpeedBoostApplied>,
        mut captured: ResMut<CapturedSpeedBoostApplied>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    fn typed_dispatch_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ChipSelected>()
            .init_resource::<ChipRegistry>()
            .init_resource::<ActiveEffects>()
            .init_resource::<CapturedPiercingApplied>()
            .init_resource::<CapturedSizeBoostApplied>()
            .init_resource::<CapturedSpeedBoostApplied>()
            .add_observer(capture_piercing_applied)
            .add_observer(capture_size_boost_applied)
            .add_observer(capture_speed_boost_applied)
            .add_systems(
                Update,
                (enqueue_chip_selected, dispatch_chip_effects).chain(),
            );
        app
    }

    #[test]
    fn dispatch_fires_piercing_applied_for_on_selected_piercing() {
        let mut app = typed_dispatch_test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "Piercing Shot".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Common,
                max_stacks: 3,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnSelected,
                    then: vec![EffectNode::Do(Effect::Piercing(1))],
                }],
                ingredients: None,
                template_name: None,
            });

        send_chip_selected(&mut app, "Piercing Shot");
        tick(&mut app);

        let captured = app.world().resource::<CapturedPiercingApplied>();
        assert_eq!(
            captured.0.len(),
            1,
            "dispatch should fire PiercingApplied (not ChipEffectApplied) for OnSelected Piercing"
        );
        assert_eq!(captured.0[0].per_stack, 1);
        assert_eq!(captured.0[0].max_stacks, 3);
        assert_eq!(captured.0[0].chip_name, "Piercing Shot");
    }

    #[test]
    fn dispatch_fires_size_boost_applied_for_on_selected_size_boost() {
        let mut app = typed_dispatch_test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "Big Shot".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Common,
                max_stacks: 3,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnSelected,
                    then: vec![EffectNode::Do(Effect::SizeBoost(Target::Bolt, 5.0))],
                }],
                ingredients: None,
                template_name: None,
            });

        send_chip_selected(&mut app, "Big Shot");
        tick(&mut app);

        let captured = app.world().resource::<CapturedSizeBoostApplied>();
        assert_eq!(
            captured.0.len(),
            1,
            "dispatch should fire SizeBoostApplied for OnSelected SizeBoost"
        );
        assert_eq!(
            captured.0[0].target,
            crate::effect::definition::Target::Bolt
        );
        assert!((captured.0[0].per_stack - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn dispatch_fires_speed_boost_applied_for_both_bolt_and_breaker() {
        let mut app = typed_dispatch_test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "Speed Mix".to_owned(),
                description: "test".to_owned(),
                rarity: Rarity::Common,
                max_stacks: 3,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnSelected,
                    then: vec![
                        EffectNode::Do(Effect::SpeedBoost {
                            target: Target::Bolt,
                            multiplier: 0.1,
                        }),
                        EffectNode::Do(Effect::SpeedBoost {
                            target: Target::Breaker,
                            multiplier: 0.2,
                        }),
                    ],
                }],
                ingredients: None,
                template_name: None,
            });

        send_chip_selected(&mut app, "Speed Mix");
        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostApplied>();
        assert_eq!(
            captured.0.len(),
            2,
            "dispatch should fire two SpeedBoostApplied events for both Bolt and Breaker targets"
        );
        assert_eq!(
            captured.0[0].target,
            crate::effect::definition::Target::Bolt
        );
        assert_eq!(
            captured.0[1].target,
            crate::effect::definition::Target::Breaker
        );
    }

    // =========================================================================
    // B12d: dispatch_chip_effects finds evolution chip in registry (B7 fix)
    // Behaviors 19-20
    // =========================================================================

    /// Behavior 19: Evolution chip is found in unified `ChipRegistry` and
    /// `OnSelected` effects fire. Before B12d, evolution chips were excluded
    /// from `ChipRegistry` (stored only in `EvolutionRegistry`), so
    /// `registry.get("Barrage")` returned None — this is the B7 fix.
    #[test]
    fn dispatch_finds_evolution_chip_in_registry_and_applies_effects() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        // Insert an Evolution-rarity chip into the unified ChipRegistry
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "Barrage".to_owned(),
                description: "Evolution chip".to_owned(),
                rarity: Rarity::Evolution,
                max_stacks: 1,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnSelected,
                    then: vec![EffectNode::Do(Effect::Piercing(5))],
                }],
                ingredients: Some(vec![crate::chips::definition::EvolutionIngredient {
                    chip_name: "Piercing Shot".to_owned(),
                    stacks_required: 2,
                }]),
                template_name: None,
            });
        app.init_resource::<ChipInventory>();

        send_chip_selected(&mut app, "Barrage");
        tick(&mut app);

        // Bolt should have Piercing(5) from the evolution chip's OnSelected effect
        let piercing = app
            .world_mut()
            .query::<&Piercing>()
            .iter(app.world())
            .next()
            .expect("bolt should have Piercing component after evolution chip selected (B7 fix)");
        assert_eq!(
            piercing.0, 5,
            "Piercing value should be 5 from evolution chip's OnSelected(Piercing(5))"
        );

        // ChipInventory should track the evolution chip
        let inventory = app.world().resource::<ChipInventory>();
        assert_eq!(
            inventory.stacks("Barrage"),
            1,
            "ChipInventory should have 1 stack of 'Barrage'"
        );
    }

    /// Behavior 20: Evolution chip with triggered chain pushes to `ActiveEffects`.
    #[test]
    fn dispatch_handles_triggered_chain_for_evolution_chip() {
        let mut app = test_app();

        // Insert an Evolution chip with a triggered chain (OnPerfectBump → Shockwave)
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition {
                name: "Evo Surge".to_owned(),
                description: "Evolution with triggered effect".to_owned(),
                rarity: Rarity::Evolution,
                max_stacks: 1,
                effects: vec![EffectNode::When {
                    trigger: Trigger::OnPerfectBump,
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                }],
                ingredients: None,
                template_name: None,
            });

        send_chip_selected(&mut app, "Evo Surge");
        tick(&mut app);

        let active = app.world().resource::<ActiveEffects>();
        assert_eq!(
            active.0.len(),
            1,
            "evolution chip's triggered chain should be pushed to ActiveEffects"
        );
        assert_eq!(
            active.0[0].0,
            Some("Evo Surge".to_owned()),
            "ActiveEffects entry should carry the evolution chip name"
        );
        assert_eq!(
            active.0[0].1,
            EffectNode::When {
                trigger: Trigger::OnPerfectBump,
                then: vec![EffectNode::Do(Effect::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                })]
            },
            "ActiveEffects chain should match the evolution chip's triggered effect"
        );
    }

    /// Edge case for behavior 20: unknown evolution chip name is skipped.
    #[test]
    fn dispatch_skips_unknown_evolution_chip_name() {
        let mut app = test_app();
        app.init_resource::<ChipInventory>();

        // Don't add any chip named "Unknown Evo" to the registry
        send_chip_selected(&mut app, "Unknown Evo");
        tick(&mut app);

        let inventory = app.world().resource::<ChipInventory>();
        assert_eq!(
            inventory.total_held(),
            0,
            "unknown chip name should be skipped, inventory remains empty"
        );
        let active = app.world().resource::<ActiveEffects>();
        assert!(
            active.0.is_empty(),
            "unknown chip name should not push to ActiveEffects"
        );
    }
}
