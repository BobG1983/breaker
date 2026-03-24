//! System to propagate `ChipSelectDefaults` asset changes to `ChipSelectConfig`.

use bevy::prelude::*;

use crate::screen::{
    chip_select::{ChipSelectConfig, ChipSelectDefaults},
    loading::resources::DefaultsCollection,
};

/// Watches for `AssetEvent::Modified` on the chip select defaults asset and
/// re-seeds `ChipSelectConfig` from the updated asset data.
pub(crate) fn propagate_chip_select_defaults(
    mut events: MessageReader<AssetEvent<ChipSelectDefaults>>,
    collection: Res<DefaultsCollection>,
    assets: Res<Assets<ChipSelectDefaults>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.is_modified(collection.chip_select.id())
            && let Some(defaults) = assets.get(collection.chip_select.id())
        {
            commands.insert_resource::<ChipSelectConfig>(defaults.clone().into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<ChipSelectDefaults>()
            .init_resource::<ChipSelectConfig>()
            .add_systems(Update, propagate_chip_select_defaults);
        app
    }

    fn make_collection(chip_select: Handle<ChipSelectDefaults>) -> DefaultsCollection {
        DefaultsCollection {
            bolt: Handle::default(),
            breaker: Handle::default(),
            cell_defaults: Handle::default(),
            playfield: Handle::default(),
            input: Handle::default(),
            main_menu: Handle::default(),
            timer_ui: Handle::default(),
            chip_select,
            cells: vec![],
            nodes: vec![],
            breakers: vec![],
            chips: vec![],
            difficulty: Handle::default(),
        }
    }

    /// After an Added event only (no Modified), `ChipSelectConfig` should not change.
    #[test]
    fn config_unchanged_when_no_modified_event() {
        let mut app = test_app();

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<ChipSelectDefaults>>();
            assets.add(ChipSelectDefaults::default())
        };
        app.world_mut().insert_resource(make_collection(handle));

        app.update();
        app.update();

        let config = app.world().resource::<ChipSelectConfig>();
        let default_timer = ChipSelectConfig::default().timer_secs;
        assert!(
            (config.timer_secs - default_timer).abs() < f32::EPSILON,
            "config should not change when only an Added event is received"
        );
    }

    /// When the chip select defaults asset is mutated, `ChipSelectConfig` must be
    /// re-seeded with the new values.
    #[test]
    fn config_updated_when_modified_event_fires() {
        let mut app = test_app();

        let new_timer = 20.0_f32;
        let defaults = ChipSelectDefaults {
            timer_secs: new_timer,
            ..Default::default()
        };

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<ChipSelectDefaults>>();
            assets.add(defaults)
        };
        app.world_mut()
            .insert_resource(make_collection(handle.clone()));

        app.update();
        app.update();

        {
            let mut assets = app.world_mut().resource_mut::<Assets<ChipSelectDefaults>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.timer_secs = new_timer;
        }

        app.update();
        app.update();

        let config = app.world().resource::<ChipSelectConfig>();
        assert!(
            (config.timer_secs - new_timer).abs() < f32::EPSILON,
            "ChipSelectConfig.timer_secs should be {new_timer} after Modified event, got {}",
            config.timer_secs
        );
    }
}
