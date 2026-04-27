# Source-filtered damage boosts in `rantzsoft_dmg`

## Summary
Add an optional `SourceId` filter to every `DamageBoostStack` entry (persistent AND one-shot) so a boost can be scoped to amplify only damage emissions whose `source` matches.

## Context
Investigated as item #4 in a deferred-work review on 2026-04-27. Today's `apply_damage_boosts` in `rantzsoft_dmg` walks every entry on the dealer's `DamageBoostStack` and aggregates them all into the emission, regardless of which mechanic produced the entry vs which mechanic produced the emission.

The current game-side game uses dedicated marker components (`BurnoutDamageBoost`, `RiskyDamageBoost`, `DebtCashOut`) for source-scoped one-shots specifically to avoid touching the unfiltered stack. That works but reinvents the same machinery per mechanic. Cross-contamination between the marker-based approaches isn't possible today, but the moment a future caller starts using the crate's `add_one_shot()` for a scoped boost, every other concurrent emission from that bolt will silently consume or apply it.

The unified design: every stack entry carries an optional `SourceId` filter; persistent and one-shot entries apply the same filter rule. Mechanics get one place to compose source-scoped boosts (whether permanent or one-shot) without rolling marker components.

## Design

### The filter

Every `DamageBoostStack` entry gains a field — name TBD, candidates: `applies_to: Option<SourceId>`, `source_filter: Option<SourceId>`, `scope: Option<SourceId>`. **NOT** named `source` since the existing entry already has a `source: SourceId` (attribution of who placed the boost — for debug / dedup).

### Apply rule

For each entry, when `apply_damage_boosts` is processing a `DamageDealt<T>`:

| Entry filter | Emission source | Apply? | Consume on apply? |
|---|---|---|---|
| `None` | anything | yes | yes (one-shot) / no (persistent) |
| `Some(f)` | `Some(s)` and `f` matches `s` | yes | yes (one-shot) / no (persistent) |
| `Some(f)` | `Some(s)` and `f` does NOT match `s` | no | **no** (preserve) |
| `Some(f)` | `None` | no | no (preserve) |

Key: a non-matching emission never consumes a one-shot. The one-shot waits for the right emission. This is what callers actually want — "amplify the next *burnout* damage event", not "amplify the next event of any kind, but only if that event happens to be burnout".

### Match predicate

Start with structural equality after instance-stripping — `protocol:burnout` matches both `protocol:burnout` and `protocol:burnout:instance:N`. Prefix-match-by-namespace can come later if a real caller needs it. Don't over-engineer the predicate before the first user.

### Backward compatibility

`filter: None` is the default — the legacy `add()` and `add_one_shot()` API stays as-is and produces filterless entries. Existing callers don't break. New API: `add_filtered(amount, filter: SourceId)` and `add_one_shot_filtered(amount, filter: SourceId)`, OR a builder pattern.

### Game-side migration (optional, separate)

Once the crate supports filtering, game-side mechanics that currently roll marker components (`BurnoutDamageBoost`, `RiskyDamageBoost`, `DebtCashOut`) could migrate to crate-owned filtered stack entries. That's a separate per-mechanic refactor — not in scope for the crate change. List those mechanics as candidates so future cleanup work can find them.

## Scope
- **In:** `rantzsoft_dmg` crate change. Add filter field to stack entry, extend the API with filtered variants, update `apply_damage_boosts` to honor the filter, ensure non-matching emissions don't consume one-shots. Tests covering the four rows of the table above.
- **Out:** Migrating game-side `BurnoutDamageBoost` / `RiskyDamageBoost` / `DebtCashOut` markers to filtered stack entries. (Separate todo per mechanic if desired.) Out: prefix matching predicate. Out: vulnerability stack — same gap likely exists there, but solve once first.

## Dependencies
- **Depends on:** none currently — `apply_damage_boosts` is in `DmgSystems::ApplyDamageBoosts` which is upstream of the rest of the chain.
- **Blocks:** future mechanics that want source-scoped persistent boosts (no current blocker).

## Notes
- The existing field on stack entries (probably named `source`) is the *attributor* — who placed this boost. The new filter is the *applies-to source* — what kind of emission this boost amplifies. They're independent fields. The naming must make this distinction clear in the API.
- Consider auditing `VulnerableStack` for the same gap — it has the same structure (per-victim instead of per-dealer) and likely the same latent issue. Either fix both crates' stacks symmetrically in this todo, or split into a follow-up todo. Recommend: do `DamageBoostStack` first, then assess if `VulnerableStack` needs the same.

## Status
`ready` — design captured; the open implementation question is whether to also include `VulnerableStack` symmetric fix, which can be answered during implementation.
