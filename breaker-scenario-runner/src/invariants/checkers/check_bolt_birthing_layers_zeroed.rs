use bevy::prelude::*;
use breaker::shared::birthing::Birthing;
use rantzsoft_physics2d::collision_layers::CollisionLayers;

use crate::{invariants::*, types::InvariantKind};

type BirthingBoltQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static CollisionLayers), (With<ScenarioTagBolt>, With<Birthing>)>;

/// Checks that all bolts with [`Birthing`] have zeroed [`CollisionLayers`].
///
/// During the birthing animation, a bolt's `CollisionLayers` are set to zero
/// (membership=0, mask=0) so it cannot collide. The real layers are stashed
/// inside the `Birthing` component and restored when the animation completes.
///
/// If a birthing bolt has non-zero collision layers, something failed to zero
/// them at spawn — the bolt could collide during its scale-up animation.
pub fn check_bolt_birthing_layers_zeroed(
    bolts: BirthingBoltQuery,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    for (entity, layers) in &bolts {
        if layers.membership != 0 || layers.mask != 0 {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltBirthingLayersZeroed,
                entity: Some(entity),
                message: format!(
                    "BoltBirthingLayersZeroed FAIL frame={} entity={entity:?} \
                     membership={:#06x} mask={:#06x} (expected both 0)",
                    frame.0, layers.membership, layers.mask,
                ),
            });
        }
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

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_bolt_birthing_layers_zeroed);
        app
    }

    fn spawn_birthing_bolt(app: &mut App, membership: u32, mask: u32) -> Entity {
        use rantzsoft_spatial2d::components::Scale2D;

        let layers = CollisionLayers::new(membership, mask);
        let birthing = Birthing::new(
            Scale2D { x: 8.0, y: 8.0 },
            CollisionLayers::new(0x01, 0x0E), // stashed (irrelevant to checker)
        );
        app.world_mut()
            .spawn((ScenarioTagBolt, layers, birthing))
            .id()
    }

    #[test]
    fn no_violation_when_no_birthing_bolts_exist() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn no_violation_when_birthing_bolt_has_zeroed_layers() {
        let mut app = test_app();
        spawn_birthing_bolt(&mut app, 0, 0);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty(), "zeroed layers should not fire violation");
    }

    #[test]
    fn fires_when_birthing_bolt_has_nonzero_membership() {
        let mut app = test_app();
        spawn_birthing_bolt(&mut app, 0x01, 0);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::BoltBirthingLayersZeroed);
    }

    #[test]
    fn fires_when_birthing_bolt_has_nonzero_mask() {
        let mut app = test_app();
        spawn_birthing_bolt(&mut app, 0, 0x0E);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::BoltBirthingLayersZeroed);
    }

    #[test]
    fn fires_when_birthing_bolt_has_both_nonzero() {
        let mut app = test_app();
        spawn_birthing_bolt(&mut app, 0x01, 0x0E);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::BoltBirthingLayersZeroed);
    }

    #[test]
    fn does_not_fire_for_non_birthing_bolt() {
        let mut app = test_app();
        // Bolt with non-zero layers but NO Birthing component — should be fine.
        app.world_mut()
            .spawn((ScenarioTagBolt, CollisionLayers::new(0x01, 0x0E)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "non-birthing bolt with active layers should not fire"
        );
    }

    #[test]
    fn message_includes_layer_values() {
        let mut app = test_app();
        spawn_birthing_bolt(&mut app, 0x01, 0x0E);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0[0].message.contains("membership=0x0001"));
        assert!(log.0[0].message.contains("mask=0x000e"));
    }

    #[test]
    fn increments_invariant_checks() {
        let mut app = test_app();
        app.insert_resource(ScenarioStats::default());
        spawn_birthing_bolt(&mut app, 0, 0);
        tick(&mut app);
        let stats = app.world().resource::<ScenarioStats>();
        assert!(
            stats.invariant_checks >= 1,
            "expected invariant_checks >= 1, got {}",
            stats.invariant_checks
        );
    }
}
