//! System that reads [`ChipSelected`] messages and applies or stacks
//! effect components on the bolt and breaker entities.

use bevy::prelude::*;
use tracing::debug;

use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    chips::{
        components::*,
        definition::{AmpEffect, AugmentEffect, ChipEffect},
        queries::{EffectQueryBolt, EffectQueryBreaker},
        resources::ChipRegistry,
    },
    ui::messages::ChipSelected,
};

/// Reads [`ChipSelected`] messages, looks up the chip definition in the
/// [`ChipRegistry`], and inserts or updates effect components on the
/// appropriate entity (bolt for Amp effects, breaker for Augment effects).
///
/// Stacking logic: adds the per-stack value to any existing component.
/// Stops stacking when `current / per_stack >= max_stacks`.
///
/// Overclock chips are a no-op here — deferred to phase 4d.
pub(crate) fn apply_chip_effect(
    mut reader: MessageReader<ChipSelected>,
    registry: Option<Res<ChipRegistry>>,
    mut bolt_query: Query<EffectQueryBolt, With<Bolt>>,
    mut breaker_query: Query<EffectQueryBreaker, With<Breaker>>,
    mut commands: Commands,
) {
    let Some(registry) = registry else {
        return;
    };
    for msg in reader.read() {
        let Some(chip) = registry.chips.iter().find(|c| c.name == msg.name) else {
            continue;
        };
        let max_stacks = chip.max_stacks;
        match chip.effect {
            ChipEffect::Amp(amp) => {
                for (entity, mut p, mut d, mut bs, mut ch, mut sz) in &mut bolt_query {
                    match amp {
                        AmpEffect::Piercing(v) => stack_u32(
                            entity,
                            p.as_deref_mut().map(|c| &mut c.0),
                            v,
                            max_stacks,
                            &mut commands,
                            Piercing,
                        ),
                        AmpEffect::DamageBoost(v) => stack_f32(
                            entity,
                            d.as_deref_mut().map(|c| &mut c.0),
                            v,
                            max_stacks,
                            &mut commands,
                            DamageBoost,
                        ),
                        AmpEffect::SpeedBoost(v) => stack_f32(
                            entity,
                            bs.as_deref_mut().map(|c| &mut c.0),
                            v,
                            max_stacks,
                            &mut commands,
                            BoltSpeedBoost,
                        ),
                        AmpEffect::ChainHit(v) => stack_u32(
                            entity,
                            ch.as_deref_mut().map(|c| &mut c.0),
                            v,
                            max_stacks,
                            &mut commands,
                            ChainHit,
                        ),
                        AmpEffect::SizeBoost(v) => stack_f32(
                            entity,
                            sz.as_deref_mut().map(|c| &mut c.0),
                            v,
                            max_stacks,
                            &mut commands,
                            BoltSizeBoost,
                        ),
                    }
                }
            }
            ChipEffect::Augment(aug) => {
                for (entity, mut w, mut s, mut b, mut t) in &mut breaker_query {
                    match aug {
                        AugmentEffect::WidthBoost(v) => stack_f32(
                            entity,
                            w.as_deref_mut().map(|c| &mut c.0),
                            v,
                            max_stacks,
                            &mut commands,
                            WidthBoost,
                        ),
                        AugmentEffect::SpeedBoost(v) => stack_f32(
                            entity,
                            s.as_deref_mut().map(|c| &mut c.0),
                            v,
                            max_stacks,
                            &mut commands,
                            BreakerSpeedBoost,
                        ),
                        AugmentEffect::BumpForce(v) => stack_f32(
                            entity,
                            b.as_deref_mut().map(|c| &mut c.0),
                            v,
                            max_stacks,
                            &mut commands,
                            BumpForceBoost,
                        ),
                        AugmentEffect::TiltControl(v) => stack_f32(
                            entity,
                            t.as_deref_mut().map(|c| &mut c.0),
                            v,
                            max_stacks,
                            &mut commands,
                            TiltControlBoost,
                        ),
                    }
                }
            }
            ChipEffect::Overclock => {
                debug!("overclock effects deferred to 4d");
            }
        }
    }
}

/// Stacks a `u32` component field on an entity.
///
/// If `field` is `Some`, adds `per_stack` when below the cap.
/// If `field` is `None`, inserts the component with `per_stack` as the initial value.
fn stack_u32<C, F>(
    entity: Entity,
    field: Option<&mut u32>,
    per_stack: u32,
    max_stacks: u32,
    commands: &mut Commands,
    constructor: F,
) where
    C: Component,
    F: FnOnce(u32) -> C,
{
    if per_stack == 0 {
        return;
    }
    if let Some(current) = field {
        if *current / per_stack < max_stacks {
            *current += per_stack;
        }
    } else {
        commands.entity(entity).insert(constructor(per_stack));
    }
}

