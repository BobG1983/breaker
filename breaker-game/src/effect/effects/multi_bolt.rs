//! `MultiBolt` effect handler — spawns additional bolts on trigger.
//!
//! Observes [`MultiBoltFired`] and writes [`SpawnAdditionalBolt`]
//! messages equal to `base_count + (stacks - 1) * count_per_level`.

use bevy::prelude::*;

use crate::{bolt::messages::SpawnAdditionalBolt, effect::definition::EffectTarget};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a multi-bolt effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct MultiBoltFired {
    /// Base number of extra bolts to spawn.
    pub base_count: u32,
    /// Additional bolts per stack beyond the first.
    pub count_per_level: u32,
    /// Current stack count.
    pub stacks: u32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Observer: handles multi-bolt effect — writes multiple [`SpawnAdditionalBolt`] messages.
///
/// Spawn count formula: `base_count + (stacks.saturating_sub(1)) * count_per_level`.
pub(crate) fn handle_multi_bolt(
    trigger: On<MultiBoltFired>,
    mut writer: MessageWriter<SpawnAdditionalBolt>,
) {
    let event = trigger.event();

    let total = event.base_count + event.stacks.saturating_sub(1) * event.count_per_level;
    for _ in 0..total {
        writer.write(SpawnAdditionalBolt {
            source_chip: event.source_chip.clone(),
            lifespan: None,
            inherit: false,
        });
    }
}

/// Registers all observers and systems for the multi-bolt effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_multi_bolt);
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Test infrastructure ---

    #[derive(Resource, Default)]
    struct CapturedSpawnBolt(Vec<SpawnAdditionalBolt>);

    fn capture_spawn(
        mut reader: MessageReader<SpawnAdditionalBolt>,
        mut captured: ResMut<CapturedSpawnBolt>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<SpawnAdditionalBolt>()
            .init_resource::<CapturedSpawnBolt>()
            .add_observer(handle_multi_bolt)
            .add_systems(FixedUpdate, capture_spawn);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn trigger_multi_bolt(
        app: &mut App,
        base_count: u32,
        count_per_level: u32,
        stacks: u32,
        source_chip: Option<String>,
    ) {
        use crate::effect::typed_events::MultiBoltFired;

        app.world_mut().commands().trigger(MultiBoltFired {
            base_count,
            count_per_level,
            stacks,
            targets: vec![],
            source_chip,
        });
        app.world_mut().flush();
        tick(app);
    }

    // --- Tests ---

    #[test]
    fn multi_bolt_stacks_one_spawns_base_count() {
        let mut app = test_app();

        trigger_multi_bolt(&mut app, 2, 1, 1, Some("test_chip".to_owned()));

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0.len(),
            2,
            "MultiBolt base_count=2, stacks=1 should spawn 2 bolts, got {}",
            captured.0.len()
        );
        for msg in &captured.0 {
            assert_eq!(
                msg.source_chip,
                Some("test_chip".to_owned()),
                "each SpawnAdditionalBolt should carry source_chip"
            );
        }
    }

    #[test]
    fn multi_bolt_stacking_formula_base_plus_extra_stacks_times_per_level() {
        let mut app = test_app();

        // Formula: 2 + (3-1)*3 = 2 + 6 = 8
        trigger_multi_bolt(&mut app, 2, 3, 3, None);

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0.len(),
            8,
            "MultiBolt base=2, per_level=3, stacks=3 should spawn 8 bolts (2 + 2*3), got {}",
            captured.0.len()
        );
    }

    #[test]
    fn multi_bolt_works_without_bolt_entity() {
        let mut app = test_app();

        trigger_multi_bolt(&mut app, 1, 0, 1, None);

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0.len(),
            1,
            "MultiBolt base=1 with bolt=None should still spawn 1 bolt, got {}",
            captured.0.len()
        );
    }

    #[test]
    fn multi_bolt_passes_source_chip_none_through() {
        let mut app = test_app();

        trigger_multi_bolt(&mut app, 1, 0, 1, None);

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(
            captured.0[0].source_chip, None,
            "SpawnAdditionalBolt should carry source_chip: None"
        );
    }

    #[test]
    fn multi_bolt_stacks_zero_uses_saturating_sub() {
        let mut app = test_app();

        // stacks=0: saturating_sub(1) = 0, so total = base_count = 2
        trigger_multi_bolt(&mut app, 2, 3, 0, None);

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0.len(),
            2,
            "MultiBolt stacks=0 should use saturating_sub, spawning base_count=2 only, got {}",
            captured.0.len()
        );
    }
}
