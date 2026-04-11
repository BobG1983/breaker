//! `EffectType` — all effect variants, each wrapping a config struct.

use serde::{Deserialize, Serialize};

use crate::effect_v3::effects::*;

/// Every effect in the game. Each variant wraps a config struct that
/// implements the `Fireable` trait. The enum is the dispatch layer;
/// the config struct is the implementation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
    /// Passive: multiply bolt speed.
    SpeedBoost(SpeedBoostConfig),
    /// Passive: multiply breaker size.
    SizeBoost(SizeBoostConfig),
    /// Passive: multiply bolt damage.
    DamageBoost(DamageBoostConfig),
    /// Passive: multiply bump force.
    BumpForce(BumpForceConfig),
    /// Passive: multiply breaker deceleration.
    QuickStop(QuickStopConfig),
    /// Toggle: enable flash step dash.
    FlashStep(FlashStepConfig),
    /// Passive: grant piercing hits.
    Piercing(PiercingConfig),
    /// Passive: multiply incoming damage.
    Vulnerable(VulnerableConfig),
    /// Passive: accumulate ramping damage.
    RampingDamage(RampingDamageConfig),
    /// Spawned: attraction steering toward entities.
    Attraction(AttractionConfig),
    /// Spawned: anchor bolt in place.
    Anchor(AnchorConfig),
    /// Spawned: periodic shockwave emitter.
    Pulse(PulseConfig),
    /// Spawned: shield wall protection.
    Shield(ShieldConfig),
    /// Spawned: second wind wall.
    SecondWind(SecondWindConfig),
    /// Spawned: expanding damage shockwave.
    Shockwave(ShockwaveConfig),
    /// Fire-and-forget: area explosion.
    Explode(ExplodeConfig),
    /// Spawned: chain lightning arcs.
    ChainLightning(ChainLightningConfig),
    /// Fire-and-forget: piercing beam line.
    PiercingBeam(PiercingBeamConfig),
    /// Fire-and-forget: spawn extra bolts.
    SpawnBolts(SpawnBoltsConfig),
    /// Spawned: spawn phantom bolt with limited lifetime.
    SpawnPhantom(SpawnPhantomConfig),
    /// Fire-and-forget: chain bolt redirect.
    ChainBolt(ChainBoltConfig),
    /// Fire-and-forget: mirror protocol bolt duplication.
    MirrorProtocol(MirrorConfig),
    /// Spawned: tether beam damage link.
    TetherBeam(TetherBeamConfig),
    /// Spawned: gravity well pulling bolts.
    GravityWell(GravityWellConfig),
    /// Fire-and-forget: lose a life.
    LoseLife(LoseLifeConfig),
    /// Fire-and-forget: add time penalty.
    TimePenalty(TimePenaltyConfig),
    /// Fire-and-forget: kill entity.
    Die(DieConfig),
    /// Stateful: circuit breaker counter.
    CircuitBreaker(CircuitBreakerConfig),
    /// Stateful: entropy engine counter.
    EntropyEngine(EntropyConfig),
    /// Fire-and-forget: pick a random effect.
    RandomEffect(RandomEffectConfig),
}
