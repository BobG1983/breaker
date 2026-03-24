//! System to propagate `TimerUiDefaults` asset changes to `TimerUiConfig`.

use bevy::prelude::*;

use crate::{
    screen::loading::resources::DefaultsCollection,
    ui::{TimerUiConfig, TimerUiDefaults},
};

/// Watches for `AssetEvent::Modified` on the timer UI defaults asset and
/// re-seeds `TimerUiConfig` from the updated asset data.
pub(crate) fn propagate_timer_ui_defaults(
    mut events: MessageReader<AssetEvent<TimerUiDefaults>>,
    collection: Res<DefaultsCollection>,
    assets: Res<Assets<TimerUiDefaults>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.is_modified(collection.timer_ui.id())
            && let Some(defaults) = assets.get(collection.timer_ui.id())
        {
            commands.insert_resource::<TimerUiConfig>(defaults.clone().into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<TimerUiDefaults>()
            .init_resource::<TimerUiConfig>()
            .add_systems(Update, propagate_timer_ui_defaults);
        app
    }

    fn make_collection(timer_ui: Handle<TimerUiDefaults>) -> DefaultsCollection {
        DefaultsCollection {
            bolt: Handle::default(),
            breaker: Handle::default(),
            cell_defaults: Handle::default(),
            playfield: Handle::default(),
            input: Handle::default(),
            main_menu: Handle::default(),
            timer_ui,
            chip_select: Handle::default(),
            cells: vec![],
            nodes: vec![],
            breakers: vec![],
            chips: vec![],
            difficulty: Handle::default(),
        }
    }

    /// After an Added event only (no Modified), `TimerUiConfig` should not change.
    #[test]
    fn config_unchanged_when_no_modified_event() {
        let mut app = test_app();

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<TimerUiDefaults>>();
            assets.add(TimerUiDefaults::default())
        };
        app.world_mut().insert_resource(make_collection(handle));

        app.update();
        app.update();

        let config = app.world().resource::<TimerUiConfig>();
        let default_size = TimerUiConfig::default().font_size;
        assert!(
            (config.font_size - default_size).abs() < f32::EPSILON,
            "config should not change when only an Added event is received"
        );
    }

    /// When the timer UI defaults asset is mutated, `TimerUiConfig` must be
    /// re-seeded with the new values.
    #[test]
    fn config_updated_when_modified_event_fires() {
        let mut app = test_app();

        let new_font_size = 64.0_f32;
        let defaults = TimerUiDefaults {
            font_size: new_font_size,
            ..Default::default()
        };

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<TimerUiDefaults>>();
            assets.add(defaults)
        };
        app.world_mut()
            .insert_resource(make_collection(handle.clone()));

        app.update();
        app.update();

        {
            let mut assets = app.world_mut().resource_mut::<Assets<TimerUiDefaults>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.font_size = new_font_size;
        }

        app.update();
        app.update();

        let config = app.world().resource::<TimerUiConfig>();
        assert!(
            (config.font_size - new_font_size).abs() < f32::EPSILON,
            "TimerUiConfig.font_size should be {new_font_size} after Modified event, got {}",
            config.font_size
        );
    }
}
