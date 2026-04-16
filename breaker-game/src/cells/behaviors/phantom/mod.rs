//! Phantom cell behavior — cells cycle through Solid, Telegraph, and Ghost
//! phases. During Ghost phase, collision layers are zeroed so bolts pass
//! through. Telegraph is a warning phase (still collidable).

pub(crate) mod components;
pub(crate) mod systems;

#[cfg(test)]
mod tests;
