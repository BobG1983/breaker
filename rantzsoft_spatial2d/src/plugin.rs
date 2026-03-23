//! `RantzSpatial2dPlugin` — registers all spatial systems.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    components::*,
    draw_layer::DrawLayer,
    propagation::*,
    systems::{
        propagate_position::propagate_position, propagate_rotation::propagate_rotation,
        propagate_scale::propagate_scale, save_previous::save_previous,
    },
};

/// Plugin that registers spatial propagation systems for a given `DrawLayer` type.
///
/// Generic over `D: DrawLayer` so each game can provide its own Z-ordering enum.
pub struct RantzSpatial2dPlugin<D: DrawLayer> {
    _marker: PhantomData<D>,
}

impl<D: DrawLayer> Default for RantzSpatial2dPlugin<D> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<D: DrawLayer> Plugin for RantzSpatial2dPlugin<D> {
    fn build(&self, app: &mut App) {
        app.register_type::<Position2D>()
            .register_type::<Rotation2D>()
            .register_type::<Scale2D>()
            .register_type::<PreviousPosition>()
            .register_type::<PreviousRotation>()
            .register_type::<PreviousScale>()
            .register_type::<InterpolateTransform2D>()
            .register_type::<VisualOffset>()
            .register_type::<PositionPropagation>()
            .register_type::<RotationPropagation>()
            .register_type::<ScalePropagation>()
            .add_systems(FixedFirst, save_previous)
            .add_systems(
                RunFixedMainLoop,
                (propagate_position::<D>, propagate_rotation, propagate_scale)
                    .chain()
                    .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component, Clone, Copy, Debug, Default, Reflect)]
    enum TestDrawLayer {
        #[default]
        A,
        B,
    }

    impl DrawLayer for TestDrawLayer {
        fn z(&self) -> f32 {
            match self {
                Self::A => 0.0,
                Self::B => 1.0,
            }
        }
    }

    // ── Behavior 24: Plugin builds without panic ──

    #[test]
    fn plugin_builds_without_panic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());
        app.update();
        app.update();
    }
}
