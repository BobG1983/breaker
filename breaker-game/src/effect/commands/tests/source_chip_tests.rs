use bevy::prelude::*;

use crate::effect::{commands::ext::*, core::*, effects::damage_boost::ActiveDamageBoosts};

// -- Section I: commands.rs source_chip threading tests ───────────────────

#[test]
fn fire_effect_command_passes_source_chip_to_fire() {
    let mut world = World::new();
    let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

    let cmd = FireEffectCommand {
        entity,
        effect: EffectKind::DamageBoost(2.0),
        source_chip: "test_chip".to_string(),
    };
    cmd.apply(&mut world);

    let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        boosts.0,
        vec![2.0],
        "fire() should have been called — ActiveDamageBoosts should have [2.0]"
    );
}

#[test]
fn fire_effect_command_with_empty_source_chip() {
    let mut world = World::new();
    let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

    let cmd = FireEffectCommand {
        entity,
        effect: EffectKind::DamageBoost(2.0),
        source_chip: String::new(),
    };
    cmd.apply(&mut world);

    let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        boosts.0,
        vec![2.0],
        "fire() should work with empty source_chip"
    );
}

#[test]
fn reverse_effect_command_passes_source_chip_to_reverse() {
    let mut world = World::new();
    let entity = world.spawn(ActiveDamageBoosts(vec![2.0])).id();

    let cmd = ReverseEffectCommand {
        entity,
        effect: EffectKind::DamageBoost(2.0),
        source_chip: String::new(),
    };
    cmd.apply(&mut world);

    let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert!(
        boosts.0.is_empty(),
        "reverse() should have removed the 2.0 entry — ActiveDamageBoosts should be empty"
    );
}

#[test]
fn fire_effect_extension_queues_command_that_fires_effect() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity = app.world_mut().spawn(ActiveDamageBoosts(vec![])).id();

    // Queue the fire_effect command via a system
    app.add_systems(Update, move |mut commands: Commands| {
        commands.fire_effect(
            entity,
            EffectKind::DamageBoost(2.0),
            "chip_name".to_string(),
        );
    });

    app.update();

    let boosts = app.world().get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        boosts.0,
        vec![2.0],
        "fire_effect command should have been applied — ActiveDamageBoosts should have [2.0]"
    );
}

#[test]
fn transfer_command_passes_chip_name_to_fire_for_do_children() {
    let mut world = World::new();
    let entity = world
        .spawn((
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
        ))
        .id();

    let cmd = TransferCommand {
        entity,
        chip_name: "transfer_chip".to_string(),
        children: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        permanent: true,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        boosts.0,
        vec![2.0],
        "TransferCommand should fire DamageBoost via chip_name as source_chip"
    );
}
