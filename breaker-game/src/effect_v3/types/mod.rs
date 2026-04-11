//! Effect type definitions — enums and structs.

mod attraction_type;
mod bump_status;
mod condition;
mod effect_type;
mod entity_kind;
mod participants;
mod reversible_effect_type;
mod root_node;
mod route_type;
mod scoped_terminal;
mod scoped_tree;
mod stamp_target;
mod terminal;
mod tree;
mod trigger;
mod trigger_context;

pub use attraction_type::AttractionType;
pub use bump_status::BumpStatus;
pub use condition::Condition;
pub use effect_type::EffectType;
pub use entity_kind::EntityKind;
pub use participants::{BoltLostTarget, BumpTarget, DeathTarget, ImpactTarget, ParticipantTarget};
pub use reversible_effect_type::ReversibleEffectType;
pub use root_node::RootNode;
pub use route_type::RouteType;
pub use scoped_terminal::ScopedTerminal;
pub use scoped_tree::ScopedTree;
pub use stamp_target::StampTarget;
pub use terminal::Terminal;
pub use tree::Tree;
pub use trigger::Trigger;
pub use trigger_context::TriggerContext;
