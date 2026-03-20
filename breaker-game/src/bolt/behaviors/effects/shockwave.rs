//! Shockwave effect handler — area damage around the bolt's position.
//!
//! Observes [`OverclockEffectFired`], pattern-matches on
//! [`TriggerChain::Shockwave`], and damages all non-locked cells within range
//! using flat `BASE_BOLT_DAMAGE` (no `DamageBoost`).

use bevy::prelude::*;

use crate::{
    bolt::behaviors::events::OverclockEffectFired,
    cells::{
        components::{Cell, CellHealth, Locked, RequiredToClear},
        messages::CellDestroyed,
    },
    chips::definition::TriggerChain,
    shared::BASE_BOLT_DAMAGE,
};

/// Cell data needed by the shockwave effect handler.
type ShockwaveCellQuery = (
    Entity,
    &'static Transform,
    &'static mut CellHealth,
    Has<RequiredToClear>,
    Has<Locked>,
);

/// Observer: handles shockwave area damage when an overclock effect fires.
///
/// Self-selects via pattern matching on [`TriggerChain::Shockwave`] — ignores
/// all other effect variants. Damages all non-locked cells within `range` of
/// the bolt's position using flat [`BASE_BOLT_DAMAGE`].
pub(crate) fn handle_shockwave(
    trigger: On<OverclockEffectFired>,
    bolt_query: Query<&Transform>,
    mut cell_query: Query<ShockwaveCellQuery, With<Cell>>,
    mut commands: Commands,
    mut destroyed_writer: MessageWriter<CellDestroyed>,
) {
    let TriggerChain::Shockwave { range } = &trigger.event().effect else {
        return;
    };
    let Ok(bolt_tf) = bolt_query.get(trigger.event().bolt) else {
        return;
    };
    let center = bolt_tf.translation.truncate();

    for (cell_entity, cell_tf, mut health, is_required, is_locked) in &mut cell_query {
        if is_locked {
            continue;
        }
        let dist = (cell_tf.translation.truncate() - center).length();
        if dist <= *range {
            let destroyed = health.take_damage(BASE_BOLT_DAMAGE);
            if destroyed {
                commands.entity(cell_entity).despawn();
                destroyed_writer.write(CellDestroyed {
                    was_required_to_clear: is_required,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cells::{
            components::{Cell, CellHealth, Locked, RequiredToClear},
            messages::CellDestroyed,
        },
        chips::definition::TriggerChain,
    };

    // --- Test infrastructure ---

    #[derive(Resource, Default)]
    struct CapturedDestroyed(Vec<CellDestroyed>);

    fn capture_destroyed(
        mut reader: MessageReader<CellDestroyed>,
        mut captured: ResMut<CapturedDestroyed>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellDestroyed>()
            .init_resource::<CapturedDestroyed>()
            .add_systems(FixedUpdate, capture_destroyed)
            .add_observer(handle_shockwave);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_bolt(app: &mut App, x: f32, y: f32) -> Entity {
        app.world_mut().spawn(Transform::from_xyz(x, y, 0.0)).id()
    }

    fn spawn_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        app.world_mut()
            .spawn((Cell, CellHealth::new(hp), Transform::from_xyz(x, y, 0.0)))
            .id()
    }

    fn spawn_required_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                CellHealth::new(hp),
                RequiredToClear,
                Transform::from_xyz(x, y, 0.0),
            ))
            .id()
    }

    fn spawn_locked_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                CellHealth::new(hp),
                Locked,
                Transform::from_xyz(x, y, 0.0),
            ))
            .id()
    }

    fn trigger_shockwave(app: &mut App, bolt: Entity, range: f32) {
        app.world_mut().commands().trigger(OverclockEffectFired {
            effect: TriggerChain::Shockwave { range },
            bolt,
        });
        // Flush commands so the observer fires synchronously, writing
        // CellDestroyed messages before the next tick's FixedUpdate runs
        // capture_destroyed via MessageReader.
        app.world_mut().flush();
        tick(app);
    }

    // --- Tests ---

    #[test]
    fn shockwave_damages_cells_within_range() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell_a = spawn_cell(&mut app, 30.0, 0.0, 20.0);
        let cell_b = spawn_cell(&mut app, 50.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let health_a = app.world().get::<CellHealth>(cell_a).unwrap();
        assert!(
            (health_a.current - 10.0).abs() < f32::EPSILON,
            "Cell A at (30,0) with 20 HP should take {} damage and have 10 HP, got {}",
            BASE_BOLT_DAMAGE,
            health_a.current
        );

        let health_b = app.world().get::<CellHealth>(cell_b).unwrap();
        assert!(
            (health_b.current - 10.0).abs() < f32::EPSILON,
            "Cell B at (50,0) with 20 HP should take {} damage and have 10 HP, got {}",
            BASE_BOLT_DAMAGE,
            health_b.current
        );
    }

    #[test]
    fn shockwave_destroys_low_hp_cell_sends_cell_destroyed() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell = spawn_required_cell(&mut app, 10.0, 0.0, 10.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        assert!(
            app.world().get_entity(cell).is_err(),
            "Cell with 10 HP taking 10 damage should be despawned"
        );

        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "exactly one CellDestroyed message should be sent"
        );
        assert!(
            captured.0[0].was_required_to_clear,
            "RequiredToClear cell should set was_required_to_clear = true"
        );
    }

    #[test]
    fn shockwave_ignores_locked_cells() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let locked_cell = spawn_locked_cell(&mut app, 10.0, 0.0, 10.0);
        // Also spawn a non-locked cell to verify shockwave IS active (prevents false pass with stub)
        let unlocked_cell = spawn_cell(&mut app, 20.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        // Locked cell should be immune
        assert!(
            app.world().get_entity(locked_cell).is_ok(),
            "Locked cell should NOT be despawned"
        );
        let health = app.world().get::<CellHealth>(locked_cell).unwrap();
        assert!(
            (health.current - 10.0).abs() < f32::EPSILON,
            "Locked cell should still have 10 HP, got {}",
            health.current
        );

        // Non-locked cell should take damage (ensures shockwave actually ran)
        let unlocked_health = app.world().get::<CellHealth>(unlocked_cell).unwrap();
        assert!(
            (unlocked_health.current - 10.0).abs() < f32::EPSILON,
            "Unlocked cell at (20,0) should take 10 damage: 20 - 10 = 10 HP, got {}",
            unlocked_health.current
        );
    }

    #[test]
    fn shockwave_ignores_cells_outside_range() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let far_cell = spawn_cell(&mut app, 100.0, 0.0, 10.0);
        // Also spawn an in-range cell to verify shockwave IS active (prevents false pass with stub)
        let near_cell = spawn_cell(&mut app, 30.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        // Far cell should be untouched
        assert!(
            app.world().get_entity(far_cell).is_ok(),
            "Cell at distance 100 should NOT be affected by range-64 shockwave"
        );
        let far_health = app.world().get::<CellHealth>(far_cell).unwrap();
        assert!(
            (far_health.current - 10.0).abs() < f32::EPSILON,
            "Cell outside range should still have 10 HP, got {}",
            far_health.current
        );

        // Near cell should take damage (ensures shockwave actually ran)
        let near_health = app.world().get::<CellHealth>(near_cell).unwrap();
        assert!(
            (near_health.current - 10.0).abs() < f32::EPSILON,
            "Cell at (30,0) in range should take 10 damage: 20 - 10 = 10 HP, got {}",
            near_health.current
        );
    }

    #[test]
    fn shockwave_no_op_when_bolt_despawned() {
        // First verify shockwave DOES damage when bolt exists (proves system is wired up)
        let mut app = test_app();
        let proof_bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let proof_cell = spawn_cell(&mut app, 10.0, 0.0, 20.0);

        trigger_shockwave(&mut app, proof_bolt, 64.0);

        let proof_health = app.world().get::<CellHealth>(proof_cell).unwrap();
        assert!(
            (proof_health.current - 10.0).abs() < f32::EPSILON,
            "Proof: shockwave with live bolt should deal 10 damage (20 - 10 = 10 HP), got {}",
            proof_health.current
        );

        // Now test the actual behavior: despawned bolt -> no damage
        let mut app2 = test_app();
        let bolt = spawn_bolt(&mut app2, 0.0, 0.0);
        let cell = spawn_cell(&mut app2, 10.0, 0.0, 10.0);

        app2.world_mut().despawn(bolt);

        trigger_shockwave(&mut app2, bolt, 64.0);

        let health = app2.world().get::<CellHealth>(cell).unwrap();
        assert!(
            (health.current - 10.0).abs() < f32::EPSILON,
            "No cells should be damaged when bolt entity is gone, but cell has {} HP",
            health.current
        );
    }

    #[test]
    fn shockwave_zero_range_damages_cell_at_same_position() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell_at_origin = spawn_cell(&mut app, 0.0, 0.0, 10.0);
        let cell_nearby = spawn_cell(&mut app, 1.0, 0.0, 10.0);

        trigger_shockwave(&mut app, bolt, 0.0);

        // Cell at exactly the same position (distance 0.0 <= range 0.0) should be damaged
        assert!(
            app.world().get_entity(cell_at_origin).is_err(),
            "Cell at distance 0.0 should be damaged and destroyed by zero-range shockwave"
        );

        // Cell at distance 1.0 should NOT be damaged (1.0 > 0.0)
        assert!(
            app.world().get_entity(cell_nearby).is_ok(),
            "Cell at distance 1.0 should NOT be affected by zero-range shockwave"
        );
        let health_nearby = app.world().get::<CellHealth>(cell_nearby).unwrap();
        assert!(
            (health_nearby.current - 10.0).abs() < f32::EPSILON,
            "Cell outside zero range should still have 10 HP, got {}",
            health_nearby.current
        );
    }

    #[test]
    fn shockwave_ignores_non_shockwave_effects() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell = spawn_cell(&mut app, 10.0, 0.0, 20.0);

        // First, prove shockwave works for Shockwave variant
        trigger_shockwave(&mut app, bolt, 64.0);

        let health_after_shockwave = app.world().get::<CellHealth>(cell).unwrap();
        assert!(
            (health_after_shockwave.current - 10.0).abs() < f32::EPSILON,
            "Proof: Shockwave variant should deal 10 damage (20 - 10 = 10 HP), got {}",
            health_after_shockwave.current
        );

        // Now fire a non-shockwave effect -- cell should NOT take further damage
        app.world_mut().commands().trigger(OverclockEffectFired {
            effect: TriggerChain::MultiBolt { count: 3 },
            bolt,
        });
        tick(&mut app);

        let health_after_multi = app.world().get::<CellHealth>(cell).unwrap();
        assert!(
            (health_after_multi.current - 10.0).abs() < f32::EPSILON,
            "MultiBolt effect should not damage cells, expected 10 HP but got {}",
            health_after_multi.current
        );
    }

    #[test]
    fn shockwave_cell_destroyed_false_for_optional_cell() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        // Cell WITHOUT RequiredToClear
        let cell = spawn_cell(&mut app, 10.0, 0.0, 10.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        assert!(
            app.world().get_entity(cell).is_err(),
            "10 HP cell taking 10 damage should be despawned"
        );

        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "exactly one CellDestroyed message should be sent"
        );
        assert!(
            !captured.0[0].was_required_to_clear,
            "Cell without RequiredToClear should set was_required_to_clear = false"
        );
    }

    #[test]
    fn shockwave_multiple_cells_destroyed_sends_multiple_messages() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell_a = spawn_required_cell(&mut app, 10.0, 0.0, 10.0);
        // Cell B has no RequiredToClear
        let cell_b = spawn_cell(&mut app, 20.0, 0.0, 10.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        assert!(
            app.world().get_entity(cell_a).is_err(),
            "Cell A (required, 10 HP) should be despawned"
        );
        assert!(
            app.world().get_entity(cell_b).is_err(),
            "Cell B (optional, 10 HP) should be despawned"
        );

        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            2,
            "two CellDestroyed messages should be sent for two destroyed cells"
        );

        let required_count = captured
            .0
            .iter()
            .filter(|m| m.was_required_to_clear)
            .count();
        let optional_count = captured
            .0
            .iter()
            .filter(|m| !m.was_required_to_clear)
            .count();
        assert_eq!(
            required_count, 1,
            "exactly one CellDestroyed with was_required_to_clear = true"
        );
        assert_eq!(
            optional_count, 1,
            "exactly one CellDestroyed with was_required_to_clear = false"
        );
    }

    #[test]
    fn shockwave_high_hp_cell_survives_no_destroyed_message() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell = spawn_cell(&mut app, 10.0, 0.0, 30.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        assert!(
            app.world().get_entity(cell).is_ok(),
            "30 HP cell taking 10 damage should survive"
        );
        let health = app.world().get::<CellHealth>(cell).unwrap();
        assert!(
            (health.current - 20.0).abs() < f32::EPSILON,
            "30 HP cell taking 10 damage should have 20 HP remaining, got {}",
            health.current
        );

        let captured = app.world().resource::<CapturedDestroyed>();
        assert!(
            captured.0.is_empty(),
            "no CellDestroyed should be sent when cell survives, got {} messages",
            captured.0.len()
        );
    }

    // --- E2E integration test: full Surge overclock chain pipeline ---

    mod e2e {
        use super::*;
        use crate::{
            bolt::behaviors::{
                ActiveOverclocks,
                armed::ArmedTriggers,
                bridges::{bridge_overclock_bump, bridge_overclock_impact},
            },
            breaker::messages::{BumpGrade, BumpPerformed},
            physics::messages::BoltHitCell,
        };

        // -- E2E test infrastructure --

        #[derive(Resource)]
        struct SendBump(Option<BumpPerformed>);

        fn send_bump(msg: Res<SendBump>, mut writer: MessageWriter<BumpPerformed>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        #[derive(Resource)]
        struct SendBoltHitCell(Option<BoltHitCell>);

        fn send_bolt_hit_cell(msg: Res<SendBoltHitCell>, mut writer: MessageWriter<BoltHitCell>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        fn e2e_test_app(active_chains: Vec<TriggerChain>) -> App {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .add_message::<BumpPerformed>()
                .add_message::<BoltHitCell>()
                .add_message::<CellDestroyed>()
                .insert_resource(ActiveOverclocks(active_chains))
                .insert_resource(SendBump(None))
                .insert_resource(SendBoltHitCell(None))
                .init_resource::<CapturedDestroyed>()
                .add_observer(handle_shockwave)
                .add_systems(
                    FixedUpdate,
                    (
                        send_bump,
                        bridge_overclock_bump,
                        send_bolt_hit_cell,
                        bridge_overclock_impact,
                        capture_destroyed,
                    )
                        .chain(),
                );
            app
        }

        /// Exercises the full Surge overclock pipeline end-to-end:
        /// `BumpPerformed(Perfect)` arms bolt, `BoltHitCell` fires shockwave,
        /// shockwave damages nearby cells and despawns those at zero HP.
        #[test]
        fn surge_e2e_perfect_bump_then_impact_fires_shockwave() {
            // Full Surge chain: OnPerfectBump(OnImpact(Shockwave{range: 64.0}))
            let surge_chain = TriggerChain::OnPerfectBump(Box::new(TriggerChain::OnImpact(
                Box::new(TriggerChain::Shockwave { range: 64.0 }),
            )));
            let mut app = e2e_test_app(vec![surge_chain]);

            // Bolt at (0, 50) — shockwave radiates from here
            let bolt = app
                .world_mut()
                .spawn(Transform::from_xyz(0.0, 50.0, 0.0))
                .id();

            // Cell A at (10, 50): distance 10 from bolt, within range 64 — 10 HP (will die)
            let cell_a = app
                .world_mut()
                .spawn((
                    Cell,
                    CellHealth::new(10.0),
                    RequiredToClear,
                    Transform::from_xyz(10.0, 50.0, 0.0),
                ))
                .id();

            // Cell B at (200, 50): distance 200 from bolt, outside range 64 — 10 HP (safe)
            let cell_b = app
                .world_mut()
                .spawn((
                    Cell,
                    CellHealth::new(10.0),
                    Transform::from_xyz(200.0, 50.0, 0.0),
                ))
                .id();

            // --- Step 1: Perfect bump arms the bolt ---
            app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
                grade: BumpGrade::Perfect,
                multiplier: 1.5,
                bolt,
            });
            tick(&mut app);

            // Bolt should now have ArmedTriggers with [OnImpact(Shockwave{64})]
            let armed = app
                .world()
                .get::<ArmedTriggers>(bolt)
                .expect("bolt should have ArmedTriggers after perfect bump");
            assert_eq!(
                armed.0.len(),
                1,
                "bolt should have exactly 1 armed trigger chain"
            );
            assert_eq!(
                armed.0[0],
                TriggerChain::OnImpact(Box::new(TriggerChain::Shockwave { range: 64.0 })),
                "armed trigger should be OnImpact(Shockwave {{range: 64.0}})"
            );

            // No cells damaged yet
            let health_a = app.world().get::<CellHealth>(cell_a).unwrap();
            assert!(
                (health_a.current - 10.0).abs() < f32::EPSILON,
                "Cell A should still have 10.0 HP after bump (no damage yet), got {}",
                health_a.current
            );
            let health_b = app.world().get::<CellHealth>(cell_b).unwrap();
            assert!(
                (health_b.current - 10.0).abs() < f32::EPSILON,
                "Cell B should still have 10.0 HP after bump (no damage yet), got {}",
                health_b.current
            );

            // --- Step 2: Impact fires the shockwave ---
            // Clear bump message, set impact message
            app.world_mut().resource_mut::<SendBump>().0 = None;
            app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
                cell: Entity::PLACEHOLDER, // The cell hit is arbitrary for trigger eval
                bolt,
            });
            tick(&mut app);

            // Cell A (within range 64 at distance 10) should be despawned
            // 10 HP - 10 BASE_BOLT_DAMAGE = 0 HP -> destroyed
            assert!(
                app.world().get_entity(cell_a).is_err(),
                "Cell A at distance 10 from bolt should be destroyed by shockwave (10 HP - 10 damage)"
            );

            // Cell B (outside range 64 at distance 200) should still exist at full HP
            assert!(
                app.world().get_entity(cell_b).is_ok(),
                "Cell B at distance 200 should NOT be affected by range-64 shockwave"
            );
            let health_b_after = app.world().get::<CellHealth>(cell_b).unwrap();
            assert!(
                (health_b_after.current - 10.0).abs() < f32::EPSILON,
                "Cell B should still have 10.0 HP after shockwave, got {}",
                health_b_after.current
            );

            // ArmedTriggers on bolt should be empty (trigger consumed)
            let armed_after = app.world().get::<ArmedTriggers>(bolt).unwrap();
            assert!(
                armed_after.0.is_empty(),
                "ArmedTriggers should be empty after shockwave fired, got {} entries",
                armed_after.0.len()
            );

            // CellDestroyed message should have been sent for Cell A
            let captured = app.world().resource::<CapturedDestroyed>();
            assert_eq!(
                captured.0.len(),
                1,
                "exactly one CellDestroyed message expected for destroyed Cell A"
            );
            assert!(
                captured.0[0].was_required_to_clear,
                "Cell A has RequiredToClear — CellDestroyed.was_required_to_clear should be true"
            );
        }
    }
}
