//! Wall domain query filters.

use bevy::prelude::*;

use crate::{bolt::components::Bolt, cells::components::Cell, wall::components::Wall};

/// Wall entities for collision queries (excludes bolt and cell for query disjointness).
pub(crate) type CollisionFilterWall = (With<Wall>, Without<Bolt>, Without<Cell>);
