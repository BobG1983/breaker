use bevy::prelude::*;
use breaker::bolt::components::{Bolt, PrimaryBolt};

use crate::{invariants::*, types::InvariantKind};

/// Checks that exactly one [`PrimaryBolt`] entity exists whenever any [`Bolt`]
/// entity is present.
///
/// Two violation conditions are distinguished:
/// - Zero `PrimaryBolt` entities while at least one `Bolt` exists.
/// - More than one `PrimaryBolt` entity (regardless of `Bolt` count).
///
/// When no `Bolt` entities exist (e.g. mid-node-transition between bolt loss
/// and respawn), the checker skips without producing a violation.
///
/// Gated on [`ScenarioStats::entered_playing`]: when [`ScenarioStats`] is
/// present and `entered_playing` is `false`, the checker early-returns without
/// producing violations.
///
/// Increments [`ScenarioStats::invariant_checks`] when the check runs.
pub fn check_exactly_one_primary_bolt(
    bolts: Query<(), With<Bolt>>,
    primary_bolts: Query<(), With<PrimaryBolt>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }

    if let Some(ref stats) = stats
        && !stats.entered_playing
    {
        return;
    }

    let bolt_count = bolts.iter().count();
    let primary_count = primary_bolts.iter().count();

    if primary_count > 1 {
        log.0.push(ViolationEntry {
            frame:     frame.0,
            invariant: InvariantKind::ExactlyOnePrimaryBolt,
            entity:    None,
            message:   format!(
                "ExactlyOnePrimaryBolt FAIL frame={} count={primary_count} (expected 1)",
                frame.0,
            ),
        });
    } else if primary_count == 0 && bolt_count > 0 {
        log.0.push(ViolationEntry {
            frame:     frame.0,
            invariant: InvariantKind::ExactlyOnePrimaryBolt,
            entity:    None,
            message:   format!(
                "ExactlyOnePrimaryBolt FAIL frame={} no PrimaryBolt when {bolt_count} bolt(s) exist",
                frame.0,
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use breaker::bolt::components::ExtraBolt;

    use super::*;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .insert_resource(ScenarioStats {
                entered_playing: true,
                ..Default::default()
            })
            .add_systems(FixedUpdate, check_exactly_one_primary_bolt);
        app
    }

    // -- Behavior 1: No Bolt entities — no violation --

    #[test]
    fn no_bolts_no_violation() {
        let mut app = test_app();
        // No entities spawned at all
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when no Bolt entities exist, got {}",
            log.0.len()
        );
    }

    // -- Behavior 2: Exactly one PrimaryBolt with bolts — no violation --

    #[test]
    fn exactly_one_primary_bolt_no_violation() {
        let mut app = test_app();
        app.world_mut().spawn((Bolt, PrimaryBolt));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when exactly 1 PrimaryBolt exists, got {}",
            log.0.len()
        );
    }

    #[test]
    fn one_primary_one_extra_no_violation() {
        let mut app = test_app();
        app.world_mut().spawn((Bolt, PrimaryBolt));
        app.world_mut().spawn((Bolt, ExtraBolt));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations with 1 primary + 1 extra bolt, got {}",
            log.0.len()
        );
    }

    // -- Behavior 3: Zero PrimaryBolt with bolts present — violation fires --

    #[test]
    fn zero_primary_with_bolt_fires_violation() {
        let mut app = test_app();
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 15;
        app.world_mut().spawn((Bolt, ExtraBolt)); // Bolt exists, no PrimaryBolt
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 violation for 0 PrimaryBolt with bolts, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::ExactlyOnePrimaryBolt);
        assert_eq!(log.0[0].frame, 15);
        assert!(
            log.0[0].entity.is_none(),
            "entity should be None for count-based checker"
        );
        assert!(
            log.0[0].message.contains("no PrimaryBolt"),
            "violation message should describe missing primary, got: {}",
            log.0[0].message
        );
    }

    // -- Behavior 4: Two PrimaryBolts — violation fires --

    #[test]
    fn two_primary_bolts_fires_violation() {
        let mut app = test_app();
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 30;
        app.world_mut().spawn((Bolt, PrimaryBolt));
        app.world_mut().spawn((Bolt, PrimaryBolt));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 violation for 2 PrimaryBolts, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::ExactlyOnePrimaryBolt);
        assert_eq!(log.0[0].frame, 30);
        assert!(
            log.0[0].message.contains("count=2"),
            "violation message should contain 'count=2', got: {}",
            log.0[0].message
        );
    }

    // -- Behavior 5: Three PrimaryBolts — violation fires --

    #[test]
    fn three_primary_bolts_fires_violation() {
        let mut app = test_app();
        for _ in 0..3 {
            app.world_mut().spawn((Bolt, PrimaryBolt));
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 violation for 3 PrimaryBolts, got {}",
            log.0.len()
        );
        assert!(
            log.0[0].message.contains("count=3"),
            "violation message should contain 'count=3', got: {}",
            log.0[0].message
        );
    }

    // -- Behavior 6: Does not fire when entered_playing is false --

    #[test]
    fn does_not_fire_when_entered_playing_false() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame(5))
            .insert_resource(ScenarioStats {
                entered_playing: false,
                ..Default::default()
            })
            .add_systems(FixedUpdate, check_exactly_one_primary_bolt);
        // Two PrimaryBolts — would be a violation if entered_playing were true
        app.world_mut().spawn((Bolt, PrimaryBolt));
        app.world_mut().spawn((Bolt, PrimaryBolt));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when entered_playing is false, got {}",
            log.0.len()
        );
    }

    // -- Behavior 7: Increments invariant_checks --

    #[test]
    fn increments_invariant_checks() {
        let mut app = test_app();
        app.world_mut().spawn((Bolt, PrimaryBolt));
        tick(&mut app);
        let stats = app.world().resource::<ScenarioStats>();
        assert_eq!(
            stats.invariant_checks, 1,
            "invariant_checks should be 1 after one checker invocation, got {}",
            stats.invariant_checks
        );
    }
}
