//! Mutators plugin — protocols + hazards consolidated.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::MutatorsPlugin;
// Test-only re-export — per-mechanic scheduling/wire tests in
// `mutators/{hazards,protocols}/<mech>/tests/` call `wire_damage_chain`
// after `<mech>::wire(app)` to install the central chain. See
// `docs/architecture/creating-a-mutator.md` §10. Gated `#[cfg(test)]`
// so production callers cannot bypass `MutatorsPlugin::build`.
#[cfg(test)]
pub(crate) use system::wire_damage_chain;
