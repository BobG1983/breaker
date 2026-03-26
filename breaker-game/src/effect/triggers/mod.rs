//! Per-trigger bridge systems — translate messages into effect events.

pub(crate) mod on_bolt_lost;
pub(crate) mod on_bump;
pub(crate) mod on_death;
pub(crate) mod on_impact;
pub(crate) mod on_no_bump;
pub(crate) mod on_timer;

// Re-export all bridge systems for plugin registration
pub(crate) use on_bolt_lost::bridge_bolt_lost;
pub(crate) use on_bump::{bridge_bump, bridge_bump_whiff};
pub(crate) use on_death::{
    apply_once_nodes, bridge_bolt_death, bridge_cell_death, cleanup_destroyed_bolts,
    cleanup_destroyed_cells,
};
pub(crate) use on_impact::{bridge_breaker_impact, bridge_cell_impact, bridge_wall_impact};
pub(crate) use on_no_bump::bridge_no_bump;
pub(crate) use on_timer::bridge_timer_threshold;
