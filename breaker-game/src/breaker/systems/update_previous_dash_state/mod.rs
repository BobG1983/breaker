//! `update_previous_dash_state` — snapshots `DashState` into `PreviousDashState`
//! each `FixedUpdate` tick, after `BreakerSystems::UpdateState` has run.

mod system;

pub(crate) use system::update_previous_dash_state;

#[cfg(test)]
mod tests;
