//! Behavior 17 — Bridge → Tick set ordering must hold so downstream systems
//! see the `BoundEffects` that the watcher inserted earlier in the same tick.

use bevy::prelude::*;

use super::{
    super::stamp_spawned_bolts,
    helpers::{set_registry, speed_boost_tree},
};
use crate::{
    bolt::components::Bolt,
    effect_v3::{
        sets::EffectV3Systems,
        storage::{BoundEffects, SpawnStampRegistry},
        types::{EntityKind, Tree},
    },
    shared::test_utils::{TestAppBuilder, tick},
};

/// Resource used by a `Tick`-set reader system to confirm that `BoundEffects`
/// are visible *after* the `Bridge` set runs, within the same tick.
#[derive(Resource, Default)]
struct TickObservation {
    seen_entries: Vec<(String, Tree)>,
    ticks:        u32,
}

fn tick_observer(mut obs: ResMut<TickObservation>, query: Query<&BoundEffects>) {
    obs.ticks += 1;
    for bound in &query {
        for entry in &bound.0 {
            obs.seen_entries.push(entry.clone());
        }
    }
}

fn ordering_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_resource::<SpawnStampRegistry>()
        .with_resource::<TickObservation>()
        .with_system(
            FixedUpdate,
            stamp_spawned_bolts.in_set(EffectV3Systems::Bridge),
        )
        .with_system(FixedUpdate, tick_observer.in_set(EffectV3Systems::Tick))
        .build();
    app.configure_sets(
        FixedUpdate,
        (
            EffectV3Systems::Bridge,
            EffectV3Systems::Tick.after(EffectV3Systems::Bridge),
            EffectV3Systems::Conditions.after(EffectV3Systems::Tick),
        ),
    );
    app
}

#[test]
fn bridge_runs_before_tick_so_downstream_sees_stamped_bound_effects() {
    let mut app = ordering_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );
    app.world_mut().spawn(Bolt);
    tick(&mut app);

    let obs = app.world().resource::<TickObservation>();
    assert_eq!(obs.ticks, 1);
    assert_eq!(
        obs.seen_entries.len(),
        1,
        "Tick-set system must observe the BoundEffects that Bridge inserted \
         earlier in the same FixedUpdate tick"
    );
    assert_eq!(obs.seen_entries[0].0, "chip_a");
    assert_eq!(obs.seen_entries[0].1, speed_boost_tree(1.5));
}

#[test]
fn bridge_tick_ordering_holds_across_two_ticks() {
    let mut app = ordering_test_app();
    set_registry(
        &mut app,
        vec![(
            EntityKind::Bolt,
            "chip_a".to_string(),
            speed_boost_tree(1.5),
        )],
    );

    // First bolt is spawned before tick 1.
    app.world_mut().spawn(Bolt);
    tick(&mut app);

    // Second bolt is spawned after tick 1 and before tick 2.
    app.world_mut().spawn(Bolt);
    tick(&mut app);

    let obs = app.world().resource::<TickObservation>();
    assert_eq!(obs.ticks, 2);
    // Tick 1: observer sees 1 BoundEffects (the first bolt).
    // Tick 2: observer sees 2 BoundEffects (both bolts). Total = 1 + 2 = 3.
    assert_eq!(
        obs.seen_entries.len(),
        3,
        "Tick-observer should see 1 BoundEffects on tick 1 and 2 on tick 2 \
         (Bridge → Tick ordering must hold every tick)"
    );
}
