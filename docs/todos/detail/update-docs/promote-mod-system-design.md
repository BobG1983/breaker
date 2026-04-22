# Promote `mod-system-design/` to canonical design docs

Wholesale migration: `docs/todos/detail/mod-system-design/{protocols,hazards}/*.md` → `docs/design/{protocols,hazards}/*.md`, cleansed to describe the post-TODO target design (TODOs #1-#9 assumed complete). After promotion, `mod-system-design/` is deleted.

## Source files (31 total)

### Protocols (15)
`docs/todos/detail/mod-system-design/protocols/`:
- `afterimage.md`
- `anchor.md`
- `burnout.md`
- `conductor.md`
- `deadline.md`
- `debt_collector.md`
- `echo_strike.md`
- `fission.md`
- `greed.md`
- `iron_curtain.md`
- `kickstart.md`
- `reckless_dash.md`
- `ricochet_protocol.md` (rename to `ricochet.md` on promotion — drop `_protocol` suffix)
- `siphon.md`
- `tier_regression.md`

### Hazards (16)
`docs/todos/detail/mod-system-design/hazards/`:
- `cascade.md`
- `decay.md`
- `diffusion.md`
- `drift.md`
- `echo_cells.md`
- `erosion.md`
- `fracture.md`
- `gravity_surge.md`
- `haste.md`
- `momentum.md`
- `overcharge.md`
- `renewal.md`
- `resonance.md`
- `sympathy.md`
- `tether.md`
- `volatility.md`

## Destination

- `docs/design/protocols/<name>.md` (snake_case)
- `docs/design/hazards/<name>.md` (snake_case)

Create parent directories if they don't exist.

Also write:
- `docs/design/protocols/index.md` — modelled on `docs/design/effects/index.md`; protocol-system philosophy, chip/protocol distinction, 15-protocol table by category, meta-progression note. Source: `mod-system-design.md` § Protocols + § Decisions.
- `docs/design/hazards/index.md` — modelled on `docs/design/effects/index.md`; hazard-system philosophy, Hard Rules list, 16-hazard pool with tuning, trap synergies, killed proposals, flat-pool + choose-your-poison rationale. Source: `mod-system-design.md` § Hazards + § Trap Synergies + § Killed Hazard Proposals + § Why sections.
- `docs/design/index.md` — update the § Sub-Documents table to add rows for `protocols/` and `hazards/`.

## Cleanse rules

Every promoted file describes the **post-TODO target design**. Apply these substitutions whenever the source content uses a stale pattern:

| Stale pattern (source) | Target pattern (post-TODO) | Source TODO |
|------------------------|----------------------------|-------------|
| `EffectStack<DamageBoostConfig>` | `DamageBoostStack { persistent, one_shots }` from `rantzsoft_dmg` | TODO #1 |
| `EffectStack<VulnerableConfig>` | `VulnerableStack` from `rantzsoft_dmg` | TODO #1 |
| Direct `DamageDealt<Cell>` emission for amplified damage | `stack.add_one_shot(multiplier)` (Pattern B via `DamageBoostStack::one_shots`) | TODO #1 |
| Bespoke "track-and-amplify" components (`BurnoutSpeedBoost`, `RiskyDamageBoost`, etc.) | `EffectStack<SpeedBoostConfig>` via `commands.fire_effect` (speed) / `DamageBoostStack` (damage) | TODO #1 |
| Every-tick `Query<Entity, (With<T>, Without<X>)>` attach pattern | `Query<Entity, Added<T>>` attach pattern, renamed `attach_<component_snake_case>` | TODO #9 |
| `ProtocolPending*` snapshot resources | Payload-free messages (e.g., `RegressTier`) — state-owning domain holds the data | TODO #9 |
| `hazard/` or `protocol/` domain paths | `mutators/{protocols,hazards}/<name>/` | TODO #2 |
| `HpScaling` as ad-hoc multiplier math | `HpScaling` enum variants: `Geometric { multiplier }`, `Linear { step }`, `Doubles` | TODO #9 |
| Per-mechanic tree-declared effect RON blocks for Anchor/Deadline/Kickstart/Ricochet | Code-driven via `commands.fire_effect` / `commands.reverse_effect` | TODO #7 |
| Direct `Velocity2D` mutation from Drift / Gravity Surge | `ApplyBoltForce { bolt, force }` message pipeline | TODO #8 |
| Effect-tree `bolt_lost` field on `BreakerDefinition` | `BoltLossBehavior::{ LifeLoss, TimeLoss, None }` component | TODO #4 |
| `stamp_root` skipped / conditional dispatch | `commands.stamp_root(&RootNode, source)` as sole dispatch facade | TODO #7 |
| Separate phantom-bolt spawn path for afterimage | Mutate the real bolt via `Bolt::become_phantom` + `LifetimeEndBehavior::RevertToNormalBolt` | TODO #6 |
| Prism breaker references | Removed entirely | TODO #4 |
| Spawn-and-forget phantom breaker without shared infra | `Breaker::builder().phantom(BreakerPhantomParams { lifespan, .. })` with shared `Lifespan` + `PhantomFlicker` | TODO #5 |
| `EffectStack<DamageBoostConfig>` on Conductor's `PunchScale` VFX insertion | Deleted entirely; `SwapBoltRoles` message handles role swap | TODO #9 (Conductor) |
| DebtCollector `DebtStack` / `DebtCashOut` visibility `pub` | `pub(crate)` | TODO #9 (Debt Collector) |
| Fission per-node counter held globally | Per-node counter reset via `.replicate_of(primary)` builder | TODO #9 (Fission) |
| Momentum spawn via raw entity construction | `Cell::builder().at_position().hp().spawn()` | TODO #9 (Momentum) |
| Siphon flat reward | Escalating reward `time_per_kill * (N - 1)` | TODO #9 (Siphon) |
| Echo Cells HP scaling via ad-hoc multiplier | `HpScaling` enum + `.ghost()` builder | TODO #9 (Echo Cells) |
| Fracture debris via raw spawn | `.debris()` builder + skip-occupied + recursion guard | TODO #9 (Fracture) |
| Cascade / Diffusion ad-hoc neighbour queries | `CellGridPosition` + `orthogonal_neighbors` helper | TODO #9 (Grid) |

Keep:
- Game Design narrative (feel, intent, player-facing behaviour).
- Config struct shape and field meanings (apply renames from the per-mechanic target-state specs where applicable).
- Component list (with substitutions above applied).
- Message reads/sends (with substitutions above applied).
- System list with schedules, run-if gates, ordering edges.
- Expected Behaviors (test-spec-ready given/when/then).
- Edge Cases.
- Cross-Domain Dependencies.

Remove:
- "Current impl" notes or "as of today" hedges — the doc describes the target, period.
- Conditional language ("if X, then Y; if not, then Z").
- Implementation-history sections (delivered waves, remaining waves) — those belong in git history, not design docs.

## File-format conventions

Section order for each per-mechanic file:

```
# {Kind}: {Name}

## Category
## Game Design
## Config Resource
## Components
## Messages
## Systems
### <system_name>
- **Schedule**:
- **run_if**:
- **Behavior**:
- **Ordering**:
## Cross-Domain Dependencies
## Expected Behaviors (for test specs)
## Edge Cases
```

Kind is `Protocol` or `Hazard`. Name uses title case with spaces (`Reckless Dash`, `Echo Cells`, `Gravity Surge`).

## Authority of per-mechanic target-state specs

For every mechanic with a corresponding file in this folder (see `index.md` table), that file is **authoritative** for any content it specifies. The promoter reads both the mod-system-design source (for structural content) and the target-state spec (for specific text), and produces the canonical doc. When they conflict, the target-state spec wins — the mod-system-design content is pre-TODO baseline, the target-state spec is post-TODO.

Mechanics with target-state specs:
- Protocols: iron_curtain, reckless_dash, tier_regression
- Hazards: renewal, sympathy, gravity_surge, erosion, fracture, resonance, diffusion, haste, overcharge

Mechanics WITHOUT target-state specs — straight cleansing from the source:
- Protocols: afterimage, anchor, burnout, conductor, deadline, debt_collector, echo_strike, fission, greed, kickstart, ricochet, siphon
- Hazards: cascade, decay, drift, echo_cells, momentum, tether, volatility

## Alongside the promotion

1. **Delete** `docs/todos/detail/mod-system-design/` in its entirety, including:
   - The 31 per-mechanic source files (their content now lives at `docs/design/{protocols,hazards}/`).
   - `mod-system-design.md` (game-design content extracted into the two new index files; implementation-history content dropped).
   - `research/` (process-stage brainstorm, superseded by per-mechanic files + code).
   - `legendary-retuning.md` (historical — legendary rarity removal is already complete).
   - `stubbing-tiers.md` (the `NodeOutcome.tier` surface is already in code + tested; the doc is informational about a completed decision).

2. **Update cross-references** across the repo. Grep for `mod-system-design` and apply these path rewrites:

   | From | To |
   |------|----|
   | `docs/todos/detail/mod-system-design/protocols/<name>.md` | `docs/design/protocols/<name>.md` |
   | `docs/todos/detail/mod-system-design/hazards/<name>.md` | `docs/design/hazards/<name>.md` |
   | `docs/todos/detail/mod-system-design/ricochet_protocol.md` | `docs/design/protocols/ricochet.md` |
   | `docs/todos/detail/mod-system-design/mod-system-design.md` | `docs/design/protocols/index.md` or `docs/design/hazards/index.md` (judgment per-ref) |
   | `docs/todos/detail/mod-system-design/research/*.md` | Removed — evaluate whether the referring text needs a replacement pointer, or just delete the reference |
   | `docs/todos/detail/mod-system-design/legendary-retuning.md` | Removed — reference drops |
   | `docs/todos/detail/mod-system-design/stubbing-tiers.md` | Removed — reference drops |

   Files to sweep include:
   - `docs/architecture/plugins.md`
   - `docs/todos/TODO.md`
   - `docs/todos/detail/fix-broken-protocols-and-hazards/*.md` (several — replace "baseline lives at mod-system-design" with "canonical design lives at `docs/design/{protocols,hazards}/<name>.md`")
   - `docs/todos/detail/apply-bolt-force-pipeline.md`, `retire-sfx.md`, `stamp-dispatcher-unification.md`
   - `docs/todos/detail/node-sequencing-refactor/*`
   - `breaker-game/src/**/*.rs` (doc-comment references)
   - `.claude/state/session-state.md`

3. **Update** `.claude/state/session-state.md` — the existing note about `mod-system-design/` being "KEPT as source material" is obsolete after deletion; remove or rewrite.

## Scope boundary

**In scope**:
- Writing 31 canonical per-mechanic files at `docs/design/{protocols,hazards}/`.
- Writing 2 index files (`protocols/index.md`, `hazards/index.md`).
- Updating `docs/design/index.md` to list the new trees.
- Deleting `docs/todos/detail/mod-system-design/`.
- Updating cross-references.

**Out of scope**:
- Any code, RON, or test change.
- Redesigning any mechanic — cleansing is pattern substitution, not re-design.
- Adding new architectural patterns (e.g., cross-domain-mechanisms registry). Architecture doc additions belong to the TODO that owns each pattern.

## Dependencies

Assumes TODOs #1-#9 are complete. Doc-only otherwise.
