//! State plugin registration.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::StatePlugin;
