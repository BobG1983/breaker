# Fix broken protocols and hazards

Per-mechanic remediation of every protocol and hazard that has broken or incorrect runtime behavior. Assumes all preceding TODOs have landed:

- TODO #0 — `rantzsoft_dmg` crate (unified death pipeline, `DamageBoostStack`, `VulnerableStack`)
- TODO #1 — `mutators/` domain (protocols + hazards consolidated, `wire_damage_chain`)
- TODO #4 — Phantom breaker refactor
- TODO #5 — Phantom bolt refactor
- TODO #6 — Stamp dispatcher unification + code-driven Anchor/Deadline/Kickstart/Ricochet
- TODO #7 — Bolt force message pipeline

## Scope

- Code fixes, RON tuning fixes, mechanic corrections, tests.
- Excludes pure design-doc alignment (those are a separate sweep).
- Excludes cross-cutting infrastructure (already covered by preceding TODOs).
- Excludes mechanics fully fixed by preceding TODOs (Anchor, Deadline, Kickstart, Ricochet, Afterimage/phantom, Greed, bolt-loss, Drift/Gravity Surge force).

## Structure

One markdown file per mechanic under this folder, named `<mechanic>.md`:

```
fix-broken-protocols-and-hazards/
  index.md            // this file — inventory + status

  // Protocols
  burnout.md
  debt-collector.md
  conductor.md
  fission.md
  tier-regression.md
  siphon.md
  momentum.md

  // Hazards
  echo-cells.md
  fracture.md
  erosion.md
  range-adjacency.md  // cross-cutting: range-based adjacency via quadtree + Cascade/Diffusion/Fracture migration
  decay.md
```

Mechanics NOT listed here either (a) have only RON-tuning fixes (consolidated in `audit/remediations/ron-tuning-values.md`), (b) have only design-doc updates, (c) are already fully addressed by preceding TODOs, or (d) were re-scoped to nothing after evaluation against the landed TODOs (e.g., Volatility's mid-node-activation concern is invalid because hazards don't activate mid-node).

Each per-mechanic file specifies:
- What's broken (concrete pointer to the current buggy behavior)
- What the correct behavior should be
- Concrete code changes (files, functions, signatures)
- RON changes
- Tests to add or rewrite
- Design-doc updates that ride along with the code change

## Inventory

Status markers:
- **TODO** — not yet spec'd
- **SPEC'D** — per-mechanic detail file written
- **DONE** — landed

### Protocols

| Mechanic | Status | Summary of what needs fixing |
|----------|--------|------------------------------|
| Burnout | SPEC'D | Speed boost via `EffectStack<SpeedBoostConfig>`; Pattern B mega-bump test via `DamageBoostStack::add_one_shot` |
| Debt Collector | SPEC'D | Narrow `pub` → `pub(crate)` on `DebtStack` and `DebtCashOut` |
| Conductor | SPEC'D | Delete `PunchScale` VFX insertion; introduce `SwapBoltRoles` message with bolt-domain consumer |
| Fission | SPEC'D | `.replicate_of(primary)` builder method; `.primary().rendered()` spawn; per-node counter reset |
| Tier Regression | SPEC'D | Delete config/pending/snapshot; payload-free `RegressTier` message; state/run domain owns tier decrement |
| Siphon | SPEC'D | Escalating reward `time_per_kill * (N - 1)` |
| Momentum | SPEC'D | `Cell::builder().at_position().hp().spawn()` migration |

### Hazards

| Mechanic | Status | Summary of what needs fixing |
|----------|--------|------------------------------|
| Echo Cells | SPEC'D | `HpScaling` enum + `.ghost()` builder transition |
| Fracture | SPEC'D | `.debris()` builder + skip-occupied + recursion guard + Cascade ordering |
| Erosion | SPEC'D | SizeBoost aggregate applies to X only |
| Range-based adjacency | SPEC'D | Quadtree `query_circle` adjacency via `rantzsoft_physics2d`; Cascade/Diffusion/Fracture migrate; works for boss cells off-grid |
| Decay | SPEC'D | `Option<Res<ActiveHazards>>` harness-safety + paused test |

### Cross-cutting

| File | Status | Summary of what needs fixing |
|------|--------|------------------------------|
| `attach-system-migration.md` | SPEC'D | Migrate `attach_stack` / `renewal_attach_timers` / `attach_volatility_timers` from every-tick `Without<X>` to `Added<T>`. Rename to canonical `attach_<component_snake_case>` — `attach_debt_stack`, `attach_renewal_timer`, `attach_volatility_timer`. DebtCollector gets a companion `attach_debt_stack_on_activate` for mid-run chip selection. |

## Out of scope

- **RON-only tuning fixes** — consolidated in `audit/remediations/ron-tuning-values.md` (Burnout shockwave, Cascade heal values, Debt Collector tuning, Echo Strike tuning, Fission divergence angle, Iron Curtain falloff, Overcharge, Haste, Echo Cells `HpScaling` RON-shape change rides with the per-mechanic file).
- **Design-doc alignment files**: `iron-curtain-design-doc-abs-symmetric.md`, `reckless-dash-design-doc-bump-performed.md`, `renewal-design-doc-alignment.md`, `sympathy-design-doc-alignment.md`, `gravity-surge-design-doc-multiplicative-formula.md`, `erosion-design-doc-effect-stack-pattern.md`, `fracture-design-doc-pins.md`, `resonance-design-doc-alignment.md`, `tier-regression-design-doc-once-per-run.md`, `tier-regression-floor-and-tests.md` (doc portion), `diffusion-flat-share-per-ring.md` (doc + test; mechanic already correct). Handled in a separate doc-alignment sweep.
- **File splits**: `burnout-file-split.md`, `resonance-file-split.md` — structural only; user directed to ignore.
- **Cross-cutting infra**: `effect-system-end-to-end-integration-tests.md`, `effect-system-time-expires-wiring.md`, `placeholder-vfx-for-deferred-items.md`, `retire-sfx-from-all-protocols-and-hazards.md`, `ron-asset-tests-not-value-pinned.md`, `architecture-doc-accepted-patterns.md`, `attach-every-tick-is-canonical.md`, `deadline-integration-tests.md` — handled separately.
- **Mechanics fully addressed by preceding TODOs**: Anchor (#7), Deadline (#7), Kickstart (#7), Ricochet (#7), Afterimage (#5/#6), Greed (#3), Drift (#8), Gravity Surge (#8), bolt-loss (#4).
- **Re-scoped to nothing**: Volatility (mid-node activation assumption invalid — no code fix needed).

## Dependencies

Blocks on: #1, #2, #7, #8.

Once those land, this TODO can be broken into parallel waves — most per-mechanic fixes are independent.
