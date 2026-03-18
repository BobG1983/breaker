//! Seeds `TimerUiConfig` from loaded `TimerUiDefaults`.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    screen::loading::resources::DefaultsCollection,
    ui::{TimerUiConfig, TimerUiDefaults},
};

/// Reads the loaded `TimerUiDefaults` asset and inserts `TimerUiConfig`.
pub(crate) fn seed_timer_ui_config(
    collection: Option<Res<DefaultsCollection>>,
    assets: Res<Assets<TimerUiDefaults>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let Some(defaults) = assets.get(&collection.timerui) else {
        return Progress { done: 0, total: 1 };
    };

    commands.insert_resource::<TimerUiConfig>(defaults.clone().into());
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<TimerUiDefaults>()
            .add_systems(Update, seed_timer_ui_config.map(drop));
        app
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<TimerUiConfig>().is_none());
    }

    #[test]
    fn seeds_config_when_asset_loaded() {
        let mut app = test_app();

        let defaults = TimerUiDefaults::default();
        let mut assets = app.world_mut().resource_mut::<Assets<TimerUiDefaults>>();
        let handle = assets.add(defaults);

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: handle,
            cell_types: vec![],
            layouts: vec![],
            archetypes: vec![],
            chipselect: Handle::default(),
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
        });

        app.update();

        assert!(app.world().get_resource::<TimerUiConfig>().is_some());
    }
}
