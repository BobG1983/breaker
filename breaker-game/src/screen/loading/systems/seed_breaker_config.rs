//! Seeds `BreakerConfig` from loaded `BreakerDefaults`.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    breaker::{BreakerConfig, BreakerDefaults},
    screen::loading::resources::DefaultsCollection,
};

/// Reads the loaded `BreakerDefaults` asset and inserts `BreakerConfig`.
pub(crate) fn seed_breaker_config(
    collection: Option<Res<DefaultsCollection>>,
    assets: Res<Assets<BreakerDefaults>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let Some(defaults) = assets.get(&collection.breaker) else {
        return Progress { done: 0, total: 1 };
    };

    commands.insert_resource::<BreakerConfig>(defaults.clone().into());
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<BreakerDefaults>()
            .add_systems(Update, seed_breaker_config.map(drop));
        app
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<BreakerConfig>().is_none());
    }

    #[test]
    fn seeds_config_when_asset_loaded() {
        let mut app = test_app();

        let defaults = BreakerDefaults::default();
        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefaults>>();
        let handle = assets.add(defaults);

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: handle,
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            cell_types: vec![],
            layouts: vec![],
            archetypes: vec![],
            chipselect: Handle::default(),
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
            difficulty: Handle::default(),
        });

        app.update();

        assert!(app.world().get_resource::<BreakerConfig>().is_some());
    }
}
