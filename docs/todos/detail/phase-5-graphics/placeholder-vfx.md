# Placeholder VFX for every deferred visual feedback item

## Problems addressed

- `audit/hazards/drift.md` Issue 4 \u2014 Drift wind-direction visual telegraph deferred to Phase 5 VFX.
- `audit/hazards/echo_cells.md` Issue 4 \u2014 Ghost cell visual distinction deferred to Phase 5 VFX.
- Cross-cutting pattern \u2014 every hazard/protocol that deferred visual feedback to Phase 5 ships with no visual at all. User direction: "we should have *SOME* placeholder VFX, it just doesn't have to be good or polished. It just needs to be simple."

This file is the placeholder-VFX sweep: every deferred item gets a minimal, unpolished visual so the player can perceive the effect without waiting for the polished Phase 5 pass.

## Rule

A placeholder VFX is SIMPLE: a tint, a sprite rotation, a small offset, a text overlay \u2014 anything that makes the effect visible. It is NOT a particle system, shader, or animation curve. Those belong to Phase 5.

## Per-item placeholders

### Drift \u2014 wind direction arrow

Spawn a simple line or triangle entity anchored to a fixed playfield corner showing the current wind direction. Updates when `DriftWind.direction` changes. One text label showing "wind" next to the arrow is acceptable. Color: neutral white/gray. Size: small (under 5% of playfield width).

Implementation notes:
- Entity spawned `OnEnter(NodeState::Playing)` when Drift is active; despawned `OnExit`.
- `drift_update_wind` already updates `DriftWind`; a new `drift_update_arrow_visual` system in `Update` (not FixedUpdate, since this is purely visual) reads `DriftWind` and rotates the arrow entity.

### Echo Cells \u2014 ghost tint

Ghost cells get a fixed low-alpha tint (e.g., 50% alpha white overlay, or a cyan tint at 70% opacity). Applied via the cell builder's `.ghost()` transition: when `GhostCell` is present, the cell's `Sprite.color` is shifted. No animation; just a static color change that distinguishes ghosts from normal cells at a glance.

Implementation notes:
- The cell builder's `.ghost()` transition inserts a sentinel `GhostCell` marker AND sets an initial tint color on the cell's `Sprite` (or `MeshMaterial2d` \u2014 match the project's existing tinting pattern for other cell kinds).
- No runtime tint-update system needed \u2014 the tint is fixed at spawn.

### Other deferred items (cross-reference)

When the remediation sweep reaches the following files, add placeholder VFX notes under each:

- **Resonance** \u2014 slowed cells get a dim color overlay.
- **Overcharge** \u2014 overcharged bolt gets a brighter tint.
- **Renewal** \u2014 revived cells briefly flash on respawn (single-frame tint flicker).
- **Volatility** \u2014 volatile cells get a red tinge.
- **Momentum** \u2014 no visual needed; it's a speed effect the player perceives kinetically.
- **Haste** \u2014 no visual; the player perceives it in timer UI.
- **Echo Strike / Ricochet / Anchor / Kickstart / Deadline** \u2014 each bolt with one of these effects gets a source-tagged tint overlay (different tint per protocol). When multiple protocols tint the same bolt, the tints layer (additive or multiplicative \u2014 pick one and document).

Each per-item placeholder is added in the audit-file remediation for that item, cross-referencing this file for the convention.

### Anti-principle: no scope creep

Placeholder VFX is SIMPLE. If a placeholder requires a particle system, shader compilation, animation curves, or multi-frame keyframing, it is not a placeholder \u2014 push it to Phase 5.

### Removal plan

When Phase 5 VFX for a specific item lands, its placeholder is replaced (not incrementally enhanced). The placeholder exists to close the gameplay-feedback gap NOW; the polished visual replaces it wholesale.

### Design-doc updates

For every hazard/protocol that gains a placeholder, add a short \u00a7Visual Feedback (Placeholder) section to its design doc describing the placeholder behavior. When Phase 5 lands, the section is rewritten or promoted to \u00a7Visual Feedback.

### Tests

Placeholders are not unit-tested. They're perceptible during play; if a placeholder breaks, the developer sees it and fixes it. A single integration-style smoke test per placeholder entity ("entity spawns when hazard/protocol active, despawns on exit") is sufficient.
