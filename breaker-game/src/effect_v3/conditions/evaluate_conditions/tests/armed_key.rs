//! Behaviors B61 / B62 — `fire_scoped_tree` armed-key construction format pins.
//!
//! These tests pin the canonical armed-key format used by
//! `fire_scoped_tree` for both `When(...)` and `On(...)` scoped trees.
//!
//! POST-W5 the canonical armed key is `<source>:armed`, produced by
//! `SourceId::armed(SourceId::from(source.to_owned())).build()`.
//! These tests assert the armed entry in `BoundEffects` ends with `":armed"`
//! AND that the entry's `SourceId` (when wrapped) reports `is_armed() == true`.
//!
//! These tests FAIL at RED because production currently formats armed keys
//! as `format!("{source}#armed[0]")` (see `evaluate_conditions/system.rs:79,
//! 84, 107, 116`). Writer-code (GREEN) migrates the four callsites to the
//! builder.

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{
    effect_v3::{
        storage::BoundEffects,
        types::{Tree, Trigger},
    },
    prelude::{SourceId, SourceIdExt},
    state::types::NodeState,
};

// ── B61 — fire_scoped_tree When installs armed key with ":armed" suffix ──

#[test]
fn fire_scoped_tree_when_installs_armed_key_ending_in_colon_armed() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    // chip_siege has a During(NodeActive, When(Bumped, Fire(SpeedBoost))) tree.
    let entity = world
        .spawn(BoundEffects(vec![during_when_bumped_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");

    // Locate the armed entry — it MUST be the one whose key ends with ":armed".
    let armed_entry = bound
        .0
        .iter()
        .find(|(name, _)| name.ends_with(":armed"))
        .expect("Should find an armed entry whose key ends in \":armed\"");

    // The armed-key MUST equal the canonical builder-produced form:
    //   SourceId::armed(SourceId::from("chip_siege".to_owned())).build()
    let expected = SourceId::armed(SourceId::from("chip_siege".to_owned())).build();
    assert_eq!(
        armed_entry.0,
        expected.0.as_ref(),
        "Shape C armed key must equal {:?}, got {:?}",
        expected.0.as_ref(),
        armed_entry.0
    );

    // The armed entry must be a Tree::When(...).
    assert!(
        matches!(armed_entry.1, Tree::When(Trigger::Bumped, _)),
        "Shape C armed entry must be a Tree::When(Bumped, ...)"
    );

    // Round-trip: wrapping the armed-key in SourceId reports is_armed.
    let armed_id = SourceId::from(armed_entry.0.clone());
    assert!(
        armed_id.is_armed(),
        "Armed-key wrapped in SourceId must report is_armed() == true"
    );

    // No `#armed[` substring — old format must be gone post-W5.
    assert!(
        !armed_entry.0.contains("#armed["),
        "Armed key must NOT contain the legacy `#armed[` token; got {:?}",
        armed_entry.0
    );
}

// ── B62 — fire_scoped_tree On installs armed key with ":armed" suffix ──

#[test]
fn fire_scoped_tree_on_installs_armed_key_ending_in_colon_armed() {
    let mut world = World::new();
    world.insert_resource(State::new(NodeState::Playing));

    // chip_redirect has a During(NodeActive, On(Bump(Bolt), Fire(SpeedBoost))) tree.
    let entity = world
        .spawn(BoundEffects(vec![during_on_bump_bolt_speed_boost()]))
        .id();

    evaluate_conditions(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should exist");

    let armed_entry = bound
        .0
        .iter()
        .find(|(name, _)| name.ends_with(":armed"))
        .expect("Should find an armed entry whose key ends in \":armed\"");

    let expected = SourceId::armed(SourceId::from("chip_redirect".to_owned())).build();
    assert_eq!(
        armed_entry.0,
        expected.0.as_ref(),
        "Shape D armed key must equal {:?}, got {:?}",
        expected.0.as_ref(),
        armed_entry.0
    );

    // The armed entry must be a Tree::On(...).
    assert!(
        matches!(armed_entry.1, Tree::On(_, _)),
        "Shape D armed entry must be a Tree::On(...)"
    );

    let armed_id = SourceId::from(armed_entry.0.clone());
    assert!(
        armed_id.is_armed(),
        "Armed-key wrapped in SourceId must report is_armed() == true"
    );

    assert!(
        !armed_entry.0.contains("#armed["),
        "Armed key must NOT contain the legacy `#armed[` token; got {:?}",
        armed_entry.0
    );
}
