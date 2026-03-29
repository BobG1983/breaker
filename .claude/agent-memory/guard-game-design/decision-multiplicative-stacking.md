---
name: Multiplicative Stacking Approved
description: Phase 3 changed stat effects from additive to multiplicative stacking; evaluated and approved against all design pillars
type: project
---

Stat effects now use multiplicative stacking (base * product(all_multipliers)) instead of the old additive model (base + flat_boost). Applies to DamageBoost, SpeedBoost, SizeBoost, BumpForce, QuickStop. Piercing remains additive (counted resource, not a scaling modifier).

**Why:** Multiplicative stacking creates the Balatro-style multiplication chain that makes build-crafting exciting. It rewards diverse chip selection over single-stat stacking (diminishing percentage returns on same-type multipliers push toward cross-axis diversity). It makes timed triggers (Overclock, Surge) dramatically more powerful when layered with passive boosts, rewarding skill expression. The power curve has a visible "knee" where the build comes online, matching Pillar 1 (Escalation) and Pillar 2 (Build the Broken Build).

**How to apply:** All future chip designs should assume multiplicative stacking. Chip values in the 1.x format (1.1 = 10% boost, 2.0 = double). Power ceiling is managed through max_taken caps and pool depletion, NOT through the stacking formula. When evaluating new chip values, trace the full multiplicative chain with realistic builds to check feel.

Key concern flagged: the multiplication should be VISIBLE on the bolt (glow intensity, trail length, color shift for stacked multipliers) and the moment of compounding should have a visual/audio pop. Not yet implemented but essential for juice.
