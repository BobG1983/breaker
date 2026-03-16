//! Side panel chrome — left (Augments) and right (Status) flanking the playfield.

use bevy::prelude::*;

use crate::{shared::CleanupOnRunEnd, ui::components::SidePanels};

/// Spawns the full-screen flex row with left and right side panels.
pub fn spawn_side_panels(mut commands: Commands, existing: Query<(), With<SidePanels>>) {
    if !existing.is_empty() {
        return;
    }
    commands
        .spawn((
            SidePanels,
            CleanupOnRunEnd,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .with_children(|root| {
            // Left panel — Augments
            root.spawn((
                Node {
                    width: Val::Percent(12.5),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    border: UiRect::right(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.01, 0.01, 0.03, 0.95)),
                BorderColor::all(Color::srgb(0.1, 0.3, 0.5)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("AUGMENTS"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.2, 0.5, 0.7)),
                ));
                panel.spawn((
                    Text::new("—"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.3, 0.3, 0.4, 1.0)),
                ));
            });

            // Center spacer
            root.spawn(Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                ..default()
            });

            // Right panel — Status
            root.spawn((
                Node {
                    width: Val::Percent(12.5),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    border: UiRect::left(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.01, 0.01, 0.03, 0.95)),
                BorderColor::all(Color::srgb(0.1, 0.3, 0.5)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("STATUS"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.2, 0.5, 0.7)),
                ));
            });
        });
}
