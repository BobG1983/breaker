//! Deactivates/reactivates attraction types based on bolt collision messages.

use bevy::prelude::*;

use crate::{
    bolt::messages::{BoltHitBreaker, BoltHitCell, BoltHitWall},
    effect::definition::AttractionType,
    effect::effects::attraction::ActiveAttractions,
};

/// Reads bolt collision messages and toggles attraction entry active state:
/// - On `BoltHitCell`: deactivate `Cell` entries on that bolt
/// - On `BoltHitWall`: deactivate `Wall` entries on that bolt
/// - On `BoltHitBreaker`: if `Breaker` is NOT in entries, reactivate ALL entries
/// - A hit with an attracted type deactivates that type but does NOT reactivate others
pub(crate) fn manage_attraction_types(
    mut cell_reader: MessageReader<BoltHitCell>,
    mut wall_reader: MessageReader<BoltHitWall>,
    mut breaker_reader: MessageReader<BoltHitBreaker>,
    mut query: Query<&mut ActiveAttractions>,
) {
    for msg in cell_reader.read() {
        if let Ok(mut aa) = query.get_mut(msg.bolt) {
            update_attraction_state(&mut aa, AttractionType::Cell);
        }
    }
    for msg in wall_reader.read() {
        if let Ok(mut aa) = query.get_mut(msg.bolt) {
            update_attraction_state(&mut aa, AttractionType::Wall);
        }
    }
    for msg in breaker_reader.read() {
        if let Ok(mut aa) = query.get_mut(msg.bolt) {
            update_attraction_state(&mut aa, AttractionType::Breaker);
        }
    }
}

/// Toggles attraction state based on which entity type was hit.
///
/// If the hit type is in the entries, deactivate it. Otherwise, reactivate
/// all entries (the bolt bounced off a non-attracted entity type).
fn update_attraction_state(aa: &mut ActiveAttractions, hit_type: AttractionType) {
    let has_type = aa.entries.iter().any(|e| e.attraction_type == hit_type);
    if has_type {
        for entry in &mut aa.entries {
            if entry.attraction_type == hit_type {
                entry.active = false;
            }
        }
    } else {
        for entry in &mut aa.entries {
            entry.active = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::Bolt;
    use crate::effect::effects::attraction::AttractionEntry;

    // -- Helper resources for injecting messages into the test app ----------

    #[derive(Resource, Default)]
    struct PendingCellHit(Option<BoltHitCell>);

    #[derive(Resource, Default)]
    struct PendingWallHit(Option<BoltHitWall>);

    #[derive(Resource, Default)]
    struct PendingBreakerHit(Option<BoltHitBreaker>);

    fn send_cell_hit(
        pending: Res<PendingCellHit>,
        mut writer: MessageWriter<BoltHitCell>,
    ) {
        if let Some(msg) = pending.0.clone() {
            writer.write(msg);
        }
    }

    fn send_wall_hit(
        pending: Res<PendingWallHit>,
        mut writer: MessageWriter<BoltHitWall>,
    ) {
        if let Some(msg) = pending.0.clone() {
            writer.write(msg);
        }
    }

    fn send_breaker_hit(
        pending: Res<PendingBreakerHit>,
        mut writer: MessageWriter<BoltHitBreaker>,
    ) {
        if let Some(msg) = pending.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .add_message::<BoltHitWall>()
            .add_message::<BoltHitBreaker>()
            .init_resource::<PendingCellHit>()
            .init_resource::<PendingWallHit>()
            .init_resource::<PendingBreakerHit>()
            .add_systems(
                FixedUpdate,
                (
                    send_cell_hit.before(manage_attraction_types),
                    send_wall_hit.before(manage_attraction_types),
                    send_breaker_hit.before(manage_attraction_types),
                    manage_attraction_types,
                ),
            );
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn cell_attraction_deactivates_on_bolt_hit_cell() {
        let mut app = test_app();

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                ActiveAttractions {
                    entries: vec![AttractionEntry {
                        attraction_type: AttractionType::Cell,
                        force: 10.0,
                        active: true,
                    }],
                },
            ))
            .id();

        app.insert_resource(PendingCellHit(Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        })));

        tick(&mut app);

        let aa = app
            .world()
            .entity(bolt)
            .get::<ActiveAttractions>()
            .expect("bolt should have ActiveAttractions");
        assert!(
            !aa.entries[0].active,
            "Cell entry should be deactivated after BoltHitCell, got active={}",
            aa.entries[0].active
        );
    }

    #[test]
    fn wall_attraction_deactivates_on_bolt_hit_wall() {
        let mut app = test_app();

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                ActiveAttractions {
                    entries: vec![AttractionEntry {
                        attraction_type: AttractionType::Wall,
                        force: 10.0,
                        active: true,
                    }],
                },
            ))
            .id();

        app.insert_resource(PendingWallHit(Some(BoltHitWall {
            bolt,
            wall: Entity::PLACEHOLDER,
        })));

        tick(&mut app);

        let aa = app
            .world()
            .entity(bolt)
            .get::<ActiveAttractions>()
            .expect("bolt should have ActiveAttractions");
        assert!(
            !aa.entries[0].active,
            "Wall entry should be deactivated after BoltHitWall, got active={}",
            aa.entries[0].active
        );
    }

    #[test]
    fn all_types_reactivate_on_hit_with_non_attracted_type() {
        let mut app = test_app();

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                ActiveAttractions {
                    entries: vec![
                        AttractionEntry {
                            attraction_type: AttractionType::Cell,
                            force: 10.0,
                            active: false,
                        },
                        AttractionEntry {
                            attraction_type: AttractionType::Wall,
                            force: 5.0,
                            active: true,
                        },
                    ],
                },
            ))
            .id();

        // Breaker is NOT in the entries, so hitting breaker should reactivate all
        app.insert_resource(PendingBreakerHit(Some(BoltHitBreaker { bolt })));

        tick(&mut app);

        let aa = app
            .world()
            .entity(bolt)
            .get::<ActiveAttractions>()
            .expect("bolt should have ActiveAttractions");
        assert!(
            aa.entries.iter().all(|e| e.active),
            "all entries should be reactivated after hitting non-attracted type (Breaker), got {:?}",
            aa.entries.iter().map(|e| e.active).collect::<Vec<_>>()
        );
    }

    #[test]
    fn hit_with_own_attracted_type_deactivates_does_not_reactivate() {
        let mut app = test_app();

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                ActiveAttractions {
                    entries: vec![AttractionEntry {
                        attraction_type: AttractionType::Cell,
                        force: 10.0,
                        active: false,
                    }],
                },
            ))
            .id();

        // Cell IS in entries, so hitting cell should deactivate (already inactive), not reactivate
        app.insert_resource(PendingCellHit(Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        })));

        tick(&mut app);

        let aa = app
            .world()
            .entity(bolt)
            .get::<ActiveAttractions>()
            .expect("bolt should have ActiveAttractions");
        assert!(
            !aa.entries[0].active,
            "Cell entry should remain inactive after hitting Cell (attracted type), got active={}",
            aa.entries[0].active
        );
    }
}
