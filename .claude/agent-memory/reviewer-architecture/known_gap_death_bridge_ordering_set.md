---
name: Death bridges no longer in EffectV3Systems::Bridge set
description: on_*_destroyed bridges moved to .after(DeathPipelineSystems::HandleKill); not in any cross-domain SystemSet; docs/architecture/ordering.md is stale
type: project
---

As of Wave 1 of the New Cell Modifiers feature (Volatile), the four death-trigger
bridge systems were moved out of `EffectV3Systems::Bridge` and now live solely
under `.after(DeathPipelineSystems::HandleKill)`:

- `on_cell_destroyed`
- `on_bolt_destroyed`
- `on_wall_destroyed`
- `on_breaker_destroyed`

Registration site: `breaker-game/src/effect_v3/triggers/death/register.rs`

**Why:** Latent bug fix — previously the bridges ran in `EffectV3Systems::Bridge`
which precedes the death pipeline within a tick. That meant `Trigger::Died` was
evaluated against the previous tick's `Destroyed<T>` messages, so victims that
died this tick had their effects fire one tick late. Moving the bridges to
`.after(HandleKill)` makes them read same-tick `Destroyed<T>` messages.

**Side effect:** Effects fired by death bridges (e.g., volatile cell explosions
that fire `DamageDealt<Cell>`) now apply on the *next* FixedUpdate tick instead
of the same tick, because they are queued *after* `ApplyDamage` ran. This is
acceptable for chain reactions but every test/scenario that asserts on chain
timing must now expect a 1-tick delay between source death and target damage.

**How to apply:**

1. **Stale docs.** `docs/architecture/ordering.md:40` and lines 169-178 still
   list the four bridges as members of `EffectV3Systems::Bridge`. This must be
   corrected. `docs/architecture/effects/death_pipeline.md` step 7 also needs
   the `.after(HandleKill)` annotation.

2. **Missing ordering anchor.** The bridges are no longer in any cross-domain
   SystemSet. If any future system needs to order against death-trigger
   evaluation, there is no anchor. Recommend adding
   `EffectV3Systems::Death` (or `DeathBridges`) and tagging the four bridges
   with it. Do NOT silently drop the cross-domain ordering surface that the
   previous Bridge membership accidentally provided.

3. **Compatible consumers.** All known same-tick consumers of `Destroyed<T>`
   (`track_cells_destroyed`, `check_lock_release`, `detect_mass_destruction`,
   `detect_combo_king`, `track_node_completion`) also order
   `.after(HandleKill)` — they run in the same parallel batch as the bridges.
   No write conflicts exist (all are message readers).