/// Stacks an `f32` component field on an entity.
///
/// If `field` is `Some`, adds `per_stack` when below the cap.
/// If `field` is `None`, inserts the component with `per_stack` as the initial value.
fn stack_f32<C, F>(
    entity: Entity,
    field: Option<&mut f32>,
    per_stack: f32,
    max_stacks: u32,
    commands: &mut Commands,
    constructor: F,
) where
    C: Component,
    F: FnOnce(f32) -> C,
{
    if per_stack == 0.0 {
        return;
    }
    if let Some(current) = field {
        // Compare via f64 to avoid u32→f32 precision loss lint.
        if f64::from(*current / per_stack) < f64::from(max_stacks) {
            *current += per_stack;
        }
    } else {
        commands.entity(entity).insert(constructor(per_stack));
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
            ChipKind,
            definition::{AmpEffect, AugmentEffect, ChipDefinition, ChipEffect},
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
            .add_systems(Update, (enqueue_chip_selected, apply_chip_effect).chain());
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn send_chip_selected(app: &mut App, name: &str, kind: ChipKind) {
        app.insert_resource(PendingChipSelected(Some(ChipSelected {
            name: name.to_owned(),
            kind,
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
            .chips
            .push(ChipDefinition::test(
                "Piercing Shot",
                ChipKind::Amp,
                ChipEffect::Amp(AmpEffect::Piercing(1)),
                3,
            ));

        send_chip_selected(&mut app, "Piercing Shot", ChipKind::Amp);
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
            .chips
            .push(ChipDefinition::test(
                "Piercing Shot",
                ChipKind::Amp,
                ChipEffect::Amp(AmpEffect::Piercing(1)),
                3,
            ));

        send_chip_selected(&mut app, "Piercing Shot", ChipKind::Amp);
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
            .chips
            .push(ChipDefinition::test(
                "Piercing Shot",
                ChipKind::Amp,
                ChipEffect::Amp(AmpEffect::Piercing(1)),
                3,
            ));

        send_chip_selected(&mut app, "Piercing Shot", ChipKind::Amp);
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
            .chips
            .push(ChipDefinition::test(
                "Wide Breaker",
                ChipKind::Augment,
                ChipEffect::Augment(AugmentEffect::WidthBoost(20.0)),
                3,
            ));

        send_chip_selected(&mut app, "Wide Breaker", ChipKind::Augment);
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
            .chips
            .push(ChipDefinition::test(
                "Wide Breaker",
                ChipKind::Augment,
                ChipEffect::Augment(AugmentEffect::WidthBoost(20.0)),
                3,
            ));

        send_chip_selected(&mut app, "Wide Breaker", ChipKind::Augment);
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

        send_chip_selected(&mut app, "Nonexistent", ChipKind::Amp);
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
            .chips
            .push(ChipDefinition::test(
                "Damage Up",
                ChipKind::Amp,
                ChipEffect::Amp(AmpEffect::DamageBoost(1.5)),
                2,
            ));

        send_chip_selected(&mut app, "Damage Up", ChipKind::Amp);
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
            .chips
            .push(ChipDefinition::test(
                "Damage Up",
                ChipKind::Amp,
                ChipEffect::Amp(AmpEffect::DamageBoost(1.5)),
                2,
            ));

        send_chip_selected(&mut app, "Damage Up", ChipKind::Amp);
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
            .chips
            .push(ChipDefinition::test(
                "Speed Up",
                ChipKind::Amp,
                ChipEffect::Amp(AmpEffect::SpeedBoost(50.0)),
                3,
            ));

        send_chip_selected(&mut app, "Speed Up", ChipKind::Amp);
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
            .chips
            .push(ChipDefinition::test(
                "Chain",
                ChipKind::Amp,
                ChipEffect::Amp(AmpEffect::ChainHit(2)),
                3,
            ));

        send_chip_selected(&mut app, "Chain", ChipKind::Amp);
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
            .chips
            .push(ChipDefinition::test(
                "Big Bolt",
                ChipKind::Amp,
                ChipEffect::Amp(AmpEffect::SizeBoost(0.5)),
                3,
            ));

        send_chip_selected(&mut app, "Big Bolt", ChipKind::Amp);
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
            .chips
            .push(ChipDefinition::test(
                "Fast Breaker",
                ChipKind::Augment,
                ChipEffect::Augment(AugmentEffect::SpeedBoost(30.0)),
                3,
            ));

        send_chip_selected(&mut app, "Fast Breaker", ChipKind::Augment);
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
            .chips
            .push(ChipDefinition::test(
                "Power Bump",
                ChipKind::Augment,
                ChipEffect::Augment(AugmentEffect::BumpForce(10.0)),
                3,
            ));

        send_chip_selected(&mut app, "Power Bump", ChipKind::Augment);
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
            .chips
            .push(ChipDefinition::test(
                "Tilt Control",
                ChipKind::Augment,
                ChipEffect::Augment(AugmentEffect::TiltControl(5.0)),
                3,
            ));

        send_chip_selected(&mut app, "Tilt Control", ChipKind::Augment);
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
    // Tests: Overclock — no components added
    // ---------------------------------------------------------------------------

    #[test]
    fn overclock_chip_adds_no_effect_components() {
        let mut app = test_app();

        app.world_mut().spawn(Bolt);
        app.world_mut().spawn(Breaker);
        app.world_mut()
            .resource_mut::<ChipRegistry>()
            .chips
            .push(ChipDefinition::test(
                "Surge",
                ChipKind::Overclock,
                ChipEffect::Overclock,
                1,
            ));

        send_chip_selected(&mut app, "Surge", ChipKind::Overclock);
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
