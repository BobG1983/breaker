use crate::types::*;

// -------------------------------------------------------------------------
// FrameMutation — RON deserialization
// -------------------------------------------------------------------------

#[test]
fn frame_mutation_set_breaker_state_parses_from_ron() {
    let ron = "(frame: 3, mutation: SetDashState(Braking))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SetDashState should parse");
    assert_eq!(result.frame, 3);
    assert_eq!(
        result.mutation,
        MutationKind::SetDashState(ScenarioDashState::Braking)
    );
}

#[test]
fn frame_mutation_set_timer_remaining_parses_from_ron() {
    let ron = "(frame: 5, mutation: SetTimerRemaining(61.0))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SetTimerRemaining should parse");
    assert_eq!(result.frame, 5);
    assert_eq!(result.mutation, MutationKind::SetTimerRemaining(61.0));
}

#[test]
fn frame_mutation_spawn_extra_entities_parses_from_ron() {
    let ron = "(frame: 119, mutation: SpawnExtraEntities(200))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraEntities should parse");
    assert_eq!(result.frame, 119);
    assert_eq!(result.mutation, MutationKind::SpawnExtraEntities(200));
}

#[test]
fn frame_mutation_move_bolt_parses_from_ron() {
    let ron = "(frame: 5, mutation: MoveBolt(999.0, 999.0))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation MoveBolt should parse");
    assert_eq!(result.frame, 5);
    assert_eq!(result.mutation, MutationKind::MoveBolt(999.0, 999.0));
}

#[test]
fn frame_mutation_toggle_pause_parses_from_ron() {
    let ron = "(frame: 3, mutation: TogglePause)";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation TogglePause should parse");
    assert_eq!(result.frame, 3);
    assert_eq!(result.mutation, MutationKind::TogglePause);
}

#[test]
fn frame_mutation_set_run_stat_nodes_cleared_parses_from_ron() {
    let ron = "(frame: 10, mutation: SetRunStat(NodesCleared, 5))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SetRunStat should parse");
    assert_eq!(result.frame, 10);
    assert_eq!(
        result.mutation,
        MutationKind::SetRunStat(RunStatCounter::NodesCleared, 5)
    );
}

#[test]
fn frame_mutation_decrement_run_stat_nodes_cleared_parses_from_ron() {
    let ron = "(frame: 30, mutation: DecrementRunStat(NodesCleared))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation DecrementRunStat should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(
        result.mutation,
        MutationKind::DecrementRunStat(RunStatCounter::NodesCleared)
    );
}

#[test]
fn run_stat_counter_all_variants_parse_from_ron() {
    let variants = [
        ("NodesCleared", RunStatCounter::NodesCleared),
        ("CellsDestroyed", RunStatCounter::CellsDestroyed),
        ("BumpsPerformed", RunStatCounter::BumpsPerformed),
        ("PerfectBumps", RunStatCounter::PerfectBumps),
        ("BoltsLost", RunStatCounter::BoltsLost),
    ];
    for (ron_str, expected) in &variants {
        let result: RunStatCounter = ron::de::from_str(ron_str)
            .unwrap_or_else(|e| panic!("RunStatCounter::{ron_str} should parse: {e}"));
        assert_eq!(
            result, *expected,
            "RunStatCounter::{ron_str} must parse to {expected:?}"
        );
    }
}

#[test]
fn frame_mutation_inject_over_stacked_chip_parses_from_ron() {
    let ron = r#"(frame: 30, mutation: InjectOverStackedChip(chip_name: "TestChip", stacks: 3, max_stacks: 2))"#;
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation InjectOverStackedChip should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(
        result.mutation,
        MutationKind::InjectOverStackedChip {
            chip_name:  "TestChip".to_owned(),
            stacks:     3,
            max_stacks: 2,
        }
    );
}

#[test]
fn frame_mutation_inject_duplicate_offers_parses_from_ron() {
    let ron = r#"(frame: 30, mutation: InjectDuplicateOffers(chip_name: "TestChip"))"#;
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation InjectDuplicateOffers should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(
        result.mutation,
        MutationKind::InjectDuplicateOffers {
            chip_name: "TestChip".to_owned(),
        }
    );
}

#[test]
fn frame_mutation_inject_maxed_chip_offer_parses_from_ron() {
    let ron = r#"(frame: 30, mutation: InjectMaxedChipOffer(chip_name: "TestChip"))"#;
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation InjectMaxedChipOffer should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(
        result.mutation,
        MutationKind::InjectMaxedChipOffer {
            chip_name: "TestChip".to_owned(),
        }
    );
}

