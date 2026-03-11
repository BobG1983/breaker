//! Physics plugin registration.

use bevy::prelude::*;

use crate::physics::messages::{BoltHitBreaker, BoltHitCell, BoltLost};

/// Plugin for the physics domain.
///
/// Owns collision detection, quadtree, and collision response systems.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BoltHitBreaker>();
        app.add_message::<BoltHitCell>();
        app.add_message::<BoltLost>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(PhysicsPlugin)
            .update();
    }
}
