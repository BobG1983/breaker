//! Apply per-bolt force messages as velocity deltas each `FixedUpdate`.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::apply_bolt_forces;
