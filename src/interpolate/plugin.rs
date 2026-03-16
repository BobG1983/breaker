//! Interpolation plugin registration.

use bevy::prelude::*;

use super::systems::{interpolate_transform, restore_authoritative, store_authoritative};

/// Plugin for visual interpolation between `FixedUpdate` ticks.
///
/// Smooths entity rendering by lerping between previous and current
/// authoritative positions using `Time<Fixed>::overstep_fraction()`.
///
/// Entities opt in by adding [`InterpolateTransform`] and [`PhysicsTranslation`]
/// components.
pub struct InterpolatePlugin;

impl Plugin for InterpolatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedFirst, restore_authoritative)
            .add_systems(FixedPostUpdate, store_authoritative)
            .add_systems(PostUpdate, interpolate_transform);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(InterpolatePlugin)
            .update();
    }
}
