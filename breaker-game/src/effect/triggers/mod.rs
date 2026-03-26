//! Per-trigger bridge systems — translate messages into effect events.

#[cfg(test)]
pub(crate) mod test_helpers;

pub(crate) mod bolt_lost;
pub(crate) mod bump;
pub(crate) mod bump_whiff;
pub(crate) mod bumped;
pub(crate) mod cell_destroyed;
pub(crate) mod death;
pub(crate) mod destroyed_cell;
pub(crate) mod died;
pub(crate) mod early_bump;
pub(crate) mod early_bumped;
pub(crate) mod impact;
pub(crate) mod impacted;
pub(crate) mod late_bump;
pub(crate) mod late_bumped;
pub(crate) mod no_bump;
pub(crate) mod once_nodes;
pub(crate) mod perfect_bump;
pub(crate) mod perfect_bumped;
pub(crate) mod timer;

pub(crate) use bolt_lost::bridge_bolt_lost;
pub(crate) use bump::bridge_bump;
pub(crate) use bump_whiff::bridge_bump_whiff;
pub(crate) use bumped::bridge_bumped;
pub(crate) use cell_destroyed::bridge_cell_destroyed;
pub(crate) use death::{bridge_bolt_death, bridge_cell_death};
pub(crate) use destroyed_cell::bridge_destroyed_cell;
pub(crate) use died::{bridge_bolt_died, bridge_cell_died};
pub(crate) use early_bump::bridge_early_bump;
pub(crate) use early_bumped::bridge_early_bumped;
pub(crate) use impact::{bridge_breaker_impact, bridge_cell_impact, bridge_wall_impact};
pub(crate) use impacted::{bridge_breaker_impacted, bridge_cell_impacted, bridge_wall_impacted};
pub(crate) use late_bump::bridge_late_bump;
pub(crate) use late_bumped::bridge_late_bumped;
pub(crate) use no_bump::bridge_no_bump;
pub(crate) use once_nodes::apply_once_nodes;
pub(crate) use perfect_bump::bridge_perfect_bump;
pub(crate) use perfect_bumped::bridge_perfect_bumped;
pub(crate) use timer::bridge_timer_threshold;

/// Registers all trigger bridge systems with the app.
pub(crate) fn register(app: &mut bevy::prelude::App) {
    bump::register(app);
    perfect_bump::register(app);
    early_bump::register(app);
    late_bump::register(app);
    bumped::register(app);
    perfect_bumped::register(app);
    early_bumped::register(app);
    late_bumped::register(app);
    bump_whiff::register(app);
    no_bump::register(app);
    impact::register(app);
    impacted::register(app);
    bolt_lost::register(app);
    cell_destroyed::register(app);
    death::register(app);
    destroyed_cell::register(app);
    died::register(app);
    timer::register(app);
}
