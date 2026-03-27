use bevy::prelude::*;

use super::super::dispatch_chip_effects;
use crate::{
    chips::resources::ChipCatalog,
    effect::effects::{
        bolt_size_boost::handle_bolt_size_boost, bolt_speed_boost::handle_bolt_speed_boost,
        breaker_speed_boost::handle_breaker_speed_boost,
        bump_force_boost::handle_bump_force_boost, chain_hit::handle_chain_hit,
        damage_boost::handle_damage_boost, piercing::handle_piercing,
        tilt_control_boost::handle_tilt_control_boost, width_boost::handle_width_boost,
    },
    ui::messages::ChipSelected,
};

/// Resource holding an optional [`ChipSelected`] message to be sent once.
#[derive(Resource)]
pub(super) struct PendingChipSelected(pub Option<ChipSelected>);

/// Helper system: writes the pending message once, then clears it.
pub(super) fn enqueue_chip_selected(
    mut pending: ResMut<PendingChipSelected>,
    mut writer: MessageWriter<ChipSelected>,
) {
    if let Some(msg) = pending.0.take() {
        writer.write(msg);
    }
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<ChipSelected>()
        .init_resource::<ChipCatalog>()
        .add_systems(
            Update,
            (enqueue_chip_selected, dispatch_chip_effects).chain(),
        )
        .add_observer(handle_piercing)
        .add_observer(handle_damage_boost)
        .add_observer(handle_bolt_speed_boost)
        .add_observer(handle_chain_hit)
        .add_observer(handle_bolt_size_boost)
        .add_observer(handle_width_boost)
        .add_observer(handle_breaker_speed_boost)
        .add_observer(handle_bump_force_boost)
        .add_observer(handle_tilt_control_boost);
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn send_chip_selected(app: &mut App, name: &str) {
    app.insert_resource(PendingChipSelected(Some(ChipSelected {
        name: name.to_owned(),
    })));
}
