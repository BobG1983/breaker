use super::*;
use crate::effect::core::EffectSourceChip;

fn maintain_chain_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, maintain_tether_chain);
    app
}

/// Accumulates one fixed timestep then runs one update.
fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

// Behavior 22: maintain_tether_chain rebuilds chain when a bolt is lost
#[test]
fn maintain_tether_chain_rebuilds_when_bolt_lost() {
    let mut app = maintain_chain_app();

    // 3 bolt entities
    let _bolt_a = app.world_mut().spawn(Bolt).id();
    let bolt_b = app.world_mut().spawn(Bolt).id();
    let _bolt_c = app.world_mut().spawn(Bolt).id();

    // 2 existing chain beams
    app.world_mut().spawn(TetherChainBeam);
    app.world_mut().spawn(TetherChainBeam);

    app.world_mut().insert_resource(TetherChainActive {
        damage_mult: 1.5,
        effective_damage_multiplier: 1.0,
        source_chip: None,
        last_bolt_count: 3,
    });

    // Despawn middle bolt
    app.world_mut().despawn(bolt_b);

    tick(&mut app);

    let chain_active = app.world().resource::<TetherChainActive>();
    assert_eq!(
        chain_active.last_bolt_count, 2,
        "last_bolt_count should be updated to 2 after losing a bolt, got {}",
        chain_active.last_bolt_count
    );

    let mut chain_query = app
        .world_mut()
        .query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_beam_count = chain_query.iter(app.world()).count();
    assert_eq!(
        chain_beam_count, 1,
        "should have 1 chain beam for 2 remaining bolts, got {chain_beam_count}"
    );
}

// Behavior 23: maintain_tether_chain rebuilds chain when a bolt spawns
#[test]
fn maintain_tether_chain_rebuilds_when_bolt_spawns() {
    let mut app = maintain_chain_app();

    // 2 bolt entities
    app.world_mut().spawn(Bolt);
    app.world_mut().spawn(Bolt);

    // 1 existing chain beam
    app.world_mut().spawn(TetherChainBeam);

    app.world_mut().insert_resource(TetherChainActive {
        damage_mult: 1.0,
        effective_damage_multiplier: 1.0,
        source_chip: None,
        last_bolt_count: 2,
    });

    // Spawn additional bolt
    app.world_mut().spawn(Bolt);

    tick(&mut app);

    let chain_active = app.world().resource::<TetherChainActive>();
    assert_eq!(
        chain_active.last_bolt_count, 3,
        "last_bolt_count should be updated to 3, got {}",
        chain_active.last_bolt_count
    );

    let mut chain_query = app
        .world_mut()
        .query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_beam_count = chain_query.iter(app.world()).count();
    assert_eq!(
        chain_beam_count, 2,
        "should have 2 chain beams for 3 bolts, got {chain_beam_count}"
    );
}

// Behavior 24: maintain_tether_chain does nothing when bolt count unchanged
#[test]
fn maintain_tether_chain_does_nothing_when_bolt_count_unchanged() {
    let mut app = maintain_chain_app();

    // 3 bolt entities
    app.world_mut().spawn(Bolt);
    app.world_mut().spawn(Bolt);
    app.world_mut().spawn(Bolt);

    // 2 chain beams
    let beam_a = app.world_mut().spawn(TetherChainBeam).id();
    let beam_b = app.world_mut().spawn(TetherChainBeam).id();

    app.world_mut().insert_resource(TetherChainActive {
        damage_mult: 1.0,
        effective_damage_multiplier: 1.0,
        source_chip: None,
        last_bolt_count: 3,
    });

    tick(&mut app);

    let chain_active = app.world().resource::<TetherChainActive>();
    assert_eq!(
        chain_active.last_bolt_count, 3,
        "last_bolt_count should remain 3, got {}",
        chain_active.last_bolt_count
    );

    // Verify same beam entities still exist (not replaced)
    assert!(
        app.world().get_entity(beam_a).is_ok(),
        "beam_a should still exist when bolt count unchanged"
    );
    assert!(
        app.world().get_entity(beam_b).is_ok(),
        "beam_b should still exist when bolt count unchanged"
    );

    let mut chain_query = app
        .world_mut()
        .query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_beam_count = chain_query.iter(app.world()).count();
    assert_eq!(
        chain_beam_count, 2,
        "should still have 2 chain beams, got {chain_beam_count}"
    );
}

// Behavior 25: maintain_tether_chain does not run when TetherChainActive resource absent
#[test]
fn maintain_tether_chain_noop_when_resource_absent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Add the system but do NOT add a run_if guard -- the system itself should handle missing resource
    // (or the production code will use resource_exists run_if and the system won't run).
    // Either way: no TetherChainBeam entities should be created.
    app.add_systems(
        Update,
        maintain_tether_chain.run_if(resource_exists::<TetherChainActive>),
    );

    // 3 bolt entities, but no TetherChainActive resource
    app.world_mut().spawn(Bolt);
    app.world_mut().spawn(Bolt);
    app.world_mut().spawn(Bolt);

    tick(&mut app);

    let mut chain_query = app
        .world_mut()
        .query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_beam_count = chain_query.iter(app.world()).count();
    assert_eq!(
        chain_beam_count, 0,
        "should have 0 chain beams when TetherChainActive resource absent, got {chain_beam_count}"
    );
}

