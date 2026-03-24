//! Thin dispatcher: reads [`ChipSelected`] messages, looks up the chip in the
//! [`ChipRegistry`], and triggers [`ChipEffectApplied`] for per-effect observers.

use bevy::prelude::*;
use tracing::debug;

use crate::{
    chips::{definition::ChipEffectApplied, inventory::ChipInventory, resources::ChipRegistry},
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
            commands.trigger(ChipEffectApplied {
                effect: effect.clone(),
                max_stacks: chip.max_stacks,
                chip_name: msg.name.clone(),
            });
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
            definition::{AmpEffect, AugmentEffect, ChipDefinition, ChipEffect, TriggerChain},
            effects::*,
            resources::ChipRegistry,
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
    // Tests: Amp — Piercing
    // ---------------------------------------------------------------------------

    #[test]
    fn piercing_inserts_on_bolt_when_amp_piercing_selected() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Piercing Shot",
                ChipEffect::Amp(AmpEffect::Piercing(1)),
                3,
            ));

        send_chip_selected(&mut app, "Piercing Shot");
        tick(&mut app);

        let piercing = app
            .world_mut()
            .query::<&Piercing>()
            .iter(app.world())
            .next()
            .expect("bolt should have Piercing component after chip selected");
        assert_eq!(piercing.0, 1);
    }

    #[test]
    fn piercing_stacks_from_1_to_2_on_reselection() {
        let mut app = test_app();

        let bolt = app.world_mut().spawn((Bolt, Piercing(1))).id();
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Piercing Shot",
                ChipEffect::Amp(AmpEffect::Piercing(1)),
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
                ChipEffect::Amp(AmpEffect::Piercing(1)),
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

    // ---------------------------------------------------------------------------
    // Tests: Augment — WidthBoost
    // ---------------------------------------------------------------------------

    #[test]
    fn width_boost_inserts_on_breaker_when_augment_width_boost_selected() {
        let mut app = test_app();

        app.world_mut().spawn(Breaker);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Wide Breaker",
                ChipEffect::Augment(AugmentEffect::WidthBoost(20.0)),
                3,
            ));

        send_chip_selected(&mut app, "Wide Breaker");
        tick(&mut app);

        let wb = app
            .world_mut()
            .query::<&WidthBoost>()
            .iter(app.world())
            .next()
            .expect("breaker should have WidthBoost component after chip selected");
        assert!(
            (wb.0 - 20.0).abs() < f32::EPSILON,
            "WidthBoost should be 20.0, got {}",
            wb.0
        );
    }

    #[test]
    fn width_boost_stacks_from_20_to_40_on_reselection() {
        let mut app = test_app();

        let breaker = app.world_mut().spawn((Breaker, WidthBoost(20.0))).id();
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Wide Breaker",
                ChipEffect::Augment(AugmentEffect::WidthBoost(20.0)),
                3,
            ));

        send_chip_selected(&mut app, "Wide Breaker");
        tick(&mut app);

        let wb = app
            .world()
            .entity(breaker)
            .get::<WidthBoost>()
            .expect("breaker should still have WidthBoost component");
        assert!(
            (wb.0 - 40.0).abs() < f32::EPSILON,
            "WidthBoost should stack 20.0 → 40.0, got {}",
            wb.0
        );
    }

    // ---------------------------------------------------------------------------
    // Tests: Unknown chip name
    // ---------------------------------------------------------------------------

    #[test]
    fn no_components_added_for_unknown_chip_name() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut().spawn(Breaker);
        // Registry is empty — no chips registered

        send_chip_selected(&mut app, "Nonexistent");
        tick(&mut app);

        // No bolt effect components should be inserted
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

    // ---------------------------------------------------------------------------
    // Tests: Amp — DamageBoost
    // ---------------------------------------------------------------------------

    #[test]
    fn damage_boost_inserts_on_first_selection() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Damage Up",
                ChipEffect::Amp(AmpEffect::DamageBoost(1.5)),
                2,
            ));

        send_chip_selected(&mut app, "Damage Up");
        tick(&mut app);

        let db = app
            .world_mut()
            .query::<&DamageBoost>()
            .iter(app.world())
            .next()
            .expect("bolt should have DamageBoost component");
        assert!(
            (db.0 - 1.5).abs() < f32::EPSILON,
            "DamageBoost should be 1.5, got {}",
            db.0
        );
    }

    #[test]
    fn damage_boost_stacks_from_1_5_to_3_0_on_second_selection() {
        let mut app = test_app();

        app.world_mut().spawn((Bolt, DamageBoost(1.5)));
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Damage Up",
                ChipEffect::Amp(AmpEffect::DamageBoost(1.5)),
                2,
            ));

        send_chip_selected(&mut app, "Damage Up");
        tick(&mut app);

        let db = app
            .world_mut()
            .query::<&DamageBoost>()
            .iter(app.world())
            .next()
            .expect("bolt should still have DamageBoost component");
        assert!(
            (db.0 - 3.0).abs() < f32::EPSILON,
            "DamageBoost should stack 1.5 → 3.0, got {}",
            db.0
        );
    }

    // ---------------------------------------------------------------------------
    // Tests: Amp — BoltSpeedBoost
    // ---------------------------------------------------------------------------

    #[test]
    fn bolt_speed_boost_inserts_on_bolt() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Speed Up",
                ChipEffect::Amp(AmpEffect::SpeedBoost(50.0)),
                3,
            ));

        send_chip_selected(&mut app, "Speed Up");
        tick(&mut app);

        let sb = app
            .world_mut()
            .query::<&BoltSpeedBoost>()
            .iter(app.world())
            .next()
            .expect("bolt should have BoltSpeedBoost component");
        assert!(
            (sb.0 - 50.0).abs() < f32::EPSILON,
            "BoltSpeedBoost should be 50.0, got {}",
            sb.0
        );
    }

    // ---------------------------------------------------------------------------
    // Tests: Amp — ChainHit
    // ---------------------------------------------------------------------------

    #[test]
    fn chain_hit_inserts_on_bolt() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Chain",
                ChipEffect::Amp(AmpEffect::ChainHit(2)),
                3,
            ));

        send_chip_selected(&mut app, "Chain");
        tick(&mut app);

        let ch = app
            .world_mut()
            .query::<&ChainHit>()
            .iter(app.world())
            .next()
            .expect("bolt should have ChainHit component");
        assert_eq!(ch.0, 2, "ChainHit should be 2, got {}", ch.0);
    }

    // ---------------------------------------------------------------------------
    // Tests: Amp — BoltSizeBoost
    // ---------------------------------------------------------------------------

    #[test]
    fn bolt_size_boost_inserts_on_bolt() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Big Bolt",
                ChipEffect::Amp(AmpEffect::SizeBoost(0.5)),
                3,
            ));

        send_chip_selected(&mut app, "Big Bolt");
        tick(&mut app);

        let bsb = app
            .world_mut()
            .query::<&BoltSizeBoost>()
            .iter(app.world())
            .next()
            .expect("bolt should have BoltSizeBoost component");
        assert!(
            (bsb.0 - 0.5).abs() < f32::EPSILON,
            "BoltSizeBoost should be 0.5, got {}",
            bsb.0
        );
    }

    // ---------------------------------------------------------------------------
    // Tests: Augment — BreakerSpeedBoost
    // ---------------------------------------------------------------------------

    #[test]
    fn breaker_speed_boost_inserts_on_breaker() {
        let mut app = test_app();

        app.world_mut().spawn(Breaker);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Fast Breaker",
                ChipEffect::Augment(AugmentEffect::SpeedBoost(30.0)),
                3,
            ));

        send_chip_selected(&mut app, "Fast Breaker");
        tick(&mut app);

        let bsb = app
            .world_mut()
            .query::<&BreakerSpeedBoost>()
            .iter(app.world())
            .next()
            .expect("breaker should have BreakerSpeedBoost component");
        assert!(
            (bsb.0 - 30.0).abs() < f32::EPSILON,
            "BreakerSpeedBoost should be 30.0, got {}",
            bsb.0
        );
    }

    // ---------------------------------------------------------------------------
    // Tests: Augment — BumpForceBoost
    // ---------------------------------------------------------------------------

    #[test]
    fn bump_force_boost_inserts_on_breaker() {
        let mut app = test_app();

        app.world_mut().spawn(Breaker);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Power Bump",
                ChipEffect::Augment(AugmentEffect::BumpForce(10.0)),
                3,
            ));

        send_chip_selected(&mut app, "Power Bump");
        tick(&mut app);

        let bfb = app
            .world_mut()
            .query::<&BumpForceBoost>()
            .iter(app.world())
            .next()
            .expect("breaker should have BumpForceBoost component");
        assert!(
            (bfb.0 - 10.0).abs() < f32::EPSILON,
            "BumpForceBoost should be 10.0, got {}",
            bfb.0
        );
    }

    // ---------------------------------------------------------------------------
    // Tests: Augment — TiltControlBoost
    // ---------------------------------------------------------------------------

    #[test]
    fn tilt_control_boost_inserts_on_breaker() {
        let mut app = test_app();

        app.world_mut().spawn(Breaker);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Tilt Control",
                ChipEffect::Augment(AugmentEffect::TiltControl(5.0)),
                3,
            ));

        send_chip_selected(&mut app, "Tilt Control");
        tick(&mut app);

        let tcb = app
            .world_mut()
            .query::<&TiltControlBoost>()
            .iter(app.world())
            .next()
            .expect("breaker should have TiltControlBoost component");
        assert!(
            (tcb.0 - 5.0).abs() < f32::EPSILON,
            "TiltControlBoost should be 5.0, got {}",
            tcb.0
        );
    }

    // ---------------------------------------------------------------------------
    // Tests: apply_chip_effect updates ChipInventory
    // ---------------------------------------------------------------------------

    #[test]
    fn apply_chip_effect_adds_chip_to_inventory_on_chip_selected() {
        let mut app = test_app();
        app.init_resource::<crate::chips::inventory::ChipInventory>();

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Piercing Shot",
                ChipEffect::Amp(AmpEffect::Piercing(1)),
                3,
            ));

        send_chip_selected(&mut app, "Piercing Shot");
        tick(&mut app);

        let inventory = app
            .world()
            .resource::<crate::chips::inventory::ChipInventory>();
        assert_eq!(
            inventory.stacks("Piercing Shot"),
            1,
            "ChipInventory should track the selected chip at 1 stack"
        );
    }

    #[test]
    fn apply_chip_effect_does_not_add_inventory_entry_for_unknown_chip() {
        let mut app = test_app();
        app.init_resource::<crate::chips::inventory::ChipInventory>();

        // Registry is empty — no chips registered
        send_chip_selected(&mut app, "Nonexistent");
        tick(&mut app);

        let inventory = app
            .world()
            .resource::<crate::chips::inventory::ChipInventory>();
        assert_eq!(
            inventory.total_held(),
            0,
            "ChipInventory should remain empty for unknown chip"
        );
    }

    #[test]
    fn apply_chip_effect_does_not_add_when_already_maxed() {
        let mut app = test_app();
        app.init_resource::<crate::chips::inventory::ChipInventory>();

        let single_def = ChipDefinition::test("Single", ChipEffect::Amp(AmpEffect::Piercing(1)), 1);

        app.world_mut().spawn(Bolt);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(single_def.clone());

        // Pre-fill inventory to max
        let _ = app
            .world_mut()
            .resource_mut::<crate::chips::inventory::ChipInventory>()
            .add_chip("Single", &single_def);

        send_chip_selected(&mut app, "Single");
        tick(&mut app);

        let inventory = app
            .world()
            .resource::<crate::chips::inventory::ChipInventory>();
        assert_eq!(
            inventory.stacks("Single"),
            1,
            "ChipInventory should not exceed max_stacks"
        );
    }

    // ---------------------------------------------------------------------------
    // Tests: Overclock — no components added
    // ---------------------------------------------------------------------------

    #[test]
    fn overclock_chip_adds_no_effect_components() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut().spawn(Breaker);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .insert(ChipDefinition::test(
                "Surge",
                ChipEffect::Overclock(TriggerChain::test_shockwave(64.0)),
                1,
            ));

        send_chip_selected(&mut app, "Surge");
        tick(&mut app);

        // No bolt effect components
        assert!(
            app.world_mut()
                .query::<&Piercing>()
                .iter(app.world())
                .next()
                .is_none(),
            "Overclock should not add Piercing"
        );
        assert!(
            app.world_mut()
                .query::<&DamageBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "Overclock should not add DamageBoost"
        );
        assert!(
            app.world_mut()
                .query::<&BoltSpeedBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "Overclock should not add BoltSpeedBoost"
        );
        assert!(
            app.world_mut()
                .query::<&ChainHit>()
                .iter(app.world())
                .next()
                .is_none(),
            "Overclock should not add ChainHit"
        );
        assert!(
            app.world_mut()
                .query::<&BoltSizeBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "Overclock should not add BoltSizeBoost"
        );
        // No breaker effect components
        assert!(
            app.world_mut()
                .query::<&WidthBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "Overclock should not add WidthBoost"
        );
        assert!(
            app.world_mut()
                .query::<&BreakerSpeedBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "Overclock should not add BreakerSpeedBoost"
        );
        assert!(
            app.world_mut()
                .query::<&BumpForceBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "Overclock should not add BumpForceBoost"
        );
        assert!(
            app.world_mut()
                .query::<&TiltControlBoost>()
                .iter(app.world())
                .next()
                .is_none(),
            "Overclock should not add TiltControlBoost"
        );
    }
}
