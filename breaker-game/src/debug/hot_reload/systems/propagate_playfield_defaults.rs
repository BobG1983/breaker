//! System to propagate `PlayfieldDefaults` asset changes to `PlayfieldConfig`.

use bevy::prelude::*;

use crate::{
    screen::loading::resources::DefaultsCollection,
    shared::{PlayfieldConfig, PlayfieldDefaults},
};

/// Watches for `AssetEvent::Modified` on the playfield defaults asset and
/// re-seeds `PlayfieldConfig` from the updated asset data.
pub(crate) fn propagate_playfield_defaults(
    mut events: MessageReader<AssetEvent<PlayfieldDefaults>>,
    collection: Res<DefaultsCollection>,
    assets: Res<Assets<PlayfieldDefaults>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.is_modified(collection.playfield.id())
            && let Some(defaults) = assets.get(collection.playfield.id())
        {
            commands.insert_resource::<PlayfieldConfig>(defaults.clone().into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<PlayfieldDefaults>()
            .init_resource::<PlayfieldConfig>()
            .add_systems(Update, propagate_playfield_defaults);
        app
    }

    fn make_collection(playfield: Handle<PlayfieldDefaults>) -> DefaultsCollection {
        DefaultsCollection {
            bolt: Handle::default(),
            breaker: Handle::default(),
            cell_defaults: Handle::default(),
            playfield,
            input: Handle::default(),
            main_menu: Handle::default(),
            timer_ui: Handle::default(),
            chip_select: Handle::default(),
            cells: vec![],
            nodes: vec![],
            breakers: vec![],
            chips: vec![],
            chip_templates: vec![],
            difficulty: Handle::default(),
        }
    }

    /// After an Added event only (no Modified), `PlayfieldConfig` should not change.
    #[test]
    fn config_unchanged_when_no_modified_event() {
        let mut app = test_app();

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<PlayfieldDefaults>>();
            assets.add(PlayfieldDefaults::default())
        };
        app.world_mut().insert_resource(make_collection(handle));

        app.update();
        app.update();

        let config = app.world().resource::<PlayfieldConfig>();
        let default_width = PlayfieldConfig::default().width;
        assert!(
            (config.width - default_width).abs() < f32::EPSILON,
            "config should not change when only an Added event is received"
        );
    }

    /// When the playfield defaults asset is mutated, `PlayfieldConfig` must be
    /// re-seeded with the new values.
    #[test]
    fn config_updated_when_modified_event_fires() {
        let mut app = test_app();

        let new_width = 600.0_f32;
        let defaults = PlayfieldDefaults {
            width: new_width,
            ..Default::default()
        };

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<PlayfieldDefaults>>();
            assets.add(defaults)
        };
        app.world_mut()
            .insert_resource(make_collection(handle.clone()));

        app.update();
        app.update();

        {
            let mut assets = app.world_mut().resource_mut::<Assets<PlayfieldDefaults>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.width = new_width;
        }

        app.update();
        app.update();

        let config = app.world().resource::<PlayfieldConfig>();
        assert!(
            (config.width - new_width).abs() < f32::EPSILON,
            "PlayfieldConfig.width should be {new_width} after Modified event, got {}",
            config.width
        );
    }
}
