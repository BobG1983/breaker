//! System to handle cell damage when hit by the bolt.

use bevy::prelude::*;

use crate::{
    cells::{components::Cell, messages::CellDestroyed, queries::DamageVisualQuery},
    chips::components::DamageBoost,
    physics::messages::BoltHitCell,
    shared::BASE_BOLT_DAMAGE,
};

/// Handles cell damage in response to [`BoltHitCell`] messages.
///
/// Decrements cell health, updates visual feedback via material color,
/// and despawns cells that reach zero HP. Sends [`CellDestroyed`] on destruction.
///
/// Guards against the same cell appearing in multiple messages in one frame
/// (e.g., two bolts hitting the same cell simultaneously): only the first hit
/// that destroys the cell is processed; subsequent messages for an already-despawned
/// cell are skipped to prevent duplicate [`CellDestroyed`] messages.
pub(crate) fn handle_cell_hit(
    mut reader: MessageReader<BoltHitCell>,
    mut cell_query: Query<DamageVisualQuery, With<Cell>>,
    mut commands: Commands,
    mut destroyed_writer: MessageWriter<CellDestroyed>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut despawned: Local<Vec<Entity>>,
    bolt_query: Query<&DamageBoost>,
) {
    // Local<Vec> reuses its heap allocation across frames — zero allocs after warmup.
    // Bounded by MAX_BOUNCES hits per frame.
    despawned.clear();
    for hit in reader.read() {
        if despawned.contains(&hit.cell) {
            continue;
        }
        let Ok((mut health, material_handle, visuals, is_required)) = cell_query.get_mut(hit.cell)
        else {
            continue;
        };

        let boost = bolt_query.get(hit.bolt).map_or(0.0_f32, |b| b.0);
        let destroyed = health.take_damage(BASE_BOLT_DAMAGE * (1.0 + boost));

        if destroyed {
            commands.entity(hit.cell).despawn();
            destroyed_writer.write(CellDestroyed {
                entity: hit.cell,
                was_required_to_clear: is_required,
            });
            despawned.push(hit.cell);
        } else {
            // Visual feedback — dim HDR intensity based on remaining health
            let frac = health.fraction();
            let intensity = frac * visuals.hdr_base;
            if let Some(material) = materials.get_mut(material_handle.id()) {
                material.color = Color::srgb(
                    intensity,
                    visuals.green_min * frac,
                    visuals.blue_range.mul_add(1.0 - frac, visuals.blue_base),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::components::*;

    #[derive(Resource)]
    struct TestMessage(Option<BoltHitCell>);

    #[derive(Resource, Default)]
    struct TestMessages(Vec<BoltHitCell>);

    #[derive(Resource, Default)]
    struct CapturedDestroyed(Vec<CellDestroyed>);

    fn enqueue_from_resource(msg_res: Res<TestMessage>, mut writer: MessageWriter<BoltHitCell>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn enqueue_all(msg_res: Res<TestMessages>, mut writer: MessageWriter<BoltHitCell>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

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
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .add_message::<BoltHitCell>()
            .add_message::<CellDestroyed>()
            .add_systems(FixedUpdate, handle_cell_hit);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn default_damage_visuals() -> CellDamageVisuals {
        CellDamageVisuals {
            hdr_base: 4.0,
            green_min: 0.2,
            blue_range: 0.4,
            blue_base: 0.2,
        }
    }

    fn spawn_cell(app: &mut App, hp: f32) -> Entity {
        let material = app
            .world_mut()
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::from_color(Color::srgb(4.0, 0.2, 0.5)));
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Rectangle::new(1.0, 1.0));
        app.world_mut()
            .spawn((
                Cell,
                CellHealth::new(hp),
                default_damage_visuals(),
                RequiredToClear,
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id()
    }

    fn spawn_optional_cell(app: &mut App, hp: f32, required: bool) -> Entity {
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
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        if required {
            entity.insert(RequiredToClear);
        }
        entity.id()
    }

    #[test]
    fn standard_cell_destroyed_on_hit() {
        let mut app = test_app();
        let cell = spawn_cell(&mut app, 10.0);

        app.insert_resource(TestMessage(Some(BoltHitCell {
            cell,
            bolt: Entity::PLACEHOLDER,
        })));

        app.add_systems(FixedUpdate, enqueue_from_resource.before(handle_cell_hit));
        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_err(),
            "standard cell should be despawned"
        );
    }

    #[test]
    fn tough_cell_survives_one_hit() {
        let mut app = test_app();
        let cell = spawn_cell(&mut app, 30.0);

        app.insert_resource(TestMessage(Some(BoltHitCell {
            cell,
            bolt: Entity::PLACEHOLDER,
        })));

        app.add_systems(FixedUpdate, enqueue_from_resource.before(handle_cell_hit));
        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_ok(),
            "tough cell should survive one hit"
        );
        let health = app.world().get::<CellHealth>(cell).unwrap();
        assert!(
            (health.current - 20.0).abs() < f32::EPSILON,
            "30.0-HP cell after 10 damage should have 20.0 HP, got {}",
            health.current
        );
    }

    #[test]
    fn destroyed_message_includes_required_to_clear() {
        let mut app = test_app();
        let cell = spawn_optional_cell(&mut app, 10.0, true);

        app.init_resource::<CapturedDestroyed>();
        app.insert_resource(TestMessage(Some(BoltHitCell {
            cell,
            bolt: Entity::PLACEHOLDER,
        })));
        app.add_systems(
            FixedUpdate,
            (
                enqueue_from_resource.before(handle_cell_hit),
                capture_destroyed.after(handle_cell_hit),
            ),
        );
        tick(&mut app);

        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "exactly one CellDestroyed should be sent"
        );
        assert!(
            captured.0[0].was_required_to_clear,
            "RequiredToClear cell should set was_required_to_clear = true"
        );
    }

    #[test]
    fn destroyed_message_false_for_non_required_cell() {
        let mut app = test_app();
        let cell = spawn_optional_cell(&mut app, 10.0, false);

        app.init_resource::<CapturedDestroyed>();
        app.insert_resource(TestMessage(Some(BoltHitCell {
            cell,
            bolt: Entity::PLACEHOLDER,
        })));
        app.add_systems(
            FixedUpdate,
            (
                enqueue_from_resource.before(handle_cell_hit),
                capture_destroyed.after(handle_cell_hit),
            ),
        );
        tick(&mut app);

        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "exactly one CellDestroyed should be sent"
        );
        assert!(
            !captured.0[0].was_required_to_clear,
            "non-required cell should set was_required_to_clear = false"
        );
    }

    #[test]
    fn double_hit_multi_hp_cell_decrements_twice() {
        let mut app = test_app();
        let cell = spawn_cell(&mut app, 30.0);

        app.init_resource::<TestMessages>();
        app.world_mut().resource_mut::<TestMessages>().0 = vec![
            BoltHitCell {
                cell,
                bolt: Entity::PLACEHOLDER,
            },
            BoltHitCell {
                cell,
                bolt: Entity::PLACEHOLDER,
            },
        ];
        app.add_systems(FixedUpdate, enqueue_all.before(handle_cell_hit));
        tick(&mut app);

        let health = app.world().get::<CellHealth>(cell).unwrap();
        assert!(
            (health.current - 10.0).abs() < f32::EPSILON,
            "two hits on a 30.0-HP cell should leave 10.0 HP, got {}",
            health.current
        );
    }

    #[test]
    fn double_hit_same_cell_only_destroys_once() {
        let mut app = test_app();
        let cell = spawn_optional_cell(&mut app, 10.0, true);

        app.init_resource::<CapturedDestroyed>();
        app.init_resource::<TestMessages>();
        app.world_mut().resource_mut::<TestMessages>().0 = vec![
            BoltHitCell {
                cell,
                bolt: Entity::PLACEHOLDER,
            },
            BoltHitCell {
                cell,
                bolt: Entity::PLACEHOLDER,
            },
        ];
        app.add_systems(
            FixedUpdate,
            (
                enqueue_all.before(handle_cell_hit),
                capture_destroyed.after(handle_cell_hit),
            ),
        );
        tick(&mut app);

        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "two hits on the same 1-HP cell should produce exactly one CellDestroyed"
        );
    }

    // --- DamageBoost tests ---

    fn spawn_bolt_with_boost(app: &mut App, boost: f32) -> Entity {
        app.world_mut().spawn(DamageBoost(boost)).id()
    }

    fn spawn_bolt_no_boost(app: &mut App) -> Entity {
        app.world_mut().spawn_empty().id()
    }

    #[test]
    fn no_damage_boost_deals_base_bolt_damage_10() {
        // Bolt with NO DamageBoost component hits a 10.0-HP cell — cell is destroyed.
        // Verifies fallback path: system reads BASE_BOLT_DAMAGE (10) when no DamageBoost.
        let mut app = test_app();
        let bolt = spawn_bolt_no_boost(&mut app);
        let cell = spawn_cell(&mut app, 10.0);

        app.init_resource::<CapturedDestroyed>();
        app.insert_resource(TestMessage(Some(BoltHitCell { cell, bolt })));
        app.add_systems(
            FixedUpdate,
            (
                enqueue_from_resource.before(handle_cell_hit),
                capture_destroyed.after(handle_cell_hit),
            ),
        );
        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_err(),
            "bolt with no DamageBoost should deal BASE_BOLT_DAMAGE (10) — 10-HP cell must be destroyed"
        );
        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "exactly one CellDestroyed should be sent"
        );
    }

    #[test]
    fn damage_boost_0_5_deals_15_damage_destroys_15hp_cell() {
        // DamageBoost(0.5) → damage = 10 * (1.0 + 0.5) = 15. Destroys a 15.0-HP cell.
        let mut app = test_app();
        let bolt = spawn_bolt_with_boost(&mut app, 0.5);
        let cell = spawn_cell(&mut app, 15.0);

        app.init_resource::<CapturedDestroyed>();
        app.insert_resource(TestMessage(Some(BoltHitCell { cell, bolt })));
        app.add_systems(
            FixedUpdate,
            (
                enqueue_from_resource.before(handle_cell_hit),
                capture_destroyed.after(handle_cell_hit),
            ),
        );
        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_err(),
            "DamageBoost(0.5) should deal 15 damage — 15-HP cell must be destroyed"
        );
        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "exactly one CellDestroyed should be sent"
        );
    }

    #[test]
    fn damage_boost_0_5_does_not_destroy_16hp_cell() {
        // Edge case: DamageBoost(0.5) → 15 damage. A 16.0-HP cell survives with 1.0 HP.
        let mut app = test_app();
        let bolt = spawn_bolt_with_boost(&mut app, 0.5);
        let cell = spawn_cell(&mut app, 16.0);

        app.insert_resource(TestMessage(Some(BoltHitCell { cell, bolt })));
        app.add_systems(FixedUpdate, enqueue_from_resource.before(handle_cell_hit));
        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_ok(),
            "DamageBoost(0.5) deals 15 damage — 16.0-HP cell must survive"
        );
        let health = app.world().get::<CellHealth>(cell).unwrap();
        assert!(
            (health.current - 1.0).abs() < f32::EPSILON,
            "16.0-HP cell with 15 damage should have 1.0 HP remaining, got {}",
            health.current
        );
    }

    #[test]
    fn damage_boost_1_0_deals_20_damage_destroys_20hp_cell() {
        // DamageBoost(1.0) → damage = 10 * (1.0 + 1.0) = 20. Destroys a 20.0-HP cell.
        let mut app = test_app();
        let bolt = spawn_bolt_with_boost(&mut app, 1.0);
        let cell = spawn_cell(&mut app, 20.0);

        app.init_resource::<CapturedDestroyed>();
        app.insert_resource(TestMessage(Some(BoltHitCell { cell, bolt })));
        app.add_systems(
            FixedUpdate,
            (
                enqueue_from_resource.before(handle_cell_hit),
                capture_destroyed.after(handle_cell_hit),
            ),
        );
        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_err(),
            "DamageBoost(1.0) should deal 20 damage — 20-HP cell must be destroyed"
        );
        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "exactly one CellDestroyed should be sent"
        );
    }

    #[test]
    fn damage_boost_applies_per_bolt_independently() {
        // Bolt A with DamageBoost(1.0) hits Cell A (30 HP) → 20 damage → 10 HP remaining.
        // Bolt B with no DamageBoost hits Cell B (30 HP) → 10 damage → 20 HP remaining.
        // Both messages in one frame. Each bolt's boost applies only to its own hit.
        let mut app = test_app();
        let bolt_a = spawn_bolt_with_boost(&mut app, 1.0);
        let bolt_b = spawn_bolt_no_boost(&mut app);
        let cell_a = spawn_cell(&mut app, 30.0);
        let cell_b = spawn_cell(&mut app, 30.0);

        app.init_resource::<TestMessages>();
        app.world_mut().resource_mut::<TestMessages>().0 = vec![
            BoltHitCell {
                cell: cell_a,
                bolt: bolt_a,
            },
            BoltHitCell {
                cell: cell_b,
                bolt: bolt_b,
            },
        ];
        app.add_systems(FixedUpdate, enqueue_all.before(handle_cell_hit));
        tick(&mut app);

        let health_a = app.world().get::<CellHealth>(cell_a).unwrap();
        assert!(
            (health_a.current - 10.0).abs() < f32::EPSILON,
            "cell A hit by bolt with DamageBoost(1.0): 30.0 - 20 = 10.0 HP, got {}",
            health_a.current
        );

        let health_b = app.world().get::<CellHealth>(cell_b).unwrap();
        assert!(
            (health_b.current - 20.0).abs() < f32::EPSILON,
            "cell B hit by bolt with no DamageBoost: 30.0 - 10 = 20.0 HP, got {}",
            health_b.current
        );
    }

    #[test]
    fn damage_boost_double_hit_dedup_still_works() {
        // Bolt with DamageBoost(0.5) → 15 damage. Cell with 15 HP.
        // Two BoltHitCell messages for same cell, same bolt, sent in two separate ticks.
        //
        // After tick 1 (with correct system): cell destroyed by 15 damage → gone.
        // After tick 1 (with current system): cell takes only 10 damage → 5 HP remaining → alive.
        //
        // The assertion on tick-1 entity state distinguishes the two behaviors.
        // Additionally, exactly 1 CellDestroyed must be sent across both ticks.
        let mut app = test_app();
        let bolt = spawn_bolt_with_boost(&mut app, 0.5);
        let cell = spawn_optional_cell(&mut app, 15.0, true);

        app.init_resource::<CapturedDestroyed>();
        // Tick 1: send first BoltHitCell
        app.insert_resource(TestMessage(Some(BoltHitCell { cell, bolt })));
        app.add_systems(
            FixedUpdate,
            (
                enqueue_from_resource.before(handle_cell_hit),
                capture_destroyed.after(handle_cell_hit),
            ),
        );
        tick(&mut app);

        // With correct system (15 damage): cell is destroyed after tick 1.
        // With current system (10 damage): cell has 5 HP and still exists — assertion FAILS here.
        assert!(
            app.world().get_entity(cell).is_err(),
            "DamageBoost(0.5) should deal 15 damage — 15-HP cell must be destroyed after first hit"
        );

        // Tick 2: send second BoltHitCell for the same (now gone) cell.
        // Entity is already despawned — query returns Err → message silently skipped.
        app.world_mut().resource_mut::<TestMessage>().0 = Some(BoltHitCell { cell, bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "second hit on an already-destroyed cell must produce no additional CellDestroyed"
        );
    }
}
