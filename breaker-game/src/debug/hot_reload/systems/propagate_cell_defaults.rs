//! System to propagate `CellDefaults` asset changes to `CellConfig`.

use bevy::prelude::*;

use crate::{
    cells::{CellConfig, CellDefaults},
    screen::loading::resources::DefaultsCollection,
};

/// Watches for `AssetEvent::Modified` on the cells defaults asset and
/// re-seeds `CellConfig` from the updated asset data.
pub(crate) fn propagate_cell_defaults(
    mut events: MessageReader<AssetEvent<CellDefaults>>,
    collection: Res<DefaultsCollection>,
    assets: Res<Assets<CellDefaults>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.is_modified(collection.cells.id())
            && let Some(defaults) = assets.get(collection.cells.id())
        {
            commands.insert_resource::<CellConfig>(defaults.clone().into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<CellDefaults>();
        app.init_resource::<CellConfig>();
        app.add_systems(Update, propagate_cell_defaults);
        app
    }

    fn make_collection(cells: Handle<CellDefaults>) -> DefaultsCollection {
        DefaultsCollection {
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells,
            playfield: Handle::default(),
            input: Handle::default(),
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

    /// After an Added event only (no Modified), `CellConfig` should not change.
    #[test]
    fn config_unchanged_when_no_modified_event() {
        let mut app = test_app();

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<CellDefaults>>();
            assets.add(CellDefaults::default())
        };
        app.world_mut().insert_resource(make_collection(handle));

        app.update();
        app.update();

        let config = app.world().resource::<CellConfig>();
        let default_width = CellConfig::default().width;
        assert!(
            (config.width - default_width).abs() < f32::EPSILON,
            "config should not change when only an Added event is received"
        );
    }

    /// When the cell defaults asset is mutated, `CellConfig` must be re-seeded
    /// with the new values.
    #[test]
    fn config_updated_when_modified_event_fires() {
        let mut app = test_app();

        let new_width = 80.0_f32;
        let defaults = CellDefaults {
            width: new_width,
            ..Default::default()
        };

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<CellDefaults>>();
            assets.add(defaults)
        };
        app.world_mut()
            .insert_resource(make_collection(handle.clone()));

        app.update();
        app.update();

        {
            let mut assets = app.world_mut().resource_mut::<Assets<CellDefaults>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.width = new_width;
        }

        app.update();
        app.update();

        let config = app.world().resource::<CellConfig>();
        assert!(
            (config.width - new_width).abs() < f32::EPSILON,
            "CellConfig.width should be {new_width} after Modified event, got {}",
            config.width
        );
    }
}
