use bevy::prelude::*;

use crate::{
    prelude::*,
    state::run::{
        resources::{NodeOutcome, NodeResult},
        run_end::{components::RunEndScreen, systems::spawn_run_end_screen::spawn_run_end_screen},
    },
};

pub(super) fn test_app(result: NodeResult) -> App {
    TestAppBuilder::new()
        .insert_resource(NodeOutcome {
            node_index: 0,
            result,
            ..default()
        })
        .with_system(Update, spawn_run_end_screen)
        .build()
}

pub(super) fn test_app_with_stats(result: NodeResult, stats: RunStats) -> App {
    let mut app = test_app(result);
    app.insert_resource(stats);
    app
}

/// Collects all `Text` component values from the world in spawn order by
/// walking the `RunEndScreen` parent's `Children` tree depth-first. The
/// `Children` component records the explicit child-spawn order, which is
/// the source of truth for "what order the system spawned these in" —
/// independent of archetype layout, entity-id reuse, or generation bumps.
pub(super) fn collect_texts(app: &mut App) -> Vec<String> {
    let world = app.world_mut();
    let Some(root) = world
        .query_filtered::<Entity, With<RunEndScreen>>()
        .iter(world)
        .next()
    else {
        return Vec::new();
    };
    let mut texts = Vec::new();
    collect_text_descending(world, root, &mut texts);
    texts
}

fn collect_text_descending(world: &mut World, entity: Entity, out: &mut Vec<String>) {
    if let Some(text) = world.get::<Text>(entity) {
        out.push(text.0.clone());
    }
    let child_entities: Vec<Entity> = world
        .get::<Children>(entity)
        .map(|children| children.iter().collect())
        .unwrap_or_default();
    for child in child_entities {
        collect_text_descending(world, child, out);
    }
}
