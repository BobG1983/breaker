//! Breaker archetype behavior system — data-driven trigger→consequence dispatch.

pub mod bolt_speed_boost;
pub mod definition;
pub mod life_lost;
pub mod registry;

pub use definition::ArchetypeDefinition;
pub use registry::ArchetypeRegistry;
