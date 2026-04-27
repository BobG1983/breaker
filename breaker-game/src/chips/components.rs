//! Chip effect components stamped onto bolt and breaker entities.
//!
//! Legacy flat-stat components have been removed. Effect state is now managed
//! through `rantzsoft_dmg` damage stacks (`DamageBoostStack`,
//! `VulnerableStack`) and `effect_v3::stacking::EffectStack<T>` for
//! time-based effect stacks.
