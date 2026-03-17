//! System to propagate `InputDefaults` asset changes to `InputConfig`.

use bevy::prelude::*;

use crate::{
    input::{InputConfig, InputDefaults},
    screen::loading::resources::DefaultsCollection,
};

/// Watches for `AssetEvent::Modified` on the input defaults asset and
/// re-seeds `InputConfig` from the updated asset data.
pub fn propagate_input_defaults(
    mut events: MessageReader<AssetEvent<InputDefaults>>,
    collection: Res<DefaultsCollection>,
    assets: Res<Assets<InputDefaults>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.is_modified(collection.input.id())
            && let Some(defaults) = assets.get(collection.input.id())
        {
            commands.insert_resource::<InputConfig>(defaults.clone().into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<InputDefaults>();
        app.init_resource::<InputConfig>();
        app.add_systems(Update, propagate_input_defaults);
        app
    }

    fn make_collection(input: Handle<InputDefaults>) -> DefaultsCollection {
        DefaultsCollection {
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            playfield: Handle::default(),
            input,
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            chipselect: Handle::default(),
            cell_types: vec![],
            layouts: vec![],
            archetypes: vec![],
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
        }
    }

    /// After an Added event only (no Modified), `InputConfig` should not change.
    #[test]
    fn config_unchanged_when_no_modified_event() {
        let mut app = test_app();

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<InputDefaults>>();
            assets.add(InputDefaults::default())
        };
        app.world_mut().insert_resource(make_collection(handle));

        app.update();
        app.update();

        let config = app.world().resource::<InputConfig>();
        let default_window = InputConfig::default().double_tap_window;
        assert!(
            (config.double_tap_window - default_window).abs() < f32::EPSILON,
            "config should not change when only an Added event is received"
        );
    }

    /// When the input defaults asset is mutated, `InputConfig` must be re-seeded
    /// with the new values.
    #[test]
    fn config_updated_when_modified_event_fires() {
        let mut app = test_app();

        let new_window = 0.5_f32;
        let defaults = InputDefaults {
            double_tap_window: new_window,
            ..Default::default()
        };

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<InputDefaults>>();
            assets.add(defaults)
        };
        app.world_mut()
            .insert_resource(make_collection(handle.clone()));

        app.update();
        app.update();

        {
            let mut assets = app.world_mut().resource_mut::<Assets<InputDefaults>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.double_tap_window = new_window;
        }

        app.update();
        app.update();

        let config = app.world().resource::<InputConfig>();
        assert!(
            (config.double_tap_window - new_window).abs() < f32::EPSILON,
            "InputConfig.double_tap_window should be {new_window} after Modified event, got {}",
            config.double_tap_window
        );
    }
}
