//! Breaker plugin registration.

use bevy::prelude::*;

use crate::breaker::messages::BumpPerformed;

/// Plugin for the breaker domain.
///
/// Owns breaker components, state machine, and bump system.
pub struct BreakerPlugin;

impl Plugin for BreakerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BumpPerformed>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(BreakerPlugin)
            .update();
    }
}
