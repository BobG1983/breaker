use super::super::*;

// -------------------------------------------------------------------------
// FrameMutation — RON deserialization
// -------------------------------------------------------------------------

#[test]
fn frame_mutation_set_breaker_state_parses_from_ron() {
    let ron = "(frame: 3, mutation: SetBreakerState(Braking))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SetBreakerState should parse");
    assert_eq!(result.frame, 3);
    assert_eq!(
        result.mutation,
        MutationKind::SetBreakerState(ScenarioBreakerState::Braking)
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
            chip_name: "TestChip".to_owned(),
            stacks: 3,
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
fn frame_mutation_inject_zero_charge_shield_parses_from_ron() {
    let ron = "(frame: 30, mutation: InjectZeroChargeShield)";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation InjectZeroChargeShield should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::InjectZeroChargeShield);
}

#[test]
fn frame_mutation_spawn_extra_pulse_rings_parses_from_ron() {
    let ron = "(frame: 30, mutation: SpawnExtraPulseRings(25))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation SpawnExtraPulseRings should parse");
    assert_eq!(result.frame, 30);
    assert_eq!(result.mutation, MutationKind::SpawnExtraPulseRings(25));
}

#[test]
fn frame_mutation_inject_wrong_effective_speed_parses_from_ron() {
    let ron = "(frame: 30, mutation: InjectWrongEffectiveSpeed(wrong_value: 99.0))";
    let result: FrameMutation =
        ron::de::from_str(ron).expect("FrameMutation InjectWrongEffectiveSpeed should parse");
    assert_eq!(result.frame, 30);
    assert!(
        matches!(
            result.mutation,
            MutationKind::InjectWrongEffectiveSpeed { wrong_value }
            if (wrong_value - 99.0).abs() < f32::EPSILON
        ),
        "InjectWrongEffectiveSpeed should parse with wrong_value=99.0"
    );
}

// -------------------------------------------------------------------------
// ScenarioBreakerState — all variants parse from RON
// -------------------------------------------------------------------------

#[test]
fn scenario_breaker_state_all_variants_parse_from_ron() {
    let variants = [
        ("Idle", ScenarioBreakerState::Idle),
        ("Dashing", ScenarioBreakerState::Dashing),
        ("Braking", ScenarioBreakerState::Braking),
        ("Settling", ScenarioBreakerState::Settling),
    ];
    for (ron_str, expected) in &variants {
        let result: ScenarioBreakerState = ron::de::from_str(ron_str)
            .unwrap_or_else(|e| panic!("ScenarioBreakerState::{ron_str} should parse: {e}"));
        assert_eq!(
            result, *expected,
            "ScenarioBreakerState::{ron_str} must parse to {expected:?}"
        );
    }
}
