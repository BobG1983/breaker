pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::SecondWindWall;
pub(crate) use system::{fire, register, reverse};
