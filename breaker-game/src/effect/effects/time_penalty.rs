//! Time penalty effect handler — observer that translates event into a message.

use bevy::prelude::*;

use crate::{
    chips::definition::TriggerChain,
    effect::{effects::shield::ShieldActive, events::EffectFired},
    run::node::messages::ApplyTimePenalty,
};

/// Observer that handles time penalty — writes [`ApplyTimePenalty`] message.
/// Skips when any entity has [`ShieldActive`].
pub(crate) fn handle_time_penalty(
    trigger: On<EffectFired>,
    mut writer: MessageWriter<ApplyTimePenalty>,
    shield_query: Query<(), With<ShieldActive>>,
) {
    let TriggerChain::TimePenalty { seconds } = &trigger.event().effect else {
        return;
    };
    if !shield_query.is_empty() {
        return;
    }
    writer.write(ApplyTimePenalty { seconds: *seconds });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct CapturedApplyTimePenalty(Vec<f32>);

    fn capture_apply(
        mut reader: MessageReader<ApplyTimePenalty>,
        mut captured: ResMut<CapturedApplyTimePenalty>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.seconds);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ApplyTimePenalty>()
            .init_resource::<CapturedApplyTimePenalty>()
            .add_observer(handle_time_penalty)
            .add_systems(FixedUpdate, capture_apply);
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
    fn handle_time_penalty_sends_apply_message() {
        let mut app = test_app();

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::TimePenalty { seconds: 5.0 },
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedApplyTimePenalty>();
        assert_eq!(captured.0.len(), 1, "should write one ApplyTimePenalty");
        assert!(
            (captured.0[0] - 5.0).abs() < f32::EPSILON,
            "ApplyTimePenalty.seconds should be 5.0, got {}",
            captured.0[0]
        );
    }

    #[test]
    fn non_time_penalty_effect_does_not_send_message() {
        let mut app = test_app();

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::LoseLife,
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedApplyTimePenalty>();
        assert_eq!(
            captured.0.len(),
            0,
            "LoseLife effect should not produce ApplyTimePenalty (self-selection)"
        );
    }

    // =========================================================================
    // Shield blocking tests
    // =========================================================================

    #[test]
    fn time_penalty_skips_when_shield_active_present() {
        use crate::effect::effects::shield::ShieldActive;

        let mut app = test_app();
        // Spawn an entity with ShieldActive so the handler can detect it
        app.world_mut().spawn(ShieldActive { remaining: 3.0 });

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::TimePenalty { seconds: 5.0 },
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedApplyTimePenalty>();
        assert_eq!(
            captured.0.len(),
            0,
            "TimePenalty should be blocked when ShieldActive is present, but {} messages emitted",
            captured.0.len()
        );
    }

    #[test]
    fn time_penalty_works_when_no_shield_active() {
        let mut app = test_app();
        // No ShieldActive present

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::TimePenalty { seconds: 5.0 },
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedApplyTimePenalty>();
        assert_eq!(
            captured.0.len(),
            1,
            "TimePenalty without ShieldActive should emit ApplyTimePenalty"
        );
        assert!(
            (captured.0[0] - 5.0).abs() < f32::EPSILON,
            "ApplyTimePenalty.seconds should be 5.0, got {}",
            captured.0[0]
        );
    }
}
