//! Portal cell behavior — invulnerable cells cleared via the portal pipeline.
//!
//! Bolt hits portal cell → `PortalEntered` → `PortalCompleted` (mock) →
//! `KillYourself<Cell>`. Real sub-node logic wired in node refactor.

pub(crate) mod components;
pub(crate) mod systems;

#[cfg(test)]
mod tests;
