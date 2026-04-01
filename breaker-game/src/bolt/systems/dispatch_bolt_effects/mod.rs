//! Dispatch bolt-defined effects to target entities.

pub(crate) use system::dispatch_bolt_effects;

mod system;

#[cfg(test)]
mod tests;
