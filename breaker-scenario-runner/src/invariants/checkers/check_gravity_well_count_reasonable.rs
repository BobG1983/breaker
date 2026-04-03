use bevy::prelude::*;
use breaker::effect::effects::gravity_well::GravityWell;

use crate::{invariants::*, lifecycle::ScenarioConfig, types::InvariantKind};

/// Checks that [`GravityWell`] entities do not accumulate unboundedly.
///
/// Gravity wells are spawned by the gravity well effect and despawned when
/// reversed. If despawning is broken, wells accumulate indefinitely.
///
/// Fires when gravity well count exceeds
/// `invariant_params.max_gravity_well_count` (default 10).
pub fn check_gravity_well_count_reasonable(
    wells: Query<Entity, With<GravityWell>>,
    config: Res<ScenarioConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let max = config.definition.invariant_params.max_gravity_well_count;
    let count = wells.iter().count();
    if count > max {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::GravityWellCountReasonable,
            entity: None,
            message: format!(
                "GravityWellCountReasonable FAIL frame={} count={count} max={max}",
                frame.0,
            ),
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

    fn test_app(max_gravity_well_count: usize) -> App {
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
                    disallowed_failures: vec![],
                    invariant_params: InvariantParams {
                        max_gravity_well_count,
                        ..InvariantParams::default()
                    },
                    ..Default::default()
                },
            })
            .add_systems(FixedUpdate, check_gravity_well_count_reasonable);
        app
    }

    #[test]
    fn no_violation_when_no_wells_exist() {
        let mut app = test_app(10);
        // Spawn an unrelated entity to verify the query filters correctly.
        app.world_mut().spawn(Transform::default());
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when no GravityWell entities exist"
        );
    }

    #[test]
    fn no_violation_at_exactly_max_wells() {
        let mut app = test_app(5);
        for _ in 0..5 {
            app.world_mut().spawn(GravityWell);
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when well count equals max (count=max=5)"
        );
    }

    #[test]
    fn fires_when_well_count_exceeds_max() {
        let mut app = test_app(5);
        for _ in 0..6 {
            app.world_mut().spawn(GravityWell);
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one GravityWellCountReasonable violation, got {}",
            log.0.len()
        );
        assert_eq!(
            log.0[0].invariant,
            InvariantKind::GravityWellCountReasonable
        );
    }

    #[test]
    fn violation_message_includes_count_and_max() {
        let mut app = test_app(5);
        for _ in 0..8 {
            app.world_mut().spawn(GravityWell);
        }
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 42;
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one violation, got {}",
            log.0.len()
        );
        assert!(
            log.0[0].message.contains("count=8"),
            "violation message should include count=8, got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].message.contains("max=5"),
            "violation message should include max=5, got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].message.contains("frame=42"),
            "violation message should include frame=42, got: {}",
            log.0[0].message
        );
    }

    #[test]
    fn uses_scenario_params_for_max() {
        // max=20 (custom high ceiling) — 15 wells should be OK
        let mut app = test_app(20);
        for _ in 0..15 {
            app.world_mut().spawn(GravityWell);
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "15 wells should be OK with max_gravity_well_count=20"
        );
    }

    #[test]
    fn uses_scenario_params_fires_with_lower_max() {
        // Same entity count (15) but with a lower max (14) — should fire
        let mut app = test_app(14);
        for _ in 0..15 {
            app.world_mut().spawn(GravityWell);
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one violation when 15 wells exceed max of 14, got {}",
            log.0.len()
        );
        assert_eq!(
            log.0[0].invariant,
            InvariantKind::GravityWellCountReasonable
        );
    }
}
