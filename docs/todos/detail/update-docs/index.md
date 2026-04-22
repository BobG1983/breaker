# Update the docs — design-doc alignment sweep

Pure documentation work. Assumes TODOs #1-#9 are complete — every spec in this folder describes the **post-TODO target state** for `docs/design/` and does not reference the pre-rationalisation `audit/` remediation list or the now-defunct `docs/todos/detail/mod-system-design/` tree.

## Why this is TODO #0

Single source of truth. Before this sweep: two locations describe each protocol/hazard — the stale pre-TODO baseline at `docs/todos/detail/mod-system-design/` and the delta at `docs/todos/detail/fix-broken-protocols-and-hazards/`. Spec writers implementing later TODOs would hit conflicting instructions.

After this sweep: `docs/design/{protocols,hazards}/` holds canonical design docs aligned with the post-TODO target. `mod-system-design/` is deleted.

Lands first because stale docs cost context re-verification on every read and because every downstream TODO assumes the canonical design is at `docs/design/`.

## Structure

Each file in this folder is a self-contained target-state spec for one section of docs. Files tell the writer: what the canonical doc must say, what it must NOT say, and why.

### Wholesale promotion

| File | Scope |
|------|-------|
| `promote-mod-system-design.md` | Write 31 canonical per-mechanic files at `docs/design/{protocols,hazards}/*.md`. Source: `docs/todos/detail/mod-system-design/{protocols,hazards}/*.md` (cleansed per the substitution table). Apply the per-mechanic target-state specs below on top where one exists. Also writes `docs/design/protocols/index.md` + `docs/design/hazards/index.md` + updates `docs/design/index.md`. Deletes `mod-system-design/` at the end. |

### Per-mechanic target-state specs

Each of these specs narrows the canonical doc for one mechanic. The promotion writer reads BOTH the mod-system-design source (for structural content — Game Design / Config / Components / Systems / Expected Behaviors / Edge Cases) AND this file (for target-state-specific corrections) and produces the canonical output.

| File | Target doc | Concern |
|------|------------|---------|
| `iron-curtain-design-doc-abs-symmetric.md` | `docs/design/protocols/iron_curtain.md` | Abs-symmetric falloff wording in § Edge Cases |
| `reckless-dash-design-doc-bump-performed.md` | `docs/design/protocols/reckless_dash.md` | `BumpPerformed` (graded) as trigger, not `BoltImpactBreaker` |
| `tier-regression-design-doc-once-per-run.md` | `docs/design/protocols/tier_regression.md` | "Once per run after selection" — convention matches every other protocol |
| `renewal-design-doc-alignment.md` | `docs/design/hazards/renewal.md` | 2-system shape with `attach_renewal_timer` + `renewal_tick`; `Added<Cell>` attach; field renames |
| `sympathy-design-doc-alignment.md` | `docs/design/hazards/sympathy.md` | `sympathy_heal_adjacent` in `DeathPipelineSystems::EmitHeal`; post-apply reactor framing |
| `gravity-surge-design-doc-multiplicative-formula.md` | `docs/design/hazards/gravity_surge.md` | Multiplicative formula; fractional scaling; `ApplyBoltForce` pipeline |
| `erosion-design-doc-effect-stack-pattern.md` | `docs/design/hazards/erosion.md` | `EffectStack<SizeBoostConfig>` reconciliation, X-only; drop `ApplyBreakerShrink` |
| `fracture-design-doc-pins.md` | `docs/design/hazards/fracture.md` | Cap-at-4 + fixed-order position selection |
| `resonance-design-doc-alignment.md` | `docs/design/hazards/resonance.md` | Full Wave/Tracker/ActiveSlows component shapes; kills-beyond-threshold drain semantics |
| `diffusion-flat-share-documentation.md` | `docs/design/hazards/diffusion.md` | Flat-share-per-ring attenuation as deliberate simplification; pre-apply mutator pipeline position |
| `haste-effect-stack-canonical.md` | `docs/design/hazards/haste.md` | `EffectStack<SpeedBoostConfig>` reconciliation as canonical |
| `overcharge-design-doc-alignment.md` | `docs/design/hazards/overcharge.md` | Fractional fields; lazy insertion (not `Added<T>` — Overcharge counts kills, not spawn events) |

## Ordering within the sweep

Land in any order — every file is self-contained and targets a distinct destination file. The promotion writer consumes all per-mechanic specs during a single pass over its 31 source files. Can run as one parallel wave of workers or as a single sequential pass.

## Scope boundary

**In scope**:
- `docs/design/protocols/**` (new tree — written during promotion).
- `docs/design/hazards/**` (new tree — written during promotion).
- `docs/design/index.md` (add entries for the two new trees).
- `docs/todos/detail/mod-system-design/**` (deleted at end of sweep).
- Every cross-ref in the repo that points at `docs/todos/detail/mod-system-design/` (~50 files — doc comments in `breaker-game/src/`, TODO detail files, architecture docs, `session-state.md`).

**Out of scope**:
- Any code, RON, or test change.
- `CLAUDE.md` or `.claude/rules/*`.
- Redesign of any mechanic. Cleansing is pattern substitution, not re-design.
- Architecture-doc updates for TODO-owned patterns (e.g., damage-mutator chain, `Added<T>` attach, mutators/ consolidation) — those ride with their owning TODO.

## Reference

- Source content: `docs/todos/detail/mod-system-design/{protocols,hazards}/` (31 files).
- Canonical target: `docs/design/{protocols,hazards}/` (31 files + 2 indexes).
- Format reference: `docs/design/effects/index.md` (index-file template).

Dependencies: none — doc-only, can land standalone.
