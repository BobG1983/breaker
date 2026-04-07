//! System to launch a serving bolt when the player presses the bump button.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_spatial2d::queries::SpatialData;

use crate::{
    bolt::{components::*, filters::ServingFilter, queries::apply_velocity_formula},
    effect::effects::speed_boost::ActiveSpeedBoosts,
    input::resources::{GameAction, InputActions},
    shared::GameRng,
};

/// Launches the bolt when the player activates bump.
///
/// Removes [`BoltServing`] and sets the launch velocity using a random
/// angle within the bolt's [`BoltAngleSpread`]. Only affects bolts that
/// are currently serving.
pub(crate) fn launch_bolt(
    actions: Res<InputActions>,
    mut commands: Commands,
    mut rng: ResMut<GameRng>,
    mut query: Query<
        (
            Entity,
            SpatialData,
            Option<&ActiveSpeedBoosts>,
            &BoltAngleSpread,
        ),
        ServingFilter,
    >,
) {
    if !actions.active(GameAction::Bump) {
        return;
    }

    for (entity, mut spatial, boosts, angle_spread) in &mut query {
        let angle = rng.0.random_range(-angle_spread.0..=angle_spread.0);
        // Set direction only; speed is applied by the velocity formula
        spatial.velocity.0 = Vec2::new(angle.sin(), angle.cos());

        // Apply the canonical velocity formula
        apply_velocity_formula(&mut spatial, boosts);

        commands.entity(entity).remove::<BoltServing>();
    }
}