// Behavior 26: maintain_tether_chain creates beams with correct damage_mult and EDM from resource
#[test]
fn maintain_tether_chain_creates_beams_with_correct_properties() {
    let mut app = maintain_chain_app();

    // 2 bolt entities
    app.world_mut().spawn(Bolt);
    app.world_mut().spawn(Bolt);

    // 1 existing chain beam (from last_bolt_count=2)
    app.world_mut().spawn(TetherChainBeam);

    app.world_mut().insert_resource(TetherChainActive {
        damage_mult: 2.5,
        effective_damage_multiplier: 1.5,
        source_chip: Some("arcwelder".to_string()),
        last_bolt_count: 2,
    });

    // Spawn additional bolt to trigger rebuild
    app.world_mut().spawn(Bolt);

    tick(&mut app);

    // Each new beam should have the correct properties from the resource
    let mut beam_query =
        app.world_mut()
            .query::<(&TetherBeamComponent, &EffectSourceChip, &CleanupOnNodeExit)>();
    let beams: Vec<_> = beam_query.iter(app.world()).collect();
    assert_eq!(
        beams.len(),
        2,
        "should have 2 chain beams for 3 bolts, got {}",
        beams.len()
    );

    for (beam, esc, _cleanup) in &beams {
        assert!(
            (beam.damage_mult - 2.5).abs() < f32::EPSILON,
            "beam damage_mult should be 2.5, got {}",
            beam.damage_mult
        );
        assert!(
            (beam.effective_damage_multiplier - 1.5).abs() < f32::EPSILON,
            "beam EDM should be 1.5, got {}",
            beam.effective_damage_multiplier
        );
        assert_eq!(
            esc.0,
            Some("arcwelder".to_string()),
            "beam should have EffectSourceChip(Some(\"arcwelder\"))"
        );
    }
}

// Behavior 27: maintain_tether_chain handles all bolts dying (0 remaining)
#[test]
fn maintain_tether_chain_handles_all_bolts_dying() {
    let mut app = maintain_chain_app();

    // 2 bolt entities
    let bolt_a = app.world_mut().spawn(Bolt).id();
    let bolt_b = app.world_mut().spawn(Bolt).id();

    // 1 chain beam
    app.world_mut().spawn(TetherChainBeam);

    app.world_mut().insert_resource(TetherChainActive {
        damage_mult: 1.0,
        effective_damage_multiplier: 1.0,
        source_chip: None,
        last_bolt_count: 2,
    });

    // Despawn both bolts
    app.world_mut().despawn(bolt_a);
    app.world_mut().despawn(bolt_b);

    tick(&mut app);

    let chain_active = app.world().resource::<TetherChainActive>();
    assert_eq!(
        chain_active.last_bolt_count, 0,
        "last_bolt_count should be 0, got {}",
        chain_active.last_bolt_count
    );

    let mut chain_query = app
        .world_mut()
        .query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_beam_count = chain_query.iter(app.world()).count();
    assert_eq!(
        chain_beam_count, 0,
        "should have 0 chain beams when all bolts are gone, got {chain_beam_count}"
    );

    // Resource should still exist (chain can resume if bolts spawn)
    assert!(
        app.world().contains_resource::<TetherChainActive>(),
        "TetherChainActive should still exist after all bolts die"
    );
}

// Behavior 28: maintain_tether_chain handles repair from zero (1 bolt spawns)
#[test]
fn maintain_tether_chain_handles_repair_from_zero_one_bolt() {
    let mut app = maintain_chain_app();

    app.world_mut().insert_resource(TetherChainActive {
        damage_mult: 1.0,
        effective_damage_multiplier: 1.0,
        source_chip: None,
        last_bolt_count: 0,
    });

    // Spawn 1 bolt
    app.world_mut().spawn(Bolt);

    tick(&mut app);

    let chain_active = app.world().resource::<TetherChainActive>();
    assert_eq!(
        chain_active.last_bolt_count, 1,
        "last_bolt_count should be 1, got {}",
        chain_active.last_bolt_count
    );

    let mut chain_query = app
        .world_mut()
        .query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_beam_count = chain_query.iter(app.world()).count();
    assert_eq!(
        chain_beam_count, 0,
        "should have 0 chain beams with only 1 bolt (need 2 for a beam), got {chain_beam_count}"
    );
}

// Behavior 29: maintain_tether_chain handles repair from zero (2 bolts spawn)
#[test]
fn maintain_tether_chain_handles_repair_from_zero_two_bolts() {
    let mut app = maintain_chain_app();

    app.world_mut().insert_resource(TetherChainActive {
        damage_mult: 1.0,
        effective_damage_multiplier: 1.0,
        source_chip: None,
        last_bolt_count: 0,
    });

    // Spawn 2 bolts
    app.world_mut().spawn(Bolt);
    app.world_mut().spawn(Bolt);

    tick(&mut app);

    let chain_active = app.world().resource::<TetherChainActive>();
    assert_eq!(
        chain_active.last_bolt_count, 2,
        "last_bolt_count should be 2, got {}",
        chain_active.last_bolt_count
    );

    let mut chain_query = app
        .world_mut()
        .query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_beam_count = chain_query.iter(app.world()).count();
    assert_eq!(
        chain_beam_count, 1,
        "should have 1 chain beam for 2 bolts, got {chain_beam_count}"
    );
}
