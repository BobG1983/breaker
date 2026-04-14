//! Shared test helpers for the `SpawnStampRegistry` watcher tests.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::{
    stamp_spawned_bolts, stamp_spawned_breakers, stamp_spawned_cells, stamp_spawned_walls,
};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        sets::EffectV3Systems,
        storage::SpawnStampRegistry,
        types::{EffectType, EntityKind, Tree},
    },
    shared::test_utils::TestAppBuilder,
};

/// Canonical test tree — the simplest observable payload.
pub(super) fn speed_boost_tree(multiplier: f32) -> Tree {
    Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
        multiplier: OrderedFloat(multiplier),
    }))
}

/// Configures the `EffectV3Systems` set ordering on a test app, mirroring
/// `EffectV3Plugin::build`.
fn configure_effect_v3_sets(app: &mut App) {
    app.configure_sets(
        FixedUpdate,
        (
            EffectV3Systems::Bridge,
            EffectV3Systems::Tick.after(EffectV3Systems::Bridge),
            EffectV3Systems::Conditions.after(EffectV3Systems::Tick),
        ),
    );
}

/// Test app with all four watcher systems registered in `EffectV3Systems::Bridge`.
pub(super) fn watcher_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_resource::<SpawnStampRegistry>()
        .with_system(
            FixedUpdate,
            stamp_spawned_bolts.in_set(EffectV3Systems::Bridge),
        )
        .with_system(
            FixedUpdate,
            stamp_spawned_cells.in_set(EffectV3Systems::Bridge),
        )
        .with_system(
            FixedUpdate,
            stamp_spawned_walls.in_set(EffectV3Systems::Bridge),
        )
        .with_system(
            FixedUpdate,
            stamp_spawned_breakers.in_set(EffectV3Systems::Bridge),
        )
        .build();
    configure_effect_v3_sets(&mut app);
    app
}

/// Test app with only the bolt watcher registered.
pub(super) fn bolt_only_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_resource::<SpawnStampRegistry>()
        .with_system(
            FixedUpdate,
            stamp_spawned_bolts.in_set(EffectV3Systems::Bridge),
        )
        .build();
    configure_effect_v3_sets(&mut app);
    app
}

/// Test app with only the cell watcher registered.
pub(super) fn cell_only_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_resource::<SpawnStampRegistry>()
        .with_system(
            FixedUpdate,
            stamp_spawned_cells.in_set(EffectV3Systems::Bridge),
        )
        .build();
    configure_effect_v3_sets(&mut app);
    app
}

/// Test app with only the wall watcher registered.
pub(super) fn wall_only_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_resource::<SpawnStampRegistry>()
        .with_system(
            FixedUpdate,
            stamp_spawned_walls.in_set(EffectV3Systems::Bridge),
        )
        .build();
    configure_effect_v3_sets(&mut app);
    app
}

/// Test app with only the breaker watcher registered.
pub(super) fn breaker_only_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_resource::<SpawnStampRegistry>()
        .with_system(
            FixedUpdate,
            stamp_spawned_breakers.in_set(EffectV3Systems::Bridge),
        )
        .build();
    configure_effect_v3_sets(&mut app);
    app
}

/// Overwrites the current `SpawnStampRegistry.entries`.
pub(super) fn set_registry(app: &mut App, entries: Vec<(EntityKind, String, Tree)>) {
    app.world_mut().resource_mut::<SpawnStampRegistry>().entries = entries;
}
