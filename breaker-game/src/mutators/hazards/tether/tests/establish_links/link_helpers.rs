//! Local link-count helpers for the `establish_links` suite.

use std::collections::HashMap;

use bevy::prelude::*;

use super::super::super::system::TetherLink;

/// Counts cells carrying `TetherLink`, divided by 2.
pub(super) fn count_linked_pairs(app: &mut App) -> usize {
    let mut q = app.world_mut().query_filtered::<Entity, With<TetherLink>>();
    let total = q.iter(app.world()).count();
    total / 2
}

/// Builds a map from entity → `TetherLink.partner`. Used to check mutual
/// bidirectional link integrity.
pub(super) fn links_map(app: &mut App) -> HashMap<Entity, Entity> {
    let mut q = app.world_mut().query::<(Entity, &TetherLink)>();
    q.iter(app.world())
        .map(|(e, link)| (e, link.partner))
        .collect()
}
