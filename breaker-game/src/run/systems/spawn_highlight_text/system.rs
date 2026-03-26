//! Spawns in-game text popups when highlight moments are detected.

use bevy::prelude::*;
use rand::Rng;

use crate::{
    fx::{FadeOut, PunchScale},
    run::{
        components::HighlightPopup, definition::HighlightConfig, messages::HighlightTriggered,
        resources::HighlightKind,
    },
    shared::{CleanupOnNodeExit, GameRng},
};

/// Spawns floating text for each [`HighlightTriggered`] message.
///
/// Each popup is vertically stacked based on its `spawn_order` (existing popup
/// count + index within this frame's messages). Horizontal jitter is applied via
/// the seeded [`GameRng`]. Excess popups beyond `popup_max_visible` are culled
/// by despawning the entity with the smallest `FadeOut.timer`.
pub(crate) fn spawn_highlight_text(
    mut reader: MessageReader<HighlightTriggered>,
    mut commands: Commands,
    config: Res<HighlightConfig>,
    mut rng: ResMut<GameRng>,
    existing_popups: Query<(Entity, &FadeOut), With<HighlightPopup>>,
) {
    let max_visible = config.popup_max_visible as usize;
    let mut existing_count = existing_popups.iter().count();

    let messages: Vec<_> = reader.read().cloned().collect();
    if messages.is_empty() {
        return;
    }

    // How many messages we intend to spawn (capped at max_visible).
    let to_spawn = messages.len().min(max_visible);
    let projected_total = existing_count + to_spawn;

    // Cull oldest existing popups to make room for new ones.
    if projected_total > max_visible {
        let mut existing_sorted: Vec<(Entity, f32)> = existing_popups
            .iter()
            .map(|(entity, fade)| (entity, fade.timer))
            .collect();
        existing_sorted.sort_by(|a, b| a.1.total_cmp(&b.1));

        let mut to_cull = projected_total - max_visible;
        for (entity, _) in &existing_sorted {
            if to_cull == 0 {
                break;
            }
            commands.entity(*entity).despawn();
            to_cull -= 1;
            existing_count -= 1;
        }
    }

    // Calculate how many new popups we can spawn without exceeding max_visible.
    let available_slots = max_visible.saturating_sub(existing_count);

    for (i, msg) in messages.iter().take(available_slots).enumerate() {
        let spawn_order = existing_count + i;

        let order_f32 = f32::from(u16::try_from(spawn_order).unwrap_or(u16::MAX));
        let y = config.popup_base_y + config.popup_vertical_spacing * order_f32;
        let x = rng
            .0
            .random_range(config.popup_jitter_min_x..=config.popup_jitter_max_x);
        let fade_timer =
            config.popup_fade_duration_secs + config.popup_cascade_stagger_secs * order_f32;

        let (text, color) = match msg.kind {
            HighlightKind::ClutchClear => ("CLUTCH CLEAR!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::MassDestruction => ("MASS DESTRUCTION!", Color::srgb(1.0, 0.6, 0.0)),
            HighlightKind::PerfectStreak => ("PERFECT STREAK!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::FastClear => ("SPEED CLEAR!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::FirstEvolution => ("FIRST EVOLUTION!", Color::srgb(1.0, 1.0, 0.0)),
            HighlightKind::NoDamageNode => ("FLAWLESS!", Color::srgb(0.0, 1.0, 0.4)),
            HighlightKind::CloseSave => ("CLOSE SAVE!", Color::srgb(0.0, 1.0, 0.4)),
            HighlightKind::SpeedDemon => ("SPEED DEMON!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::Untouchable => ("UNTOUCHABLE!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::ComboKing => ("COMBO KING!", Color::srgb(1.0, 0.6, 0.0)),
            HighlightKind::PinballWizard => ("PINBALL WIZARD!", Color::srgb(1.0, 0.6, 0.0)),
            HighlightKind::Comeback => ("COMEBACK!", Color::srgb(0.0, 1.0, 0.4)),
            HighlightKind::PerfectNode => ("PERFECT NODE!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::NailBiter => ("NAIL BITER!", Color::srgb(0.0, 1.0, 0.4)),
            HighlightKind::MostPowerfulEvolution => ("DEVASTATING!", Color::srgb(1.0, 0.6, 0.0)),
        };

        commands.spawn((
            Text2d::new(text),
            TextColor(color),
            TextFont::from_font_size(64.0),
            TextLayout::new_with_justify(Justify::Center),
            Transform::from_xyz(x, y, 10.0),
            FadeOut {
                timer: fade_timer,
                duration: config.popup_fade_duration_secs,
            },
            PunchScale {
                timer: config.popup_overshoot_duration_secs,
                duration: config.popup_overshoot_duration_secs,
                overshoot: config.popup_overshoot_scale,
            },
            CleanupOnNodeExit,
            HighlightPopup,
        ));
    }
}
