---
name: Effect System Coverage Map
description: Which effects have scenario coverage and which are completely untested — updated after Phase 4+5 audit
type: project
---

## Effects with Scenario Coverage (Phase 4+5 state)

| Effect | Scenario(s) | Quality |
|--------|-------------|---------|
| SpeedBoost | surge_speed_stress, impacted_wall_speed, overclock_until_speed, initial_effects_bolt, passive_chips_chaos | Good — multiple triggers, stress, Until reversal |
| DamageBoost | passive_chips_chaos | Minimal — only Bumped trigger, no reversal |
| Piercing | passive_chips_chaos | Minimal — no scenario verifying cell pass-through count |
| SizeBoost | passive_chips_chaos | Minimal — applied but no size invariant checks it |
| Shockwave | surge_overclock, cascade_shockwave_stress, supernova_chain_stress, entropy_engine_stress, flux_random_chaos | Good |
| SpawnBolts | spawn_bolts_stress, supernova_chain_stress, entropy_engine_stress | Good |
| ChainBolt | tether_chain_bolt_stress | Good |
| EntropyEngine | entropy_engine_stress | Good |
| RandomEffect | flux_random_chaos | Good |

## Effects with NO Scenario Coverage

- **SecondWind** — zero scenarios. Critical gap: unit tests pass but integration (bolt hits wall, wall despawns, bolt not lost) never run under scenario runner.
- **Attraction** — zero scenarios. Novel physics behavior (velocity steering), entity cleanup on type deactivation completely untested.
- **SpawnPhantom** — zero scenarios. Infinite piercing + BoltLifespan cleanup races untested.
- **Shield** — zero scenarios. Breaker bolt-loss immunity path never exercised in runner.
- **GravityWell** — zero scenarios. Velocity mutation + duration timer + max cap untested in runner.
- **Pulse** — zero scenarios. PulseRing spawn/expand/despawn lifecycle under load untested.
- **Explode** — zero scenarios. ExplodeRequest spawn-then-despawn lifecycle untested in runner.
- **PiercingBeam** — zero scenarios. Beam entity lifecycle untested in runner.
- **ChainLightning** — zero scenarios. Arc jump pattern + entity cleanup untested.
- **RampingDamage** — zero scenarios. Accumulation + NoBump reset behavior untested.
- **QuickStop** — zero scenarios. Breaker deceleration behavior untested.
- **BumpForce** — zero scenarios. Force multiplier accumulation untested.
- **TetherBeam** (mechanic) — tether_chain_bolt_stress only tests ChainBolt, not TetherBeam specifically.
- **TimePenalty** — zero scenarios (chrono scenarios don't exercise it via initial_effects).
- **LoseLife** — zero scenarios (aegis_lives_exhaustion uses organic bolt loss, not LoseLife effect).

## Triggers with NO Scenario Coverage

- NoBump — never used in any initial_effects block
- BumpWhiff — never used in any initial_effects block
- Death / Died — never used in any initial_effects block
- NodeStart / NodeEnd — never used in any initial_effects block
- NodeTimerThreshold — never used in any initial_effects block
- BoltLost — never used in any initial_effects block (godmode_breaker uses it as self-test only)
- EarlyBump / EarlyBumped / LateBump / LateBumped — only in self-test perfect_input_* scenarios, not in mechanic/stress scenarios with initial_effects

**Why:** These trigger gaps mean entire branches of the effect bridge systems are never exercised under load.

## Invariant Gaps

Properties with no invariant checker:
- Active bolt count > configured max_bolt_count: partially covered by BoltCountReasonable but not per-effect
- ShieldActive on Breaker protects bolt loss: no invariant for this
- SecondWind wall entity count (should be 0 or 1): no invariant
- GravityWell entity count vs max cap: no invariant
- PulseRing entity count not unbounded: no invariant
- RampingDamage accumulated bonus is NaN-free and non-negative: NoNaN partially covers this
- EffectiveSpeedMultiplier, EffectiveSizeMultiplier, EffectiveDamageMultiplier correctness: no invariant verifying Effective* components match expected product formula

**How to apply:** When writing scenarios for these effects, flag missing invariants as HIGH priority.
