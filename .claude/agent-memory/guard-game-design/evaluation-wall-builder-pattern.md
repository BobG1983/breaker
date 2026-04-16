---
name: Wall Builder Pattern Design Evaluation
description: Typestate builder, WallDefinition RON, WallRegistry, WallSize removal, per-side definitions, Visible/Invisible visual dimension — all approved, no design constraints
type: project
---

## Evaluation: Wall Builder Pattern (2026-04-02)

### Approved — No Design Concerns

Pure structural infrastructure completing the builder pattern trilogy (bolt, breaker, wall). No gameplay behavior changes. Opens design space for effect-bearing walls.

### Key Findings

1. **Per-side definitions via typestate (Left/Right/Ceiling/Floor) unlock asymmetric playfield design.** Each side can reference a different WallDefinition from WallRegistry. Today all use "Wall"; future node layouts can use distinct wall types per side for variety.

2. **`effects: Vec<RootNode>` on WallDefinition plugs directly into existing effect system.** StampTarget::ActiveWalls and StampTarget::EveryWall already exist. Wall-bound effects compose with all chips targeting walls. No new plumbing required for wall-specific chips.

3. **Floor-only Lifetime restriction (Timed/OneShot) is correct.** Timed side walls that disappear mid-node would be confusing (Pillar 5). Timed floor walls create tension. SecondWind migration to `Wall::builder().floor().one_shot()` is clean.

4. **WallSize removal is correct.** Empty struct with zero readers. Dead component causing archetype fragmentation.

5. **Visible/Invisible visual dimension enables effect-bearing wall juice.** HDR color_rgb with values >1.0 designed for bloom. Shield walls, SecondWind flash, glowing effect walls all architecturally possible.

6. **WallRegistry as SeedableRegistry enables hot-reload.** RON-based wall definitions from `assets/walls/`, hot-reloadable for rapid iteration on wall effects.

### Design Opportunities Unlocked (ordered by identity impact)

- **Ricochet walls**: SpeedBoost on wall impact. Escalation pillar — bolt accelerates with each wall bounce within a node.
- **Magnetized walls**: Attraction(Wall) effects create curved trajectories. Skill ceiling — experts predict curve radius.
- **Fragile walls**: OneShot/Timed side walls that change playfield geometry mid-node. Environmental tension.
- **Wall-specific chips**: Chips targeting AllWalls to modify wall behavior globally. Synergy with existing effect system.

### Pattern Consistency

Wall builder follows identical typestate pattern as bolt builder (approved 2026-03-31) and breaker builder (approved 2026-04-02). All three core entity domains now have consistent data-driven construction.

**Why:** Tracks design evaluation of wall builder infrastructure to prevent revisiting.
**How to apply:** Reference when designing wall-specific effects, wall-bearing chips, asymmetric node layouts, or visible wall rendering (Phase 5j / Shield).
