//! Life-lost effect handler — observer, component, and HUD.

use bevy::prelude::*;

use crate::{
    effect::{definition::EffectTarget, effects::shield::ShieldActive},
    run::messages::RunLost,
    ui::components::StatusPanel,
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a lose-life effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct LoseLifeFired {
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Number of remaining lives on the breaker entity.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct LivesCount(pub u32);

/// Marker for the lives HUD display entity.
#[derive(Component, Debug)]
pub(crate) struct LivesDisplay;

/// Observer that handles life loss — decrements `LivesCount`, sends [`RunLost`]
/// when lives reach zero. Skips entities that have [`ShieldActive`].
pub(crate) fn handle_life_lost(
    trigger: On<LoseLifeFired>,
    mut lives_query: Query<(&mut LivesCount, Option<&ShieldActive>)>,
    mut writer: MessageWriter<RunLost>,
) {
    for (mut lives, shield) in &mut lives_query {
        if shield.is_some() {
            continue;
        }
        if lives.0 == 0 {
            continue;
        }
        lives.0 -= 1;

        if lives.0 == 0 {
            writer.write(RunLost);
        }
    }
}

/// Spawns the lives display as a child of the [`StatusPanel`].
pub(crate) fn spawn_lives_display(
    mut commands: Commands,
    lives_query: Query<&LivesCount>,
    existing: Query<(), With<LivesDisplay>>,
    status_panel: Query<Entity, With<StatusPanel>>,
) {
    if !existing.is_empty() {
        return;
    }

    let Ok(lives) = lives_query.single() else {
        return;
    };

    let Ok(panel) = status_panel.single() else {
        return;
    };

    commands.entity(panel).with_children(|parent| {
        parent
            .spawn((
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ))
            .with_child((
                LivesDisplay,
                Text::new(format_lives(lives.0)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
    });
}

/// Updates the lives display text to match the current `LivesCount`.
pub(crate) fn update_lives_display(
    lives_query: Query<&LivesCount>,
    mut display_query: Query<&mut Text, With<LivesDisplay>>,
) {
    let Ok(lives) = lives_query.single() else {
        return;
    };

    for mut text in &mut display_query {
        text.0 = format_lives(lives.0);
    }
}

/// Returns the HUD display string for the current life count.
///
/// Format: `"Lives: {count}"`. Both `spawn_lives_display` and
/// `update_lives_display` call this function — change the format here only.
fn format_lives(count: u32) -> String {
    format!("Lives: {count}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct CapturedRunLost(u32);

    fn capture_run_lost(mut reader: MessageReader<RunLost>, mut captured: ResMut<CapturedRunLost>) {
        for _msg in reader.read() {
            captured.0 += 1;
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RunLost>()
            .init_resource::<CapturedRunLost>()
            .add_observer(handle_life_lost)
            .add_systems(FixedUpdate, capture_run_lost);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn lose_life_decrements_count() {
        use crate::effect::typed_events::LoseLifeFired;

        let mut app = test_app();
        let entity = app.world_mut().spawn(LivesCount(3)).id();

        app.world_mut().commands().trigger(LoseLifeFired {
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, 2,
            "LoseLife effect should decrement LivesCount from 3 to 2"
        );
    }

    #[test]
    fn lose_last_life_sends_run_lost() {
        use crate::effect::typed_events::LoseLifeFired;

        let mut app = test_app();
        app.world_mut().spawn(LivesCount(1));

        app.world_mut().commands().trigger(LoseLifeFired {
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedRunLost>();
        assert_eq!(captured.0, 1, "should send RunLost when lives reach zero");
    }

    // =========================================================================
    // Shield blocking tests
    // =========================================================================

    #[test]
    fn lose_life_skips_when_shield_active_present() {
        use crate::effect::{effects::shield::ShieldActive, typed_events::LoseLifeFired};

        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((LivesCount(3), ShieldActive { remaining: 3.0 }))
            .id();

        app.world_mut().commands().trigger(LoseLifeFired {
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, 3,
            "LoseLife should be blocked when ShieldActive is present, but lives changed to {}",
            lives.0
        );
    }

    #[test]
    fn lose_life_works_when_no_shield_active() {
        use crate::effect::typed_events::LoseLifeFired;

        let mut app = test_app();
        let entity = app.world_mut().spawn(LivesCount(3)).id();

        app.world_mut().commands().trigger(LoseLifeFired {
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, 2,
            "LoseLife without ShieldActive should decrement lives from 3 to 2"
        );
    }

    #[test]
    fn shield_protects_last_life() {
        use crate::effect::{effects::shield::ShieldActive, typed_events::LoseLifeFired};

        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((LivesCount(1), ShieldActive { remaining: 2.0 }))
            .id();

        app.world_mut().commands().trigger(LoseLifeFired {
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, 1,
            "ShieldActive should protect last life, but lives changed to {}",
            lives.0
        );

        let captured = app.world().resource::<CapturedRunLost>();
        assert_eq!(
            captured.0, 0,
            "RunLost should NOT be sent when shield protects last life"
        );
    }

    // =========================================================================
    // B12c: handle_life_lost observes LoseLifeFired (not EffectFired) (behavior 21/16)
    // =========================================================================

    fn typed_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RunLost>()
            .init_resource::<CapturedRunLost>()
            .add_observer(handle_life_lost)
            .add_systems(FixedUpdate, capture_run_lost);
        app
    }

    #[test]
    fn lose_life_fired_decrements_count_via_typed_event() {
        use crate::effect::typed_events::LoseLifeFired;

        let mut app = typed_test_app();
        let entity = app.world_mut().spawn(LivesCount(3)).id();

        app.world_mut().commands().trigger(LoseLifeFired {
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, 2,
            "LoseLifeFired typed event should decrement LivesCount from 3 to 2"
        );
    }

    #[test]
    fn lose_life_fired_sends_run_lost_when_last_life() {
        use crate::effect::typed_events::LoseLifeFired;

        let mut app = typed_test_app();
        app.world_mut().spawn(LivesCount(1));

        app.world_mut().commands().trigger(LoseLifeFired {
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedRunLost>();
        assert_eq!(
            captured.0, 1,
            "LoseLifeFired should send RunLost when lives reach zero"
        );
    }

    #[test]
    fn lose_life_fired_skips_when_shield_active() {
        use crate::effect::{effects::shield::ShieldActive, typed_events::LoseLifeFired};

        let mut app = typed_test_app();
        let entity = app
            .world_mut()
            .spawn((LivesCount(3), ShieldActive { remaining: 3.0 }))
            .id();

        app.world_mut().commands().trigger(LoseLifeFired {
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, 3,
            "LoseLifeFired should be blocked when ShieldActive is present"
        );
    }
}
