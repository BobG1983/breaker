use bevy::prelude::*;

use crate::{invariants::*, lifecycle::ScenarioConfig, types::InvariantKind};

/// Checks that the bolt count stays within `invariant_params.max_bolt_count`.
///
/// Catches bolt accumulation leaks (e.g. Prism bolts not despawned on loss).
pub fn check_bolt_count_reasonable(
    bolts: Query<Entity, With<ScenarioTagBolt>>,
    config: Res<ScenarioConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let max = config.definition.invariant_params.max_bolt_count;
    let count = bolts.iter().count();
    if count > max {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::BoltCountReasonable,
            entity: None,
            message: format!("BoltCountReasonable FAIL frame={} count={count}", frame.0),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{InputStrategy, InvariantParams, ScenarioDefinition, ScriptedParams};

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn bolt_count_test_app(max_bolt_count: usize) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .insert_resource(ScenarioConfig {
                definition: ScenarioDefinition {
                    breaker: "Aegis".to_owned(),
                    layout: "Corridor".to_owned(),
                    input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
                    max_frames: 1000,
                    invariants: vec![],
                    expected_violations: None,
                    debug_setup: None,
                    invariant_params: InvariantParams { max_bolt_count },
                    allow_early_end: true,
                    stress: None,
                },
            })
            .add_systems(FixedUpdate, check_bolt_count_reasonable);
        app
    }

    #[test]
    fn bolt_count_reasonable_fires_when_count_exceeds_max() {
        let mut app = bolt_count_test_app(8);

        for _ in 0..9 {
            app.world_mut().spawn(ScenarioTagBolt);
        }

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::BoltCountReasonable);
    }

    #[test]
    fn bolt_count_reasonable_does_not_fire_at_max() {
        let mut app = bolt_count_test_app(8);

        for _ in 0..8 {
            app.world_mut().spawn(ScenarioTagBolt);
        }

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn bolt_count_reasonable_uses_scenario_params() {
        let mut app = bolt_count_test_app(12);

        for _ in 0..10 {
            app.world_mut().spawn(ScenarioTagBolt);
        }

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "10 bolts should be OK with max_bolt_count=12"
        );
    }
}