#[test]
fn frame_mutation_spawn_extra_second_wind_walls_parses_from_ron() {
    let ron = "(frame: 30, mutation: SpawnExtraSecondWindWalls(2))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraSecondWindWalls should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::SpawnExtraSecondWindWalls(2));
}

#[test]
fn frame_mutation_spawn_extra_shield_walls_parses_from_ron() {
    let ron = "(frame: 30, mutation: SpawnExtraShieldWalls(2))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraShieldWalls should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::SpawnExtraShieldWalls(2));
}

#[test]
fn frame_mutation_spawn_extra_pulse_rings_parses_from_ron() {
    let ron = "(frame: 30, mutation: SpawnExtraPulseRings(25))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraPulseRings should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::SpawnExtraPulseRings(25));
}

// -------------------------------------------------------------------------
// FrameMutation — SpawnExtraChainArcs RON deserialization
// -------------------------------------------------------------------------

#[test]
fn frame_mutation_spawn_extra_chain_arcs_parses_from_ron() {
    let ron = "(frame: 30, mutation: SpawnExtraChainArcs(10))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraChainArcs should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::SpawnExtraChainArcs(10));
}

#[test]
fn frame_mutation_spawn_extra_chain_arcs_zero_parses_from_ron() {
    let ron = "(frame: 30, mutation: SpawnExtraChainArcs(0))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraChainArcs(0) should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::SpawnExtraChainArcs(0));
}

// -------------------------------------------------------------------------
// FrameMutation — InjectMismatchedBoltAabb RON deserialization (behaviors 14-15)
// -------------------------------------------------------------------------

#[test]
fn frame_mutation_inject_mismatched_bolt_aabb_parses_from_ron() {
    let ron = "(frame: 30, mutation: InjectMismatchedBoltAabb)";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation InjectMismatchedBoltAabb should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::InjectMismatchedBoltAabb);
}

#[test]
fn mutation_kind_inject_mismatched_bolt_aabb_equality() {
    let a = MutationKind::InjectMismatchedBoltAabb;
    let b = MutationKind::InjectMismatchedBoltAabb;
    assert_eq!(a, b, "Two InjectMismatchedBoltAabb values should be equal");
    assert_ne!(
        a,
        MutationKind::SpawnExtraGravityWells(0),
        "InjectMismatchedBoltAabb should not equal SpawnExtraGravityWells"
    );
}

// -------------------------------------------------------------------------
// FrameMutation — SpawnExtraGravityWells RON deserialization (behaviors 16-17)
// -------------------------------------------------------------------------

#[test]
fn frame_mutation_spawn_extra_gravity_wells_parses_from_ron() {
    let ron = "(frame: 30, mutation: SpawnExtraGravityWells(15))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraGravityWells should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::SpawnExtraGravityWells(15));
}

#[test]
fn frame_mutation_spawn_extra_gravity_wells_zero_parses_from_ron() {
    let ron = "(frame: 30, mutation: SpawnExtraGravityWells(0))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraGravityWells(0) should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::SpawnExtraGravityWells(0));
}

// -------------------------------------------------------------------------
// FrameMutation — SpawnExtraPrimaryBreakers RON deserialization
// -------------------------------------------------------------------------

#[test]
fn frame_mutation_spawn_extra_primary_breakers_parses_from_ron() {
    let ron = "(frame: 30, mutation: SpawnExtraPrimaryBreakers(1))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraPrimaryBreakers should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::SpawnExtraPrimaryBreakers(1));
}

#[test]
fn frame_mutation_spawn_extra_primary_breakers_zero_parses_from_ron() {
    let ron = "(frame: 30, mutation: SpawnExtraPrimaryBreakers(0))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraPrimaryBreakers(0) should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::SpawnExtraPrimaryBreakers(0));
}

// -------------------------------------------------------------------------
// ScenarioDashState — all variants parse from RON
// -------------------------------------------------------------------------

#[test]
fn scenario_breaker_state_all_variants_parse_from_ron() {
    let variants = [
        ("Idle", ScenarioDashState::Idle),
        ("Dashing", ScenarioDashState::Dashing),
        ("Braking", ScenarioDashState::Braking),
        ("Settling", ScenarioDashState::Settling),
    ];
    for (ron_str, expected) in &variants {
        let result: ScenarioDashState = ron::de::from_str(ron_str)
            .unwrap_or_else(|e| panic!("ScenarioDashState::{ron_str} should parse: {e}"));
        assert_eq!(
            result, *expected,
            "ScenarioDashState::{ron_str} must parse to {expected:?}"
        );
    }
}
