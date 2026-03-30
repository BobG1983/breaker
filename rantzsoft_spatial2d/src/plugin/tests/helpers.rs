use bevy::prelude::*;

use crate::draw_layer::DrawLayer;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub(super) enum TestDrawLayer {
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

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
