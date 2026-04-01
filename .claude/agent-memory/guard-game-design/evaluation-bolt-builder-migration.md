---
name: Bolt Builder Migration Design Evaluation
description: Directional steering (attraction/gravity well), BreakerReflectionSpread rename, PrimaryBolt marker — all approved against all pillars
type: project
---

## Evaluation: Bolt Builder Migration (2026-03-31)

### Directional Steering (Attraction + Gravity Well) — APPROVED

Changed from additive force (accumulates velocity magnitude) to directional steering (normalize direction, apply formula speed). Key line: `spatial.velocity.0 = (spatial.velocity.0 + steering).normalize_or_zero(); apply_velocity_formula(...)`.

**Why this matters for feel**: Bolt always moves at formula speed. Gravity wells and attraction bend the path without creating sluggish moments. Old model caused deceleration when fighting pull — dead air. New model: full-speed curves like a banked turn.

**Skill ceiling raised**: Steering rate is predictable, so experts can anticipate curve radius and time bumps. Trajectories are geometrically learnable.

**Synergy cleaned up**: SpeedBoost + Attraction no longer has confusing additive-vs-multiplicative interaction. SpeedBoost makes bolt faster along whatever curve attraction creates. Clean composition.

**Carried forward**: Attraction(Breaker) force cap concern is MORE acute under steering model because bolt never slows down while curving toward breaker. Keep force tight or Legendary-only.

### BreakerReflectionSpread Rename — APPROVED

MaxReflectionAngle was ambiguous. BreakerReflectionSpread communicates it's a breaker property controlling the reflection fan width. Aligns with TiltControl chip design language.

### PrimaryBolt Marker — APPROVED

Explicit positive marker replaces negative filter (`Without<ExtraBolt>`). Enables clean queries for primary bolt in MirrorProtocol, bolt-lost detection, and future chips that distinguish primary vs spawned bolts. Infrastructure for future synergy design (primary-only effects vs all-bolt effects).

**Why:** Tracks design evaluation of a significant architectural migration to prevent revisiting approved decisions.
**How to apply:** Reference when evaluating future attraction/gravity well tuning, bolt type distinctions, or breaker reflection chips.
