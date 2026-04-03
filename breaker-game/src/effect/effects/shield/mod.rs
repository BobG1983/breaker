pub use system::{ShieldWall, ShieldWallTimer};
pub(crate) use system::{fire, register, reverse};

mod system;

#[cfg(test)]
mod tests;
