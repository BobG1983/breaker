//! Piercing beam effect handler — fires a beam through cells in a line.
//!
//! Observes [`PiercingBeamFired`] and damages cells along the beam path.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::{
        components::{Cell, Locked},
        messages::DamageCell,
    },
    chips::components::DamageBoost,
    effect::definition::EffectTarget,
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a piercing beam effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct PiercingBeamFired {
    /// Damage multiplier for the beam.
    pub damage_mult: f32,
    /// Width of the beam in world units.
    pub width: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Observer: handles piercing beam — fires a beam along the bolt's velocity
/// direction and damages all cells within the beam width.
///
/// Algorithm:
/// 1. Extract bolt entity from targets.
/// 2. Get bolt position and velocity direction.
/// 3. For each unlocked cell, check if it lies within `width/2` of the beam
///    line and is ahead of the bolt (positive projection along direction).
/// 4. Write [`DamageCell`] for each cell hit.
pub(crate) fn handle_piercing_beam(
    trigger: On<PiercingBeamFired>,
    bolt_query: Query<(&Position2D, &Velocity2D, Option<&DamageBoost>)>,
    cell_query: Query<(Entity, &Position2D, Has<Locked>), With<Cell>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let event = trigger.event();

    let bolt_entity = event.targets.iter().find_map(|t| match t {
        EffectTarget::Entity(e) => Some(*e),
        EffectTarget::Location(_) => None,
    });
    let Some(bolt_entity) = bolt_entity else {
        return;
    };
    let Ok((bolt_pos, bolt_vel, damage_boost)) = bolt_query.get(bolt_entity) else {
        return;
    };

    let dir = bolt_vel.0.normalize_or_zero();
    if dir.length_squared() < 0.5 {
        return; // zero velocity — no beam direction
    }

    let boost = damage_boost.map_or(0.0, |b| b.0);
    let damage = BASE_BOLT_DAMAGE * (1.0 + boost) * event.damage_mult;
    let half_width = event.width / 2.0;

    for (cell_entity, cell_pos, is_locked) in &cell_query {
        if is_locked {
            continue;
        }

        // Check distance from cell to the beam line
        let to_cell = cell_pos.0 - bolt_pos.0;
        let along = to_cell.dot(dir);
        if along < 0.0 {
            continue; // behind the bolt
        }

        let perp_dist = (to_cell - dir * along).length();
        if perp_dist <= half_width {
            damage_writer.write(DamageCell {
                cell: cell_entity,
                damage,
                source_chip: event.source_chip.clone(),
            });
        }
    }
}

/// Registers all observers and systems for the piercing beam effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_piercing_beam);
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::components::Bolt,
        cells::{components::Cell, messages::DamageCell},
    };

    #[derive(Resource, Default)]
    struct CapturedDamageCell(Vec<DamageCell>);

    fn capture_damage_cell(
        mut reader: MessageReader<DamageCell>,
        mut captured: ResMut<CapturedDamageCell>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<DamageCell>()
            .init_resource::<CapturedDamageCell>()
            .add_observer(handle_piercing_beam)
            .add_systems(FixedUpdate, capture_damage_cell);
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
    fn handle_piercing_beam_does_not_panic() {
        use crate::effect::typed_events::PiercingBeamFired;

        let mut app = test_app();

        app.world_mut().commands().trigger(PiercingBeamFired {
            damage_mult: 1.5,
            width: 10.0,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }

    /// A beam fired along the bolt's velocity direction should damage all
    /// cells that lie within the beam path (within `width`).
    ///
    /// Bolt at (0,0) velocity (0,400). Cell A at (0,100), Cell B at (0,200).
    /// Both should receive `DamageCell`.
    #[test]
    fn piercing_beam_damages_cells_along_line() {
        let mut app = test_app();

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        let _cell_a = app
            .world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))))
            .id();

        let _cell_b = app
            .world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 200.0))))
            .id();

        app.world_mut().commands().trigger(PiercingBeamFired {
            damage_mult: 1.0,
            width: 20.0,
            targets: vec![EffectTarget::Entity(bolt)],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedDamageCell>();
        assert_eq!(
            captured.0.len(),
            2,
            "piercing beam through 2 cells along path should produce 2 DamageCell messages, got {}",
            captured.0.len()
        );
    }

    /// When no cells lie in the beam path, no `DamageCell` messages are produced.
    #[test]
    fn piercing_beam_no_cells_in_path_is_noop() {
        let mut app = test_app();

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        // No cells spawned — beam has nothing to hit

        app.world_mut().commands().trigger(PiercingBeamFired {
            damage_mult: 1.0,
            width: 20.0,
            targets: vec![EffectTarget::Entity(bolt)],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedDamageCell>();
        assert_eq!(
            captured.0.len(),
            0,
            "piercing beam with no cells in path should produce 0 DamageCell messages, got {}",
            captured.0.len()
        );
    }
}
