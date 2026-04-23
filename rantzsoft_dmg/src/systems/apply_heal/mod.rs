//! `apply_heal::<T>` — applies heal messages to HP with per-message ceiling.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::apply_heal;
