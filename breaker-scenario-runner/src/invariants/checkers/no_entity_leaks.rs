use bevy::prelude::*;
use breaker::state::run::node::messages::SpawnNodeComplete;

use crate::{invariants::*, types::InvariantKind};

/// Checks for unexpected entity accumulation over time.
///
/// Waits for [`SpawnNodeComplete`] to fire (all domain spawn systems done),
/// then samples the baseline entity count immediately. Every 120 frames after
/// that, checks if count exceeds 2x baseline.
pub fn check_no_entity_leaks(
    all_entities: Query<Entity>,
    frame: Res<ScenarioFrame>,
    mut spawn_reader: MessageReader<SpawnNodeComplete>,
    mut baseline: ResMut<EntityLeakBaseline>,
    mut log: ResMut<ViolationLog>,
) {
    let count = all_entities.iter().count();

    // When SpawnNodeComplete arrives, all gameplay entities are spawned — sample now.
    for _ in spawn_reader.read() {
        baseline.baseline = Some(count);
    }

    let Some(base) = baseline.baseline else {
        return;
    };

    // Check every 120 frames (~1.9 s at 64 Hz fixed timestep)
    if frame.0.is_multiple_of(120) && count > base * 2 {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::NoEntityLeaks,
            entity: None,
            message: format!(
                "NoEntityLeaks FAIL frame={} count={count} baseline={base} (>{} threshold)",
                frame.0,
                base * 2,
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app_entity_leaks() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .insert_resource(EntityLeakBaseline::default())
            .add_message::<SpawnNodeComplete>()
            .add_systems(FixedUpdate, check_no_entity_leaks);
        app
    }

    #[test]
    fn no_entity_leaks_defers_baseline_until_spawn_complete() {
        let mut app = test_app_entity_leaks();
        // No SpawnNodeComplete message sent — baseline should remain None.
        tick(&mut app);

        let baseline = app.world().resource::<EntityLeakBaseline>();
        assert!(
            baseline.baseline.is_none(),
            "baseline must not be set without SpawnNodeComplete"
        );
    }

    #[test]
    fn no_entity_leaks_sets_baseline_immediately_on_spawn_complete() {
        let mut app = test_app_entity_leaks();

        // Spawn some entities to form the baseline.
        for _ in 0..8 {
            app.world_mut().spawn(Transform::default());
        }

        // Send SpawnNodeComplete.
        app.world_mut()
            .resource_mut::<Messages<SpawnNodeComplete>>()
            .write(SpawnNodeComplete);
        tick(&mut app);

        let baseline = app.world().resource::<EntityLeakBaseline>();
        assert!(
            baseline.baseline.is_some(),
            "baseline must be set on the same tick as SpawnNodeComplete"
        );
        // Baseline should count all entities (8 we spawned + MinimalPlugins internals).
        assert!(
            baseline.baseline.unwrap() >= 8,
            "baseline must include spawned entities"
        );
    }

    #[test]
    fn no_entity_leaks_fires_when_count_exceeds_double_baseline() {
        let mut app = test_app_entity_leaks();
        app.insert_resource(ScenarioFrame(360));
        app.insert_resource(EntityLeakBaseline { baseline: Some(5) });

        // Spawn enough entities to exceed 2x5 = 10
        for _ in 0..15 {
            app.world_mut().spawn(Transform::default());
        }

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0
                .iter()
                .any(|v| v.invariant == InvariantKind::NoEntityLeaks),
            "expected NoEntityLeaks violation when count >> baseline"
        );
    }

    #[test]
    fn no_entity_leaks_does_not_fire_when_count_is_normal() {
        let mut app = test_app_entity_leaks();
        app.insert_resource(ScenarioFrame(360));
        app.insert_resource(EntityLeakBaseline {
            baseline: Some(100),
        });

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no NoEntityLeaks violation when count <= baseline"
        );
    }
}
