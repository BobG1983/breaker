//! Bevy-schedule-introspection helper for set-membership assertions.
//!
//! `system_in_set` returns `true` iff the given system function is a member
//! of the given `SystemSet` in the given schedule. Used by W5 scheduling
//! tests to assert that damage-emitting systems carry the
//! `DmgSystems::EmitDamage` tag (and that whitelist systems stay in
//! `DmgSystems::PostApplyDamage` and out of `DmgSystems::EmitDamage`).
//!
//! Approach:
//! 1. Convert `system_fn` to its `SystemTypeSet<FunctionSystem<Marker, (), (), F>>`
//!    via the blanket `IntoSystemSet<(IsFunctionSystem, Marker)>` impl. Every
//!    function system Bevy schedules is automatically a member of its
//!    `SystemTypeSet`.
//! 2. Convert `set` to its interned form.
//! 3. `Schedule::initialize(world)` — required before `systems_in_set` returns
//!    anything other than `ScheduleError::Uninitialized`.
//! 4. For each schedule set, call `ScheduleGraph::systems_in_set(interned)`
//!    to get the `IndexSet<SystemKey>`. If the function's `SystemKey`
//!    membership is a non-empty subset of the target set's, the function is
//!    in the target set.
//!
//! Takes `&mut App` because `Schedule::initialize` mutates the world and the
//! schedule. Cheap to call from tests.

use bevy::{
    ecs::schedule::{IntoSystemSet, ScheduleLabel},
    prelude::*,
};

/// Returns `true` if `system_fn` is a member of `set` in `schedule`.
///
/// Returns `false` when:
/// - The schedule doesn't exist on the app.
/// - The system is not registered in this schedule at all.
/// - The system is registered but its `SystemTypeSet` does not overlap the
///   target set.
/// - The target set has no systems configured (`SetNotFound`).
/// - Schedule initialization fails for any reason (ambiguity errors, graph
///   build errors, etc.) — `Schedule::initialize` errors are swallowed and
///   produce a `false` return. This is acceptable in test contexts: a
///   schedule that can't initialize also can't run, so no membership claim
///   is meaningful.
pub(crate) fn system_in_set<M1, M2, S1, S2>(
    app: &mut App,
    schedule: impl ScheduleLabel,
    system_fn: S1,
    set: S2,
) -> bool
where
    S1: IntoSystemSet<M1>,
    S2: IntoSystemSet<M2>,
{
    let fn_set = system_fn.into_system_set().intern();
    let target_set = set.into_system_set().intern();
    let schedule_label = schedule.intern();

    // Only proceed if the schedule exists — otherwise nothing is in it.
    if !app.world().resource::<Schedules>().contains(schedule_label) {
        return false;
    }

    app.world_mut()
        .schedule_scope(schedule_label, |world, schedule| {
            // `systems_in_set` returns `ScheduleError::Uninitialized` if the
            // schedule's graph is `changed`. Initialize it in-place so we can
            // query the graph without running systems.
            if schedule.initialize(world).is_err() {
                return false;
            }
            let graph = schedule.graph();

            let Ok(fn_keys) = graph.systems_in_set(fn_set) else {
                return false;
            };
            if fn_keys.is_empty() {
                return false;
            }
            let Ok(target_keys) = graph.systems_in_set(target_set) else {
                return false;
            };
            // The function's SystemTypeSet typically contains exactly one
            // `SystemKey`. The function is in the target set iff every key in
            // the function's set is also in the target set.
            fn_keys.iter().all(|key| target_keys.contains(key))
        })
}
