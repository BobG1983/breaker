# Sympathy: design doc says logic lives in hazard domain, not cells domain

## Problems addressed

- `audit/hazards/sympathy.md` Issue 1 \u2014 Design \u00a7Systems says "No hazard-domain runtime systems for the healing logic. The cells domain's `apply_damage::<Cell>` reads `SympathyConfig` and handles it." Impl correctly keeps Sympathy in the hazard domain (`sympathy_heal_adjacent` reads `DamageDealt<Cell>`). Design doc is wrong; update to match the cleaner architectural split.

Other Sympathy issues covered by existing remediations:
- Issue 2 (branching-graph attenuation) \u2014 `diffusion-flat-share-per-ring.md` (same per-ring pattern).
- Issue 3 (ADJACENCY_RADIUS_SQ) \u2014 `grid-orthogonal-adjacency.md`.
- Issue 4 (cleanup) \u2014 `run-end-config-cleanup.md` (Sympathy in "no state resources" list).

## Remediation

Open `docs/todos/detail/mod-system-design/hazards/sympathy.md`.

\u00a7Systems \u2014 replace the "no hazard-domain runtime systems" language with:

> **`sympathy_heal_adjacent`** (runs in `FixedUpdate`, set `DeathPipelineSystems::ApplyHeal`, ordered `.before(apply_heal::<Cell>)`).
>
> Reads `DamageDealt<Cell>` messages (post-damage-transformation \u2014 after Diffusion, if active, has modified the damage values). For each damaged cell, BFS outward through adjacent live cells and emit `HealDealt<Cell>` messages with geometric ring attenuation. The heal is applied same-tick via `apply_heal::<Cell>`.
>
> Architectural note: Sympathy's logic lives in the hazard domain rather than the cells domain because it REACTS to damage already applied (via messages), whereas Diffusion MODIFIES damage before application (and so must live in the cells domain's `apply_damage_to_cells`). The asymmetry reflects the distinct pipeline responsibilities.

\u00a7Architecture \u2014 append:

> Sympathy reads `DamageDealt<Cell>` after Diffusion has transformed it. Cross-reference `sympathy-reads-post-diffusion-damage.md` for the ordering pin and the rationale ("Sympathy is slightly less potent when Diffusion is active").

No code change. No test change. Doc-only.
