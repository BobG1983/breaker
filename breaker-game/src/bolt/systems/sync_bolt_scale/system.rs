//! Syncs bolt `Scale2D` to effective radius each frame.
//!
//! Replaces `bolt_scale_visual` -- applies `ActiveSizeBoosts`, `NodeScalingFactor`,
//! and optional min/max radius constraints via `effective_radius`.

use bevy::prelude::*;

use crate::{bolt::queries::SyncBoltScaleData, prelude::*};

/// Sets bolt [`Scale2D`] based on [`BaseRadius`], optional [`ActiveSizeBoosts`],
/// optional [`NodeScalingFactor`], and optional min/max radius constraints.
pub(crate) fn sync_bolt_scale(
    mut query: Query<SyncBoltScaleData, (With<Bolt>, Without<Birthing>)>,
) {
    use crate::shared::size::{ClampRange, effective_radius};

    for mut data in &mut query {
        let boost = data
            .size_boosts
            .map_or(1.0, crate::effect_v3::stacking::EffectStack::aggregate);
        let node = data.node_scale.map_or(1.0, |s| s.0);
        let range = ClampRange {
            min: data.min_radius.map(|m| m.0),
            max: data.max_radius.map(|m| m.0),
        };
        let r = effective_radius(data.base_radius.0, boost, node, range);
        data.scale.x = r;
        data.scale.y = r;
    }
}
