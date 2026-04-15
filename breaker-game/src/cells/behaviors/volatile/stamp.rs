//! Volatile stamp: the `BoundEffects` tree a volatile cell carries, and the
//! source key under which it is registered.
//!
//! Extracted from `cells/builder/core/terminal.rs` so the generic cell builder
//! does not need to know about `effect_v3` internals (`ExplodeConfig`,
//! `EffectType`, `Tree`, `Trigger`, `OrderedFloat`). Keeping the
//! behavior→effect mapping next to the Volatile module preserves the
//! plugin-per-domain boundary: only the `volatile` sub-module touches
//! `effect_v3` types.

use ordered_float::OrderedFloat;

use crate::effect_v3::{
    effects::ExplodeConfig,
    types::{EffectType, Tree, Trigger},
};

/// `BoundEffects` stamp source key for volatile detonations.
pub(crate) const STAMP_SOURCE: &str = "volatile";

/// Builds the `When(Died, Fire(Explode(..)))` tree that a volatile cell stamps
/// at spawn time.
pub(crate) fn volatile_tree(damage: f32, radius: f32) -> Tree {
    Tree::When(
        Trigger::Died,
        Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
            range:  OrderedFloat(radius),
            damage: OrderedFloat(damage),
        }))),
    )
}
