//! Second wind systems — despawn wall on first bolt reflection.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::despawn_on_first_reflection;
