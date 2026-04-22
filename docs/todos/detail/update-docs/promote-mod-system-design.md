# Promote `mod-system-design/` to canonical design docs

Wholesale migration: `docs/todos/detail/mod-system-design/{protocols,hazards}/*.md` → `docs/design/{protocols,hazards}/*.md`, cleansed to describe the **target** design (post-TODOs #1–#8), not today's buggy impl. After promotion, `mod-system-design/` is deleted.

## Why this belongs in TODO #0

Single source of truth. Without this step, two locations describe each protocol/hazard:

- `docs/todos/detail/mod-system-design/...` — pre-TODO baseline (stale: references `EffectStack<DamageBoostConfig>`, `BurnoutSpeedBoost`, `ProtocolPending*` snapshots, every-tick `Without<X>` attach, etc.)
- `docs/todos/detail/fix-broken-protocols-and-hazards/...` — delta against that baseline

Subagents implementing TODO #9 would hit contradictions. Promoting + cleansing first gives the fix-broken delta a canonical baseline to reference.

Pure doc work, no code, no RON, no tests — belongs in TODO #0 alongside the other alignment files.

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

Create parent directories if they don't exist. Add `docs/design/protocols/index.md` and `docs/design/hazards/index.md` listing the full set with one-line summaries.

## Cleanse rules

Every promoted file describes the **post-TODO target design**. Strike or rewrite these patterns when present:

| Stale pattern | Replace with |
|---------------|--------------|
| `EffectStack<DamageBoostConfig>` | `DamageBoostStack { persistent, one_shots }` (from `rantzsoft_dmg` crate, TODO #1) |
| `EffectStack<VulnerableConfig>` | `VulnerableStack` (from `rantzsoft_dmg` crate, TODO #1) |
| Direct `DamageDealt<Cell>` emission for amplified damage | `stack.add_one_shot(multiplier)` (Pattern B via `DamageBoostStack::one_shots`) |
| Bespoke "track-and-amplify" components (`BurnoutSpeedBoost`, etc.) | `EffectStack<SpeedBoostConfig>` via `commands.fire_effect` (speed) / `DamageBoostStack` (damage) |
| Every-tick `Query<Entity, (With<T>, Without<X>)>` attach pattern | `Query<Entity, Added<T>>` attach pattern (canonical) |
| `ProtocolPending*` snapshots | Payload-free messages (e.g., `RegressTier`) — state-owning domain holds the data |
| `hazard/` or `protocol/` domain paths | `mutators/{protocols,hazards}/<name>/` (TODO #2) |
| `HpScaling` as ad-hoc multiplier math | `HpScaling` enum variants: `Geometric { multiplier }`, `Linear { step }`, `Doubles` (TODO #9 echo-cells fix) |
| Per-mechanic tree-declared effect RON blocks (Anchor/Deadline/Kickstart/Ricochet) | Code-driven via `commands.fire_effect` / `commands.reverse_effect` (TODO #7) |
| Direct `Velocity2D` mutation from Drift / Gravity Surge | `ApplyBoltForce { bolt, force }` messages (TODO #8) |
| Effect-tree `bolt_lost` field on `BreakerDefinition` | `BoltLossBehavior::{ LifeLoss, TimeLoss, None }` component (TODO #4) |
| `stamp_root` skipped / conditional dispatch | `commands.stamp_root(&RootNode, source)` as sole dispatch facade (TODO #7) |
| Separate phantom-bolt spawn path for afterimage | Mutate the real bolt via `Bolt::become_phantom` + `LifetimeEndBehavior::RevertToNormalBolt` (TODO #6) |
| Prism breaker references | Removed (TODO #4 retires it) |
| Spawn-and-forget phantom breaker without `Lifespan`/`PhantomFlicker` | `Breaker::builder().phantom(BreakerPhantomParams { lifespan, .. })` with shared infra (TODO #5) |

Keep:
- Game Design narrative (feel, intent, player-facing behavior)
- Config struct shape and field meanings (update field names if the fix spec renames them — e.g., Overcharge)
- Component list (with the above substitutions applied)
- Message reads/sends (with the above substitutions applied)
- System list with schedules, run-if gates, ordering edges
- Expected Behaviors (test-spec-ready given/when/then)
- Edge Cases
- Cross-Domain Dependencies

Remove:
- "Current impl" notes or "as of today" hedges — the doc describes the target, period
- Phase-2-punt language, conditional language ("if X, then Y; if not, then Z")

## File-format conventions

Keep the existing section order from `mod-system-design`:

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

## Alignment with the existing per-mechanic fix files

For each promoted file, check the corresponding `fix-broken-protocols-and-hazards/<name>.md` if one exists. The fix file's "Fix" section describes what the target design looks like — the promoted doc must match. If they disagree, the fix file wins (it's newer and TODO-aligned).

Mechanics with existing fix-broken files:
- Protocols: burnout, debt-collector, conductor, fission, tier-regression, siphon, momentum
- Hazards: echo-cells, fracture, erosion, decay (+ grid, which is cross-cutting and not a single mechanic)
- Cross-cutting: attach-system-migration covers debt-collector, renewal, volatility attach systems

Mechanics with NO fix-broken file (design carries forward unchanged except for cross-cutting pattern substitutions above):
- Protocols: afterimage, anchor, deadline, echo_strike, greed, iron_curtain, kickstart, reckless_dash, ricochet
- Hazards: cascade, diffusion, drift, gravity_surge, haste, overcharge, renewal, resonance, sympathy, tether, volatility

## What else needs doing alongside the promotion

1. **Delete** `docs/todos/detail/mod-system-design/` after all 31 files are promoted and verified. Also delete:
   - `docs/todos/detail/mod-system-design/mod-system-design.md` (the old overarching system-design doc — its content belongs in `docs/design/protocols/index.md` + `docs/design/hazards/index.md` where it lands naturally)
   - `docs/todos/detail/mod-system-design/research/` (obsolete research notes)
   - `docs/todos/detail/mod-system-design/legendary-retuning.md`
   - `docs/todos/detail/mod-system-design/stubbing-tiers.md`

   If any of those non-per-mechanic files contain decisions not yet captured elsewhere, spot-promote them to the appropriate `docs/design/` subtree first — don't just delete.

2. **Update** `fix-broken-protocols-and-hazards/index.md` and every per-mechanic fix file so "See the canonical design at `docs/design/{protocols,hazards}/<name>.md`" replaces any reference to `mod-system-design/`.

3. **Grep** for any other references to `mod-system-design` across the repo and update. Session-state notes it was "KEPT as source material" — update that note once deletion is done.

## Scope boundary

In scope:
- Moving + cleansing 31 per-mechanic files into `docs/design/{protocols,hazards}/`.
- Creating `docs/design/{protocols,hazards}/index.md`.
- Deleting `docs/todos/detail/mod-system-design/` and updating cross-references.

Out of scope:
- Any change to `fix-broken-protocols-and-hazards/*.md` bodies beyond the "canonical design lives at X" pointer.
- Any code, RON, or test change.
- Redesigning any mechanic — cleansing is pattern substitution, not re-design.

## Dependencies

None. Doc-only. Can land in parallel with the other update-docs files (itself a parallel sweep).

## Reference

The existing per-mechanic fix files in `docs/todos/detail/fix-broken-protocols-and-hazards/` are the authoritative source for what the target design looks like. When in doubt, align the promoted doc with the fix-broken file's "Fix" section.
