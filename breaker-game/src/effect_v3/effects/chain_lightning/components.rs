//! Chain lightning runtime components.

use std::collections::HashSet;

use bevy::prelude::*;

/// Tracks the state of a chain lightning effect as it arcs between cells.
#[derive(Component, Debug, Clone)]
pub struct ChainLightningChain {
    /// Number of jumps remaining before the chain ends.
    pub remaining_jumps: u32,
    /// Damage dealt per arc.
    pub damage:          f32,
    /// Entities already hit by this chain (prevents revisiting).
    pub hit_set:         HashSet<Entity>,
    /// Current state of the chain (idle or traveling).
    pub state:           ChainState,
    /// Maximum distance the chain can jump between targets.
    pub range:           f32,
    /// Travel speed of the arc visual between targets.
    pub arc_speed:       f32,
    /// Position the chain originated from.
    pub source_pos:      Vec2,
}

/// Visual arc entity traveling between chain lightning targets.
#[derive(Component, Debug)]
pub struct ChainLightningArc;

/// State machine for a single chain lightning arc.
#[derive(Debug, Clone)]
pub enum ChainState {
    /// Waiting to select the next target.
    Idle,
    /// Arc is traveling toward a target.
    ArcTraveling {
        /// Target entity the arc is moving toward.
        target:     Entity,
        /// Position of the target entity.
        target_pos: Vec2,
        /// Entity representing the arc visual.
        arc_entity: Entity,
        /// Current position of the arc visual.
        arc_pos:    Vec2,
    },
}
