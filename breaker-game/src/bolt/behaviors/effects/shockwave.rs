//! Shockwave effect handler — area damage around the bolt's position.
//!
//! Observes [`OverclockEffectFired`], pattern-matches on
//! [`TriggerChain::Shockwave`], and writes [`DamageCell`] messages for all
//! non-locked cells within range. Damage includes [`DamageBoost`] if present.

use bevy::prelude::*;

use crate::{
    bolt::behaviors::events::OverclockEffectFired,
    cells::{
        components::{Cell, Locked},
        messages::DamageCell,
    },
    chips::{components::DamageBoost, definition::TriggerChain},
    shared::BASE_BOLT_DAMAGE,
};

/// Cell data needed by the shockwave effect handler.
type ShockwaveCellQuery = (Entity, &'static Transform, Has<Locked>);

/// Observer: handles shockwave area damage when an overclock effect fires.
///
/// Self-selects via pattern matching on [`TriggerChain::Shockwave`] — ignores
/// all other effect variants. Writes [`DamageCell`] messages for all non-locked
/// cells within `range` of the bolt's position. Damage is calculated as
/// `BASE_BOLT_DAMAGE * (1.0 + DamageBoost)`.
pub(crate) fn handle_shockwave(
    trigger: On<OverclockEffectFired>,
    bolt_query: Query<(&Transform, Option<&DamageBoost>)>,
    cell_query: Query<ShockwaveCellQuery, With<Cell>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let TriggerChain::Shockwave {
        base_range,
        range_per_level,
        stacks,
    } = &trigger.event().effect
    else {
        return;
    };
    let range = base_range + f32::from((*stacks).saturating_sub(1)) * range_per_level;
    let Ok((bolt_tf, damage_boost)) = bolt_query.get(trigger.event().bolt) else {
        return;
    };
    let boost = damage_boost.map_or(0.0, |b| b.0);
    let damage = BASE_BOLT_DAMAGE * (1.0 + boost);
    let center = bolt_tf.translation.truncate();

    for (cell_entity, cell_tf, is_locked) in &cell_query {
        if is_locked {
            continue;
        }
        let dist = (cell_tf.translation.truncate() - center).length();
        if dist <= range {
            damage_writer.write(DamageCell {
                cell: cell_entity,
                damage,
                source_bolt: trigger.event().bolt,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cells::{
            components::{Cell, CellHealth, Locked, RequiredToClear},
            messages::{CellDestroyed, DamageCell},
        },
        chips::{components::DamageBoost, definition::TriggerChain},
    };

    // --- Test infrastructure ---

    /// Captured `DamageCell` messages written by the shockwave observer.
    #[derive(Resource, Default)]
    struct CapturedDamage(Vec<DamageCell>);

    fn capture_damage(mut reader: MessageReader<DamageCell>, mut captured: ResMut<CapturedDamage>) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    /// Captured `CellDestroyed` messages — used to verify shockwave does NOT write them.
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
            .add_message::<DamageCell>()
            .add_message::<CellDestroyed>()
            .init_resource::<CapturedDamage>()
            .init_resource::<CapturedDestroyed>()
            .add_systems(FixedUpdate, (capture_damage, capture_destroyed))
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

    fn spawn_bolt_with_damage_boost(app: &mut App, x: f32, y: f32, boost: f32) -> Entity {
        app.world_mut()
            .spawn((Transform::from_xyz(x, y, 0.0), DamageBoost(boost)))
            .id()
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
            effect: TriggerChain::Shockwave {
                base_range: range,
                range_per_level: 0.0,
                stacks: 1,
            },
            bolt,
        });
        // Flush commands so the observer fires synchronously, writing
        // DamageCell messages before the next tick's FixedUpdate runs
        // capture_damage via MessageReader.
        app.world_mut().flush();
        tick(app);
    }

    fn trigger_shockwave_stacked(
        app: &mut App,
        bolt: Entity,
        base_range: f32,
        range_per_level: f32,
        stacks: u32,
    ) {
        app.world_mut().commands().trigger(OverclockEffectFired {
            effect: TriggerChain::Shockwave {
                base_range,
                range_per_level,
                stacks,
            },
            bolt,
        });
        app.world_mut().flush();
        tick(app);
    }

    // --- Tests ---

    // Behavior 1: Shockwave writes DamageCell for each in-range non-locked cell
    #[test]
    fn shockwave_writes_damage_cell_for_each_in_range_cell() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell_a = spawn_cell(&mut app, 30.0, 0.0, 20.0);
        let cell_b = spawn_cell(&mut app, 50.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            2,
            "shockwave with two in-range cells should write two DamageCell messages, got {}",
            captured.0.len()
        );

        // Find message for cell A
        let msg_a = captured
            .0
            .iter()
            .find(|m| m.cell == cell_a)
            .expect("should have a DamageCell for cell A at (30, 0)");
        assert!(
            (msg_a.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "DamageCell for cell A should have damage == {}, got {}",
            BASE_BOLT_DAMAGE,
            msg_a.damage
        );
        assert_eq!(
            msg_a.source_bolt, bolt,
            "DamageCell.source_bolt should be the bolt entity"
        );

        // Find message for cell B
        let msg_b = captured
            .0
            .iter()
            .find(|m| m.cell == cell_b)
            .expect("should have a DamageCell for cell B at (50, 0)");
        assert!(
            (msg_b.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "DamageCell for cell B should have damage == {}, got {}",
            BASE_BOLT_DAMAGE,
            msg_b.damage
        );
        assert_eq!(
            msg_b.source_bolt, bolt,
            "DamageCell.source_bolt should be the bolt entity"
        );
    }

    // Behavior 2: Shockwave at exact range boundary includes cell
    #[test]
    fn shockwave_includes_cell_at_exact_range_boundary() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let _cell = spawn_cell(&mut app, 64.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "cell at exactly range 64.0 should be included, got {} messages",
            captured.0.len()
        );
        assert!(
            (captured.0[0].damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "DamageCell.damage should be {}, got {}",
            BASE_BOLT_DAMAGE,
            captured.0[0].damage
        );
    }

    // Behavior 3: Shockwave does NOT directly mutate CellHealth
    #[test]
    fn shockwave_does_not_directly_mutate_cell_health() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell = spawn_cell(&mut app, 10.0, 0.0, 10.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        // CellHealth should be UNCHANGED — shockwave delegates damage to handle_cell_hit
        let health = app.world().get::<CellHealth>(cell).unwrap();
        assert!(
            (health.current - 10.0).abs() < f32::EPSILON,
            "Shockwave should NOT mutate CellHealth directly; expected 10.0, got {}",
            health.current
        );

        // Cell entity should still exist (not despawned by shockwave)
        assert!(
            app.world().get_entity(cell).is_ok(),
            "Shockwave should NOT despawn cells directly — that is handle_cell_hit's job"
        );

        // DamageCell should still be written
        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "one DamageCell message should be written for the in-range cell"
        );
    }

    // Behavior 4: Shockwave does NOT write CellDestroyed messages
    #[test]
    fn shockwave_does_not_write_cell_destroyed() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let _cell = spawn_required_cell(&mut app, 10.0, 0.0, 10.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            0,
            "Shockwave should NOT write CellDestroyed — that is handle_cell_hit's job, got {} messages",
            captured.0.len()
        );
    }

    // Behavior 5: Shockwave applies DamageBoost to damage amount
    #[test]
    fn shockwave_applies_damage_boost() {
        let mut app = test_app();
        let bolt = spawn_bolt_with_damage_boost(&mut app, 0.0, 0.0, 0.5);
        let _cell = spawn_cell(&mut app, 10.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "one DamageCell should be written for in-range cell"
        );
        // damage = BASE_BOLT_DAMAGE * (1.0 + 0.5) = 10.0 * 1.5 = 15.0
        assert!(
            (captured.0[0].damage - 15.0).abs() < f32::EPSILON,
            "DamageCell.damage with DamageBoost(0.5) should be 15.0, got {}",
            captured.0[0].damage
        );
        assert_eq!(
            captured.0[0].source_bolt, bolt,
            "DamageCell.source_bolt should be the bolt entity"
        );
    }

    // Behavior 6: Shockwave with large DamageBoost
    #[test]
    fn shockwave_with_large_damage_boost() {
        let mut app = test_app();
        let bolt = spawn_bolt_with_damage_boost(&mut app, 0.0, 0.0, 1.0);
        let _cell = spawn_cell(&mut app, 10.0, 0.0, 30.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(captured.0.len(), 1, "one DamageCell should be written");
        // damage = BASE_BOLT_DAMAGE * (1.0 + 1.0) = 10.0 * 2.0 = 20.0
        assert!(
            (captured.0[0].damage - 20.0).abs() < f32::EPSILON,
            "DamageCell.damage with DamageBoost(1.0) should be 20.0, got {}",
            captured.0[0].damage
        );
    }

    // Behavior 7: Shockwave without DamageBoost uses base damage
    #[test]
    fn shockwave_without_damage_boost_uses_base_damage() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0); // no DamageBoost component
        let _cell = spawn_cell(&mut app, 10.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(captured.0.len(), 1, "one DamageCell should be written");
        assert!(
            (captured.0[0].damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "DamageCell.damage without DamageBoost should be {}, got {}",
            BASE_BOLT_DAMAGE,
            captured.0[0].damage
        );
    }

    // Behavior 8: Shockwave skips locked cells
    #[test]
    fn shockwave_skips_locked_cells() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let _locked_cell = spawn_locked_cell(&mut app, 10.0, 0.0, 10.0);
        let unlocked_cell = spawn_cell(&mut app, 20.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "only one DamageCell for the unlocked cell, got {}",
            captured.0.len()
        );
        assert_eq!(
            captured.0[0].cell, unlocked_cell,
            "DamageCell should target the unlocked cell, not the locked one"
        );
    }

    // Behavior 9: Shockwave skips cells outside range
    #[test]
    fn shockwave_skips_cells_outside_range() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let _far_cell = spawn_cell(&mut app, 100.0, 0.0, 10.0);
        let near_cell = spawn_cell(&mut app, 30.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "only one DamageCell for the near cell, got {}",
            captured.0.len()
        );
        assert_eq!(
            captured.0[0].cell, near_cell,
            "DamageCell should target the near cell (distance 30), not the far cell (distance 100)"
        );
    }

    // Behavior 10: Shockwave no-op when bolt despawned
    #[test]
    fn shockwave_no_op_when_bolt_despawned() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let _cell = spawn_cell(&mut app, 10.0, 0.0, 10.0);

        app.world_mut().despawn(bolt);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            0,
            "no DamageCell messages when bolt entity is despawned, got {}",
            captured.0.len()
        );
    }

    // Behavior 11: Shockwave no-op with Entity::PLACEHOLDER bolt (global trigger)
    #[test]
    fn shockwave_no_op_with_placeholder_bolt() {
        let mut app = test_app();
        let _cell = spawn_cell(&mut app, 10.0, 0.0, 10.0);

        trigger_shockwave(&mut app, Entity::PLACEHOLDER, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            0,
            "no DamageCell messages when bolt is Entity::PLACEHOLDER, got {}",
            captured.0.len()
        );
    }

    // Behavior 12: Shockwave zero range — co-located cell only
    #[test]
    fn shockwave_zero_range_targets_colocated_cell_only() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let colocated_cell = spawn_cell(&mut app, 0.0, 0.0, 10.0);
        let _nearby_cell = spawn_cell(&mut app, 1.0, 0.0, 10.0);

        trigger_shockwave(&mut app, bolt, 0.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "zero-range shockwave should only target co-located cell (distance 0.0), got {}",
            captured.0.len()
        );
        assert_eq!(
            captured.0[0].cell, colocated_cell,
            "DamageCell should target the co-located cell, not the nearby one"
        );
    }

    // Behavior 13: Shockwave ignores non-Shockwave effects
    #[test]
    fn shockwave_ignores_non_shockwave_effects() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let _cell = spawn_cell(&mut app, 10.0, 0.0, 20.0);

        // Fire a MultiBolt effect — shockwave observer should ignore it
        app.world_mut().commands().trigger(OverclockEffectFired {
            effect: TriggerChain::MultiBolt {
                base_count: 3,
                count_per_level: 0,
                stacks: 1,
            },
            bolt,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            0,
            "MultiBolt effect should not produce any DamageCell messages, got {}",
            captured.0.len()
        );
    }

    // --- Phase D: Stacking integration tests ---

    // Behavior: stacks=2 computes effective range = base + 1*per_level = 64+32 = 96
    #[test]
    fn shockwave_uses_effective_range_from_stacked_variant() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell_30 = spawn_cell(&mut app, 30.0, 0.0, 20.0);
        let cell_50 = spawn_cell(&mut app, 50.0, 0.0, 20.0);
        let cell_70 = spawn_cell(&mut app, 70.0, 0.0, 20.0);
        let _cell_100 = spawn_cell(&mut app, 100.0, 0.0, 20.0);

        // Shockwave { base_range: 64.0, range_per_level: 32.0, stacks: 2 }
        // effective range = 64.0 + (2-1)*32.0 = 96.0
        trigger_shockwave_stacked(&mut app, bolt, 64.0, 32.0, 2);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            3,
            "stacks=2 effective range 96.0: cells at 30, 50, 70 should be hit, not cell at 100; got {} hits",
            captured.0.len()
        );
        let hit_cells: Vec<Entity> = captured.0.iter().map(|m| m.cell).collect();
        assert!(
            hit_cells.contains(&cell_30),
            "cell at 30 should be within effective range 96.0"
        );
        assert!(
            hit_cells.contains(&cell_50),
            "cell at 50 should be within effective range 96.0"
        );
        assert!(
            hit_cells.contains(&cell_70),
            "cell at 70 should be within effective range 96.0"
        );
    }

    // Behavior: stacks=1 uses base_range only (effective = 64.0)
    #[test]
    fn shockwave_stacks_1_uses_base_range_only() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell_30 = spawn_cell(&mut app, 30.0, 0.0, 20.0);
        let cell_50 = spawn_cell(&mut app, 50.0, 0.0, 20.0);
        let _cell_70 = spawn_cell(&mut app, 70.0, 0.0, 20.0);
        let _cell_100 = spawn_cell(&mut app, 100.0, 0.0, 20.0);

        // Shockwave { base_range: 64.0, range_per_level: 32.0, stacks: 1 }
        // effective range = 64.0 + (1-1)*32.0 = 64.0
        trigger_shockwave_stacked(&mut app, bolt, 64.0, 32.0, 1);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            2,
            "stacks=1 effective range 64.0: cells at 30 and 50 should be hit; got {} hits",
            captured.0.len()
        );
        let hit_cells: Vec<Entity> = captured.0.iter().map(|m| m.cell).collect();
        assert!(
            hit_cells.contains(&cell_30),
            "cell at 30 should be within effective range 64.0"
        );
        assert!(
            hit_cells.contains(&cell_50),
            "cell at 50 should be within effective range 64.0"
        );
    }

    // --- E2E integration tests: full Surge pipeline ---

    mod e2e {
        use super::*;
        use crate::{
            bolt::behaviors::{
                ActiveOverclocks,
                armed::ArmedTriggers,
                bridges::{bridge_overclock_bump, bridge_overclock_cell_impact},
            },
            breaker::messages::{BumpGrade, BumpPerformed},
            cells::components::CellDamageVisuals,
            // NOTE: handle_cell_hit needs a pub(crate) re-export from cells/mod.rs.
            // The orchestrator adds: `pub(crate) use systems::handle_cell_hit;`
            cells::handle_cell_hit,
            chips::definition::ImpactTarget,
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

        fn default_damage_visuals() -> CellDamageVisuals {
            CellDamageVisuals {
                hdr_base: 4.0,
                green_min: 0.2,
                blue_range: 0.4,
                blue_base: 0.2,
            }
        }

        fn e2e_test_app(active_chains: Vec<TriggerChain>) -> App {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .add_message::<DamageCell>()
                .add_message::<BumpPerformed>()
                .add_message::<BoltHitCell>()
                .add_message::<CellDestroyed>()
                .init_resource::<Assets<ColorMaterial>>()
                .init_resource::<Assets<Mesh>>()
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
                        bridge_overclock_cell_impact,
                        handle_cell_hit,
                        capture_destroyed,
                    )
                        .chain(),
                );
            app
        }

        fn spawn_e2e_cell(app: &mut App, x: f32, y: f32, hp: f32, required: bool) -> Entity {
            let material = app
                .world_mut()
                .resource_mut::<Assets<ColorMaterial>>()
                .add(ColorMaterial::from_color(Color::srgb(4.0, 0.2, 0.5)));
            let mesh = app
                .world_mut()
                .resource_mut::<Assets<Mesh>>()
                .add(Rectangle::new(1.0, 1.0));
            let mut entity = app.world_mut().spawn((
                Cell,
                CellHealth::new(hp),
                default_damage_visuals(),
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_xyz(x, y, 0.0),
            ));
            if required {
                entity.insert(RequiredToClear);
            }
            entity.id()
        }

        // Behavior 14: E2E full Surge pipeline — DamageCell -> handle_cell_hit -> CellDestroyed
        #[test]
        fn surge_e2e_shockwave_routes_damage_cell_through_handle_cell_hit() {
            // Full Surge chain: OnPerfectBump(OnImpact(Cell, Shockwave{range: 64.0}))
            let surge_chain = TriggerChain::OnPerfectBump(Box::new(TriggerChain::OnImpact(
                ImpactTarget::Cell,
                Box::new(TriggerChain::test_shockwave(64.0)),
            )));
            let mut app = e2e_test_app(vec![surge_chain]);

            // Bolt at (0, 50)
            let bolt = app
                .world_mut()
                .spawn(Transform::from_xyz(0.0, 50.0, 0.0))
                .id();

            // Cell A at (10, 50): distance 10 from bolt, within range 64 — 10 HP (will die)
            let cell_a = spawn_e2e_cell(&mut app, 10.0, 50.0, 10.0, true);

            // Cell B at (200, 50): distance 200, outside range 64 — 10 HP (safe)
            let cell_b = spawn_e2e_cell(&mut app, 200.0, 50.0, 10.0, false);

            // --- Step 1: Perfect bump arms the bolt ---
            app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
                grade: BumpGrade::Perfect,
                multiplier: 1.5,
                bolt,
            });
            tick(&mut app);

            // Bolt should now have ArmedTriggers
            let armed = app
                .world()
                .get::<ArmedTriggers>(bolt)
                .expect("bolt should have ArmedTriggers after perfect bump");
            assert_eq!(
                armed.0.len(),
                1,
                "bolt should have exactly 1 armed trigger chain"
            );

            // No cells damaged yet
            let health_a = app.world().get::<CellHealth>(cell_a).unwrap();
            assert!(
                (health_a.current - 10.0).abs() < f32::EPSILON,
                "Cell A should still have 10.0 HP after bump, got {}",
                health_a.current
            );

            // --- Step 2: Impact fires shockwave -> DamageCell -> handle_cell_hit ---
            app.world_mut().resource_mut::<SendBump>().0 = None;
            app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
                cell: Entity::PLACEHOLDER,
                bolt,
            });
            tick(&mut app);

            // Cell A (distance 10, in range 64, 10 HP - 10 damage = 0) should be despawned
            assert!(
                app.world().get_entity(cell_a).is_err(),
                "Cell A at distance 10 should be destroyed via DamageCell -> handle_cell_hit"
            );

            // Cell B (distance 200, out of range) should still exist
            assert!(
                app.world().get_entity(cell_b).is_ok(),
                "Cell B at distance 200 should NOT be affected by range-64 shockwave"
            );
            let health_b = app.world().get::<CellHealth>(cell_b).unwrap();
            assert!(
                (health_b.current - 10.0).abs() < f32::EPSILON,
                "Cell B should still have 10.0 HP, got {}",
                health_b.current
            );

            // CellDestroyed from handle_cell_hit (NOT from shockwave directly)
            let captured = app.world().resource::<CapturedDestroyed>();
            assert_eq!(
                captured.0.len(),
                1,
                "exactly one CellDestroyed expected from handle_cell_hit for Cell A, got {}",
                captured.0.len()
            );
            assert!(
                captured.0[0].was_required_to_clear,
                "Cell A has RequiredToClear — CellDestroyed.was_required_to_clear should be true"
            );

            // ArmedTriggers on bolt should be empty (trigger consumed)
            let armed_after = app.world().get::<ArmedTriggers>(bolt).unwrap();
            assert!(
                armed_after.0.is_empty(),
                "ArmedTriggers should be empty after shockwave fired, got {} entries",
                armed_after.0.len()
            );
        }

        // Behavior 15: E2E shockwave with DamageBoost through full pipeline
        #[test]
        fn surge_e2e_shockwave_with_damage_boost_partial_damage() {
            let surge_chain = TriggerChain::OnPerfectBump(Box::new(TriggerChain::OnImpact(
                ImpactTarget::Cell,
                Box::new(TriggerChain::test_shockwave(64.0)),
            )));
            let mut app = e2e_test_app(vec![surge_chain]);

            // Bolt at (0, 50) with DamageBoost(0.5) -> damage = 10 * 1.5 = 15
            let bolt = app
                .world_mut()
                .spawn((Transform::from_xyz(0.0, 50.0, 0.0), DamageBoost(0.5)))
                .id();

            // Cell at (10, 50): 20 HP — should survive with 5 HP after 15 damage
            let cell = spawn_e2e_cell(&mut app, 10.0, 50.0, 20.0, true);

            // Step 1: Perfect bump arms the bolt
            app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
                grade: BumpGrade::Perfect,
                multiplier: 1.5,
                bolt,
            });
            tick(&mut app);

            // Step 2: Impact fires shockwave -> DamageCell(15) -> handle_cell_hit
            app.world_mut().resource_mut::<SendBump>().0 = None;
            app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
                cell: Entity::PLACEHOLDER,
                bolt,
            });
            tick(&mut app);

            // Cell should survive with 5.0 HP (20.0 - 15.0)
            assert!(
                app.world().get_entity(cell).is_ok(),
                "20 HP cell hit by 15 damage (DamageBoost 0.5) should survive"
            );
            let health = app.world().get::<CellHealth>(cell).unwrap();
            assert!(
                (health.current - 5.0).abs() < f32::EPSILON,
                "Cell should have 5.0 HP (20.0 - 15.0), got {}",
                health.current
            );

            // No CellDestroyed — cell survived
            let captured = app.world().resource::<CapturedDestroyed>();
            assert_eq!(
                captured.0.len(),
                0,
                "surviving cell should not produce CellDestroyed, got {}",
                captured.0.len()
            );
        }
    }
}
