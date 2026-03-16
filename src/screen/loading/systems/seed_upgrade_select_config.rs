//! Seeds `UpgradeSelectConfig` from loaded `UpgradeSelectDefaults`.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::screen::{
    loading::resources::DefaultsCollection,
    upgrade_select::{UpgradeSelectConfig, UpgradeSelectDefaults},
};

/// Reads the loaded `UpgradeSelectDefaults` asset and inserts `UpgradeSelectConfig`.
pub fn seed_upgrade_select_config(
    collection: Option<Res<DefaultsCollection>>,
    assets: Res<Assets<UpgradeSelectDefaults>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let Some(defaults) = assets.get(&collection.upgradeselect) else {
        return Progress { done: 0, total: 1 };
    };

    commands.insert_resource::<UpgradeSelectConfig>(defaults.clone().into());
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::upgrade_select::UpgradeSelectDefaults;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<UpgradeSelectDefaults>();
        app.add_systems(Update, seed_upgrade_select_config.map(drop));
        app
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<UpgradeSelectConfig>().is_none());
    }

    #[test]
    fn seeds_config_when_asset_loaded() {
        let mut app = test_app();

        let defaults = UpgradeSelectDefaults::default();
        let mut assets = app
            .world_mut()
            .resource_mut::<Assets<UpgradeSelectDefaults>>();
        let handle = assets.add(defaults);

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            cell_types: vec![],
            layouts: vec![],
            archetypes: vec![],
            upgradeselect: handle,
        });

        app.update();

        assert!(app.world().get_resource::<UpgradeSelectConfig>().is_some());
    }
}
