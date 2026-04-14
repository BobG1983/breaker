//! Death pipeline systems module — production code plus tests.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{apply_damage, detect_deaths, handle_kill, process_despawn_requests};
