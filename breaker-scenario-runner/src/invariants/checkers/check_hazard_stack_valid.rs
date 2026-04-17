use bevy::prelude::*;
use breaker::hazard::resources::ActiveHazards;

use crate::{invariants::*, types::InvariantKind};

/// Checks that every entry in [`ActiveHazards`] has `stacks > 0`.
///
/// [`ActiveHazards`] documents the invariant that its `add_stack` method is
/// the only insertion path, and always increments the counter to at least 1
/// before returning. A 0-stack entry would mean a hazard kind is present in
/// the map but the `stacks()` accessor still reports 0 — a ghost entry.
///
/// A violation here indicates a bug in some other mutation path (or a
/// regression in `add_stack`) that introduced a 0-valued entry. The invariant
/// is intentionally triggered in the self-test scenario via a
/// `MutationKind::InjectZeroStackHazard` frame mutation which uses the
/// `force_insert_entry` backdoor.
pub fn check_hazard_stack_valid(
    active: Option<Res<ActiveHazards>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    let Some(active) = active else { return };

    for (kind, stacks) in active.iter() {
        if stacks == 0 {
            log.0.push(ViolationEntry {
                frame:     frame.0,
                invariant: InvariantKind::HazardStackValid,
                entity:    None,
                message:   format!(
                    "HazardStackValid FAIL frame={} kind={kind:?} stacks=0 (should be ≥1)",
                    frame.0,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use breaker::hazard::definition::HazardKind;

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
            .add_systems(FixedUpdate, check_hazard_stack_valid);
        app
    }

    #[test]
    fn skips_when_no_active_hazards_resource() {
        let mut app = test_app();
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn empty_active_hazards_no_violation() {
        let mut app = test_app();
        app.insert_resource(ActiveHazards::default());
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn healthy_stack_no_violation() {
        let mut app = test_app();
        let mut active = ActiveHazards::default();
        active.force_insert_entry(HazardKind::Decay, 3);
        app.insert_resource(active);
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn zero_stack_entry_fires_violation() {
        let mut app = test_app();
        let mut active = ActiveHazards::default();
        active.force_insert_entry(HazardKind::Decay, 0);
        app.insert_resource(active);
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::HazardStackValid);
    }
}
