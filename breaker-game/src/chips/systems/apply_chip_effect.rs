//! Thin dispatcher: reads [`ChipSelected`] messages, looks up the chip in the
//! [`ChipRegistry`], and triggers [`ChipEffectApplied`] for per-effect observers.

use bevy::prelude::*;
use tracing::debug;

use crate::{
    chips::{
        definition::{ChipEffectApplied, TriggerChain},
        inventory::ChipInventory,
        resources::ChipRegistry,
    },
    effect::ActiveEffects,
    ui::messages::ChipSelected,
};

/// Reads [`ChipSelected`] messages, looks up the chip definition in the
/// [`ChipRegistry`], and triggers [`ChipEffectApplied`] for each selected chip.
///
/// Per-effect observers handle the actual stacking logic.
/// Overclock chips trigger the event too — observers self-select.
pub(crate) fn apply_chip_effect(
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
                TriggerChain::OnSelected(inner) => {
                    for leaf in inner {
                        commands.trigger(ChipEffectApplied {
                            effect: leaf.clone(),
                            max_stacks: chip.max_stacks,
                            chip_name: msg.name.clone(),
                        });
                    }
                }
                chain if chain.is_leaf() => {
                    commands.trigger(ChipEffectApplied {
                        effect: chain.clone(),
                        max_stacks: chip.max_stacks,
                        chip_name: msg.name.clone(),
                    });
                }
                // Any trigger-wrapper variant (OnPerfectBump, OnBump, OnImpact, etc.)
                // is pushed to ActiveEffects for runtime evaluation by bridge systems.
                chain => {
                    if let Some(ref mut active) = active_chains {
                        active.0.push((Some(msg.name.clone()), chain.clone()));
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
            definition::{ChipDefinition, ImpactTarget, Rarity, Target, TriggerChain},
            effects::*,
            inventory::ChipInventory,
            resources::ChipRegistry,
        },
        effect::ActiveEffects,
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
            .add_systems(Update, (enqueue_chip_selected, apply_chip_effect).chain())
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
                effects: vec![TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])],
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
                effects: vec![TriggerChain::OnSelected(vec![
                    TriggerChain::Piercing(1),
                    TriggerChain::DamageBoost(0.5),
                ])],
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

        let chain = TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
            ImpactTarget::Cell,
            vec![TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 32.0,
                stacks: 1,
                speed: 400.0,
            }],
        )]);
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
    // B3: apply_chip_effect updates ChipInventory (32)
    // ---------------------------------------------------------------------------

    #[test]
    fn apply_chip_effect_adds_chip_to_inventory_on_chip_selected() {
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
                effects: vec![TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])],
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
    fn apply_chip_effect_does_not_add_inventory_entry_for_unknown_chip() {
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
                effects: vec![TriggerChain::OnPerfectBump(vec![TriggerChain::SpawnBolt])],
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
                TriggerChain::Piercing(1),
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
                effects: vec![TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])],
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
                effects: vec![TriggerChain::OnSelected(vec![])],
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
                    TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)]),
                    TriggerChain::OnPerfectBump(vec![TriggerChain::SpawnBolt]),
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
            TriggerChain::OnPerfectBump(vec![TriggerChain::SpawnBolt])
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
                effects: vec![TriggerChain::OnSelected(vec![TriggerChain::SpeedBoost {
                    target: Target::AllBolts,
                    multiplier: 1.1,
                }])],
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
    // B3: Handlers ignore non-matching variants (28)
    // ---------------------------------------------------------------------------

    #[test]
    fn shockwave_via_chip_effect_applied_inserts_no_passive_components() {
        let mut app = test_app();

        let bolt = app.world_mut().spawn(Bolt).id();
        let breaker = app.world_mut().spawn(Breaker).id();

        app.world_mut().commands().trigger(ChipEffectApplied {
            effect: TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            },
            max_stacks: 1,
            chip_name: "test".to_owned(),
        });
        app.world_mut().flush();

        // Bolt should have no passive components
        assert!(
            app.world().entity(bolt).get::<Piercing>().is_none(),
            "Shockwave should not insert Piercing on bolt"
        );
        assert!(
            app.world().entity(bolt).get::<DamageBoost>().is_none(),
            "Shockwave should not insert DamageBoost on bolt"
        );
        assert!(
            app.world().entity(bolt).get::<BoltSpeedBoost>().is_none(),
            "Shockwave should not insert BoltSpeedBoost on bolt"
        );
        assert!(
            app.world().entity(bolt).get::<ChainHit>().is_none(),
            "Shockwave should not insert ChainHit on bolt"
        );
        assert!(
            app.world().entity(bolt).get::<BoltSizeBoost>().is_none(),
            "Shockwave should not insert BoltSizeBoost on bolt"
        );

        // Breaker should have no passive components
        assert!(
            app.world().entity(breaker).get::<WidthBoost>().is_none(),
            "Shockwave should not insert WidthBoost on breaker"
        );
        assert!(
            app.world()
                .entity(breaker)
                .get::<BreakerSpeedBoost>()
                .is_none(),
            "Shockwave should not insert BreakerSpeedBoost on breaker"
        );
        assert!(
            app.world()
                .entity(breaker)
                .get::<BumpForceBoost>()
                .is_none(),
            "Shockwave should not insert BumpForceBoost on breaker"
        );
        assert!(
            app.world()
                .entity(breaker)
                .get::<TiltControlBoost>()
                .is_none(),
            "Shockwave should not insert TiltControlBoost on breaker"
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
                TriggerChain::Piercing(1),
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
                TriggerChain::Piercing(1),
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
                effects: vec![TriggerChain::OnSelected(vec![TriggerChain::SizeBoost(
                    Target::Breaker,
                    20.0,
                )])],
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
    // B12b: apply_chip_effect dispatch patterns with EffectNode (behaviors 25-26)
    // These tests verify the EffectNode dispatch logic that apply_chip_effect
    // will use after migration. They exercise evaluate_node which fails
    // with todo!().
    // =========================================================================

    #[test]
    fn effect_node_on_selected_dispatches_leaf_effects() {
        use crate::effect::definition::{Effect, EffectNode, Trigger};
        use crate::effect::evaluate::{NodeEvalResult, TriggerKind, evaluate_node};

        // After migration, apply_chip_effect will match on
        // EffectNode::Trigger(Trigger::OnSelected, inner) and fire
        // ChipEffectApplied for each inner Leaf's Effect.
        let node = EffectNode::Trigger(
            Trigger::OnSelected,
            vec![EffectNode::Leaf(Effect::Piercing(1))],
        );
        // Verify EffectNode structure and inner extraction
        match &node {
            EffectNode::Trigger(Trigger::OnSelected, inner) => {
                assert_eq!(inner.len(), 1);
                match &inner[0] {
                    EffectNode::Leaf(effect) => {
                        assert_eq!(*effect, Effect::Piercing(1));
                    }
                    EffectNode::Trigger(..) => panic!("expected Leaf, got Trigger"),
                }
            }
            EffectNode::Trigger(..) | EffectNode::Leaf(..) => {
                panic!("expected Trigger(OnSelected, _)")
            }
        }
        // evaluate_node should return NoMatch for OnSelected — it's handled
        // by apply_chip_effect, not by bridges
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(result, vec![NodeEvalResult::NoMatch]);
    }

    #[test]
    fn effect_node_on_selected_multiple_leaves_extracts_all() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        let node = EffectNode::Trigger(
            Trigger::OnSelected,
            vec![
                EffectNode::Leaf(Effect::Piercing(1)),
                EffectNode::Leaf(Effect::DamageBoost(0.5)),
            ],
        );
        match &node {
            EffectNode::Trigger(Trigger::OnSelected, inner) => {
                assert_eq!(inner.len(), 2);
                assert_eq!(inner[0], EffectNode::Leaf(Effect::Piercing(1)));
                assert_eq!(inner[1], EffectNode::Leaf(Effect::DamageBoost(0.5)));
            }
            EffectNode::Trigger(..) | EffectNode::Leaf(..) => {
                panic!("expected Trigger(OnSelected, 2 children)")
            }
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

        // After migration, apply_chip_effect pushes non-OnSelected triggers
        // to ActiveEffects. Verify this EffectNode evaluates as expected.
        let node = EffectNode::Trigger(
            Trigger::OnPerfectBump,
            vec![EffectNode::Leaf(Effect::SpawnBolt)],
        );
        // Should NOT match OnSelected — bridge evaluation handles it
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::SpawnBolt)],
            "OnPerfectBump with Leaf(SpawnBolt) should fire on PerfectBump"
        );
    }

    #[test]
    fn effect_node_bare_leaf_pattern_for_direct_dispatch() {
        use crate::effect::{
            definition::{Effect, EffectNode},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        // After migration, a bare EffectNode::Leaf is treated as an immediate
        // passive — dispatched via ChipEffectApplied directly (replacing
        // the old is_leaf() check).
        let node = EffectNode::Leaf(Effect::Piercing(1));
        assert!(
            matches!(&node, EffectNode::Leaf(_)),
            "bare Leaf should be directly dispatchable"
        );
        // Verify it extracts correctly
        if let EffectNode::Leaf(effect) = &node {
            assert_eq!(*effect, Effect::Piercing(1));
        }
        // Bare leaf returns NoMatch from evaluate_node (fails with todo!)
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(result, vec![NodeEvalResult::NoMatch]);
    }
}
