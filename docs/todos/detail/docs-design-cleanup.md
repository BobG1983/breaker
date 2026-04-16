# docs/design/ Cleanup — Post-Architecture-Rewrite

The `docs/architecture/effects/` rewrite (2026-04-15/16) created authoritative homes for the technical content that was misfiled in `docs/design/`. This todo captures the remaining cleanup: removing redundant design files, stripping tech content from split files, and deleting Phase 5 work-tracking duplicates.

## Context

A full taxonomy audit (`agent-memory/general-purpose/ephemeral/audit-2026-04-15-design-taxonomy.md`) classified every file under `docs/design/`. The architecture rewrite absorbed the technical content. What remains is cleanup of the design-side originals.

## 1. Delete design/triggers/ (21 files → redirect)

**All 21 files** are pure technical descriptors with zero design content. The authoritative reference is now `docs/architecture/effects/trigger_reference.md`.

Action: Delete all files in `docs/design/triggers/` (20 individual files + `index.md`). Update `docs/design/index.md` to remove the triggers link (or point it at the architecture reference).

Files: `bolt_lost.md`, `bump_whiff.md`, `bump.md`, `bumped.md`, `cell_destroyed.md`, `death.md`, `died.md`, `early_bump.md`, `early_bumped.md`, `impact.md`, `impacted.md`, `index.md`, `late_bump.md`, `late_bumped.md`, `no_bump.md`, `node_end.md`, `node_start.md`, `node_timer_threshold.md`, `perfect_bump.md`, `perfect_bumped.md`, `time_expires.md`.

## 2. Clean design/effects/ (27 files → reduce to VFX-only or delete)

The authoritative per-effect reference is now `docs/architecture/effects/effect_reference.md`.

**20 files with zero design content** — delete or replace with a one-line redirect:
`attraction.md`, `bump_force.md`, `chain_bolt.md`, `chain_lightning.md`, `damage_boost.md`, `entropy_engine.md`, `gravity_well.md`, `lose_life.md`, `piercing_beam.md`, `piercing.md`, `pulse.md`, `ramping_damage.md`, `random_effect.md`, `second_wind.md`, `shield.md`, `shockwave.md`, `size_boost.md`, `spawn_bolts.md`, `spawn_phantom.md`, `speed_boost.md`, `time_penalty.md`.

**7 files with VFX/Ingredients sections** — strip the tech content (Parameters, Behavior, Reversal, Components), keep only VFX direction. Merge VFX paragraphs into `design/graphics/effects-particles.md` if desired:
`anchor.md`, `circuit_breaker.md`, `explode.md`, `flash_step.md`, `mirror_protocol.md`, `quick_stop.md`, `tether_beam.md`.

**effects/index.md** — strip the "Buff Stacking" table (now in architecture); keep the categorical effect listing (Combat, Bolt Spawning, etc.) as a design-level index, or delete entirely.

## 3. Delete design/graphics/catalog/ (7 files)

All 7 files are Phase 5 work-tracking artifacts that duplicate existing `docs/todos/detail/phase-5*.md` todo entries.

Action: Delete all files in `docs/design/graphics/catalog/`:
`index.md`, `entities.md`, `effects.md`, `evolutions.md`, `feedback.md`, `ui-screens.md`, `systems.md`.

## 4. Move design/graphics/data-driven-graphics.md → todo

Documents an enum-composition system (`CellShape`, `EntityVisualConfig`, `AttachVisuals`) that **does not exist in code**. Forward-looking spec for Phase 5 work.

Action: Move content into a new `docs/todos/detail/data-driven-visual-composition.md` todo, or fold into existing `phase-5i-cell-visuals.md`. Delete or redirect the design file.

## 5. Redirect design/decisions/chip-template-system.md

Content is now fully covered by `docs/architecture/content.md` (Template-Based Authoring section). The design file is redundant.

Action: Replace content with a one-line redirect: "See `docs/architecture/content.md` — Template-Based Authoring."

## 6. Split candidates (design intent stays, type names go)

Three files mix design philosophy with implementation type names. The design portion stays; the implementation details should eventually move to architecture docs for their respective domains. Low priority — these files aren't actively misleading.

- `decisions/entity-scale.md` — formula and `NodeScalingFactor`/`BaseWidth`/`BaseHeight` component names → `architecture/layout.md` or new `architecture/scaling.md`
- `decisions/node-type-differentiation.md` — `Toughness`, `TierDefinition`, `NodePool` type names → `architecture/cell-behaviors.md` or new `architecture/node-types.md`
- `decisions/chip-evolution.md` — bottom "Evolution Recipes" section duplicates `chip-catalog.md` → delete duplicate section

## 7. Update cross-references

After deletions, update:
- `docs/design/index.md` — remove triggers link, update effects link
- `docs/architecture/effects/index.md` — remove "Design Reference" link at the bottom (currently points to `design/effects/` and `design/triggers/` which may be gone)
- `docs/design/graphics/index.md` — remove catalog/ link
- `docs/design/graphics/decisions-required.md` — the "Architecture Decisions (Phase 5 specific)" section references `architecture/rendering/*.md` files that don't exist yet; leave as-is until Phase 5 lands.

## Priority

The deletions (items 1, 2, 3) are safe to batch — they remove redundant content whose authoritative home now exists. The split items (6) are low priority and can be deferred indefinitely. The cross-reference updates (7) should follow immediately after deletions.
