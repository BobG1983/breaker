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
