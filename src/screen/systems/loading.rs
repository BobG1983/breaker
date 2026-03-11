//! Loading state — asset collection, config seeding, and loading screen UI.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_progress::prelude::*;

use crate::bolt::BoltConfig;
use crate::breaker::BreakerConfig;
use crate::cells::CellConfig;
use crate::physics::PhysicsConfig;
use crate::screen::defaults::{
    BoltDefaults, BreakerDefaults, CellDefaults, PhysicsDefaults, PlayfieldDefaults,
};
use crate::shared::PlayfieldConfig;

/// Asset collection for all defaults — automatically loaded during
/// [`GameState::Loading`] by `bevy_asset_loader`.
#[derive(AssetCollection, Resource)]
pub struct DefaultsCollection {
    /// Handle for playfield defaults.
    #[asset(path = "config/defaults.playfield.ron")]
    pub playfield: Handle<PlayfieldDefaults>,
    /// Handle for bolt defaults.
    #[asset(path = "config/defaults.bolt.ron")]
    pub bolt: Handle<BoltDefaults>,
    /// Handle for breaker defaults.
    #[asset(path = "config/defaults.breaker.ron")]
    pub breaker: Handle<BreakerDefaults>,
    /// Handle for cells defaults.
    #[asset(path = "config/defaults.cells.ron")]
    pub cells: Handle<CellDefaults>,
    /// Handle for physics defaults.
    #[asset(path = "config/defaults.physics.ron")]
    pub physics: Handle<PhysicsDefaults>,
}

/// Reads loaded `*Defaults` assets and inserts the corresponding `*Config`
/// resources. Returns [`Progress`] to block the loading state transition
/// until seeding is complete.
// Each `Assets<*Defaults>` store is a required Bevy system param — no way
// to reduce the count without a custom `SystemParam` that would add more
// complexity than it removes.
#[allow(clippy::too_many_arguments)]
pub fn seed_configs_from_defaults(
    collection: Option<Res<DefaultsCollection>>,
    playfield_assets: Res<Assets<PlayfieldDefaults>>,
    bolt_assets: Res<Assets<BoltDefaults>>,
    breaker_assets: Res<Assets<BreakerDefaults>>,
    cell_assets: Res<Assets<CellDefaults>>,
    physics_assets: Res<Assets<PhysicsDefaults>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    // All assets must be loaded before we can seed
    let Some(playfield) = playfield_assets.get(&collection.playfield) else {
        return Progress { done: 0, total: 1 };
    };
    let Some(bolt) = bolt_assets.get(&collection.bolt) else {
        return Progress { done: 0, total: 1 };
    };
    let Some(breaker) = breaker_assets.get(&collection.breaker) else {
        return Progress { done: 0, total: 1 };
    };
    let Some(cells) = cell_assets.get(&collection.cells) else {
        return Progress { done: 0, total: 1 };
    };
    let Some(physics) = physics_assets.get(&collection.physics) else {
        return Progress { done: 0, total: 1 };
    };

    commands.insert_resource::<PlayfieldConfig>(playfield.clone().into());
    commands.insert_resource::<BoltConfig>(bolt.clone().into());
    commands.insert_resource::<BreakerConfig>(breaker.clone().into());
    commands.insert_resource::<CellConfig>(cells.clone().into());
    commands.insert_resource::<PhysicsConfig>(physics.clone().into());

    *seeded = true;
    Progress { done: 1, total: 1 }
}

/// Marker component for loading screen entities.
#[derive(Component)]
pub struct LoadingScreen;

/// Marker for the loading progress bar inner fill.
#[derive(Component)]
pub struct LoadingBarFill;

/// Marker for the loading progress text.
#[derive(Component)]
pub struct LoadingProgressText;

/// Width of the loading bar background in pixels.
const LOADING_BAR_WIDTH: f32 = 400.0;

/// Height of the loading bar in pixels.
const LOADING_BAR_HEIGHT: f32 = 24.0;

/// Spawns the loading screen UI.
pub fn spawn_loading_screen(mut commands: Commands) {
    commands
        .spawn((
            LoadingScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Progress text
            parent.spawn((
                LoadingProgressText,
                Text::new("Loading..."),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Bar background
            parent
                .spawn(Node {
                    width: Val::Px(LOADING_BAR_WIDTH),
                    height: Val::Px(LOADING_BAR_HEIGHT),
                    ..default()
                })
                .insert(BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.15)))
                .with_children(|bar_bg| {
                    // Bar fill
                    bar_bg.spawn((
                        LoadingBarFill,
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.8, 1.0)),
                    ));
                });
        });
}

/// Updates the loading bar width and text based on global progress.
pub fn update_loading_bar(
    progress: Res<ProgressTracker<crate::shared::GameState>>,
    mut bar_query: Query<&mut Node, With<LoadingBarFill>>,
    mut text_query: Query<&mut Text, With<LoadingProgressText>>,
) {
    let global = progress.get_global_progress();
    #[allow(clippy::cast_precision_loss)]
    let ratio = if global.total > 0 {
        global.done as f32 / global.total as f32
    } else {
        0.0
    };

    for mut node in &mut bar_query {
        node.width = Val::Percent(ratio * 100.0);
    }

    for mut text in &mut text_query {
        **text = format!("Loading... {}/{}", global.done, global.total);
    }
}

/// Despawns all loading screen entities.
pub fn cleanup_loading_screen(mut commands: Commands, query: Query<Entity, With<LoadingScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
