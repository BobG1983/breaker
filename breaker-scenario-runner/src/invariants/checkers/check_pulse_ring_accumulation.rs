use bevy::prelude::*;
use breaker::effect::effects::pulse::PulseRing;

use crate::{invariants::*, lifecycle::ScenarioConfig, types::InvariantKind};

/// Checks that [`PulseRing`] entities do not accumulate unboundedly.
///
/// Each bolt with a [`PulseEmitter`](breaker::effect::effects::pulse::PulseEmitter)
/// spawns expanding ring entities periodically. Rings self-despawn when they reach
/// their maximum radius. If despawning is broken — missing system, wrong ordering,
/// or rings stuck at radius-zero — they accumulate indefinitely.
///
/// Fires when ring count exceeds `invariant_params.max_pulse_ring_count` (default 20).
/// A clean run with a single emitter should never have more than a handful of rings
/// alive simultaneously. Twenty is a conservative ceiling that catches accumulation
/// while tolerating burst spawning from multiple bolts.
pub fn check_pulse_ring_accumulation(
    rings: Query<Entity, With<PulseRing>>,
    config: Res<ScenarioConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let max = config.definition.invariant_params.max_pulse_ring_count;
    let count = rings.iter().count();
    if count > max {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::PulseRingAccumulation,
            entity: None,
            message: format!(
                "PulseRingAccumulation FAIL frame={} count={count} max={max}",
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

    fn test_app(max_pulse_ring_count: usize) -> App {
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
                        max_pulse_ring_count,
                        ..InvariantParams::default()
                    },
                    ..Default::default()
                },
            })
            .add_systems(FixedUpdate, check_pulse_ring_accumulation);
        app
    }

    #[test]
    fn no_violation_when_no_rings_exist() {
        let mut app = test_app(20);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when no PulseRing entities exist"
        );
    }

    #[test]
    fn no_violation_at_exactly_max_rings() {
        let mut app = test_app(5);
        for _ in 0..5 {
            app.world_mut().spawn(PulseRing);
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when ring count equals max (count=max=5)"
        );
    }

    #[test]
    fn fires_when_ring_count_exceeds_max() {
        let mut app = test_app(5);
        for _ in 0..6 {
            app.world_mut().spawn(PulseRing);
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one PulseRingAccumulation violation, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::PulseRingAccumulation);
    }

    #[test]
    fn violation_message_includes_count_and_max() {
        let mut app = test_app(5);
        for _ in 0..8 {
            app.world_mut().spawn(PulseRing);
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
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
    }

    #[test]
    fn uses_scenario_params_for_max() {
        // max=20 (default) — 15 rings should be OK
        let mut app = test_app(20);
        for _ in 0..15 {
            app.world_mut().spawn(PulseRing);
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "15 rings should be OK with max_pulse_ring_count=20"
        );
    }
}
