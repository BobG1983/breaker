pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::ScenarioLifecycle;
#[cfg(test)]
pub(crate) use system::playing_state_gate;
