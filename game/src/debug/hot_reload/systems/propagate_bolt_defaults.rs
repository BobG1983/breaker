//! System to propagate `BoltDefaults` asset changes to `BoltConfig`.

use bevy::prelude::*;

use crate::{
    bolt::{BoltConfig, BoltDefaults},
    screen::loading::resources::DefaultsCollection,
};

/// Watches for `AssetEvent::Modified` on the bolt defaults asset and
/// re-seeds `BoltConfig` from the updated asset data.
pub fn propagate_bolt_defaults(
    mut events: MessageReader<AssetEvent<BoltDefaults>>,
    collection: Res<DefaultsCollection>,
    assets: Res<Assets<BoltDefaults>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.is_modified(collection.bolt.id())
            && let Some(defaults) = assets.get(collection.bolt.id())
        {
            commands.insert_resource::<BoltConfig>(defaults.clone().into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<BoltDefaults>();
        app.init_resource::<BoltConfig>();
        app.add_systems(Update, propagate_bolt_defaults);
        app
    }

    fn make_collection(bolt: Handle<BoltDefaults>) -> DefaultsCollection {
        use crate::{
            breaker::BreakerDefaults,
            cells::CellDefaults,
            input::InputDefaults,
            screen::{chip_select::ChipSelectDefaults, main_menu::MainMenuDefaults},
            shared::PlayfieldDefaults,
            ui::TimerUiDefaults,
        };
        DefaultsCollection {
            bolt,
            breaker: Handle::default(),
            cells: Handle::default(),
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

    /// After adding an asset (Added event only), the config should remain at
    /// its initial value — no Modified event means no propagation.
    #[test]
    fn config_unchanged_when_no_modified_event() {
        let mut app = test_app();

        let initial_speed = 400.0_f32;
        let mut defaults = BoltDefaults::default();
        defaults.base_speed = initial_speed;

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<BoltDefaults>>();
            assets.add(defaults)
        };

        app.world_mut().insert_resource(make_collection(handle));

        // Two updates: PostUpdate flushes Added, First rotates buffer.
        // System runs but sees Added (not Modified) — config must not change.
        app.update();
        app.update();

        let config = app.world().resource::<BoltConfig>();
        assert!(
            (config.base_speed - initial_speed).abs() < f32::EPSILON,
            "config should not change when only an Added event is received"
        );
    }

    /// When the bolt defaults asset is mutated (triggering Modified), the
    /// propagation system must re-seed BoltConfig with the new values.
    #[test]
    fn config_updated_when_modified_event_fires() {
        let mut app = test_app();

        let new_speed = 500.0_f32;
        let mut defaults = BoltDefaults::default();
        defaults.base_speed = new_speed;

        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<BoltDefaults>>();
            assets.add(defaults)
        };

        app.world_mut()
            .insert_resource(make_collection(handle.clone()));

        // Let the Added event be generated and flushed.
        app.update();
        app.update();

        // Trigger a Modified event by taking a mutable reference to the asset.
        {
            let mut assets = app.world_mut().resource_mut::<Assets<BoltDefaults>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.base_speed = new_speed;
        }

        // PostUpdate flushes Modified into the message buffer.
        app.update();
        // First rotates the buffer; Update runs our system.
        app.update();

        let config = app.world().resource::<BoltConfig>();
        assert!(
            (config.base_speed - new_speed).abs() < f32::EPSILON,
            "BoltConfig.base_speed should be {new_speed} after Modified event, got {}",
            config.base_speed
        );
    }

    /// A Modified event for a different asset ID (not the one in DefaultsCollection)
    /// must not update BoltConfig.
    #[test]
    fn config_unchanged_when_modified_event_is_for_different_handle() {
        let mut app = test_app();

        let registered_handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<BoltDefaults>>();
            assets.add(BoltDefaults::default())
        };

        // A second, unregistered handle with a different base_speed.
        let unregistered_handle = {
            let mut defaults = BoltDefaults::default();
            defaults.base_speed = 999.0;
            let mut assets = app.world_mut().resource_mut::<Assets<BoltDefaults>>();
            assets.add(defaults)
        };

        app.world_mut()
            .insert_resource(make_collection(registered_handle));

        app.update();
        app.update();

        // Mutate the unregistered handle — should not affect BoltConfig.
        {
            let mut assets = app.world_mut().resource_mut::<Assets<BoltDefaults>>();
            let asset = assets
                .get_mut(unregistered_handle.id())
                .expect("asset should exist");
            asset.base_speed = 999.0;
        }

        app.update();
        app.update();

        let config = app.world().resource::<BoltConfig>();
        let default_speed = BoltConfig::default().base_speed;
        assert!(
            (config.base_speed - default_speed).abs() < f32::EPSILON,
            "config should not change when Modified event is for a different handle"
        );
    }
}
