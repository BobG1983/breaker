//! Cells domain query filters.

use bevy::prelude::*;

use crate::{bolt::components::Bolt, cells::components::Cell, wall::components::Wall};

/// Cell entities for collision queries (excludes bolt and wall for query disjointness).
pub(crate) type CollisionFilterCell = (With<Cell>, Without<Bolt>, Without<Wall>);
