//! Syncs the breaker's `Scale2D` to its effective dimensions each frame.
//!
//! Replaces `width_boost_visual` — applies `ActiveSizeBoosts` to both
//! width AND height (the old system only applied boost to width).

use bevy::prelude::*;

use crate::{
    breaker::{components::Breaker, queries::SyncBreakerScaleData},
    effect::effects::size_boost::ActiveSizeBoosts,
    shared::size::{ClampRange, effective_size},
};

/// Sets the breaker's [`Scale2D`] to reflect its effective dimensions.
///
/// Reads `BaseWidth`, `BaseHeight`, optional `ActiveSizeBoosts`,
/// optional `NodeScalingFactor`, and optional min/max constraint
/// components. Delegates math to [`effective_size`].
pub(crate) fn sync_breaker_scale(mut query: Query<SyncBreakerScaleData, With<Breaker>>) {
    for mut data in &mut query {
        let size = effective_size(
            data.base_width.0,
            data.base_height.0,
            data.size_boosts.map_or(1.0, ActiveSizeBoosts::multiplier),
            data.node_scale.map_or(1.0, |s| s.0),
            ClampRange {
                min: data.min_w.map(|m| m.0),
                max: data.max_w.map(|m| m.0),
            },
            ClampRange {
                min: data.min_h.map(|m| m.0),
                max: data.max_h.map(|m| m.0),
            },
        );
        data.scale.x = size.x;
        data.scale.y = size.y;
    }
}
