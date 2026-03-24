//! Seeds `ChipSelectConfig` from loaded `ChipSelectDefaults`.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::screen::{
    chip_select::{ChipSelectConfig, ChipSelectDefaults},
    loading::resources::DefaultsCollection,
};

/// Reads the loaded `ChipSelectDefaults` asset and inserts `ChipSelectConfig`.
pub(crate) fn seed_chip_select_config(
    collection: Option<Res<DefaultsCollection>>,
    assets: Res<Assets<ChipSelectDefaults>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let Some(defaults) = assets.get(&collection.chip_select) else {
        return Progress { done: 0, total: 1 };
    };

    commands.insert_resource::<ChipSelectConfig>(defaults.clone().into());
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::chip_select::ChipSelectDefaults;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<ChipSelectDefaults>()
            .add_systems(Update, seed_chip_select_config.map(drop));
        app
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<ChipSelectConfig>().is_none());
    }

    #[test]
    fn seeds_config_when_asset_loaded() {
        let mut app = test_app();

        let defaults = ChipSelectDefaults::default();
        let mut assets = app.world_mut().resource_mut::<Assets<ChipSelectDefaults>>();
        let handle = assets.add(defaults);

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cell_defaults: Handle::default(),
            input: Handle::default(),
            main_menu: Handle::default(),
            timer_ui: Handle::default(),
            cells: vec![],
            nodes: vec![],
            breakers: vec![],
            chip_select: handle,
            chips: vec![],
            difficulty: Handle::default(),
        });

        app.update();

        assert!(app.world().get_resource::<ChipSelectConfig>().is_some());
    }

    #[test]
    fn only_seeds_once() {
        let mut app = test_app();

        let defaults = ChipSelectDefaults::default();
        let mut assets = app.world_mut().resource_mut::<Assets<ChipSelectDefaults>>();
        let handle = assets.add(defaults);

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cell_defaults: Handle::default(),
            input: Handle::default(),
            main_menu: Handle::default(),
            timer_ui: Handle::default(),
            cells: vec![],
            nodes: vec![],
            breakers: vec![],
            chip_select: handle,
            chips: vec![],
            difficulty: Handle::default(),
        });

        // First update seeds the config
        app.update();
        assert!(app.world().get_resource::<ChipSelectConfig>().is_some());

        // Remove the resource — if the system re-seeds, it would reappear
        app.world_mut().remove_resource::<ChipSelectConfig>();
        app.update();

        assert!(
            app.world().get_resource::<ChipSelectConfig>().is_none(),
            "system should not re-seed after the first successful seed"
        );
    }
}
