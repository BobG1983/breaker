# Source-filtered damage boosts AND vulnerability in `rantzsoft_dmg`

## Summary
Add an optional `SourceId` filter to every entry in `DamageBoostStack` AND `VulnerableStack` (persistent AND one-shot), so a boost or vulnerability can be scoped to amplify only emissions whose `source` matches.

## Context
Investigated as item #4 in a deferred-work review on 2026-04-27. Today's `apply_damage_boosts` in `rantzsoft_dmg` walks every entry on the dealer's `DamageBoostStack` and aggregates them all into the emission, regardless of which mechanic produced the entry vs which mechanic produced the emission. `apply_vulnerable` has the symmetric gap on the victim side: every `VulnerableStack` entry applies to every incoming emission regardless of source.

The current game-side game uses dedicated marker components (`BurnoutDamageBoost`, `RiskyDamageBoost`, `DebtCashOut`) for source-scoped one-shot damage boosts specifically to avoid touching the unfiltered stack. That works but reinvents the same machinery per mechanic. Cross-contamination between the marker-based approaches isn't possible today, but the moment a future caller starts using the crate's `add_one_shot()` for a scoped boost, every other concurrent emission from that bolt will silently consume or apply it. The same hazard exists on the vulnerability side the moment a mechanic wants "this cell takes extra damage from *burnout* hits but normal damage from chain lightning".

The unified design: every `DamageBoostStack` and `VulnerableStack` entry carries an optional `SourceId` filter; persistent and one-shot entries apply the same filter rule symmetrically. Mechanics get one place to compose source-scoped boosts AND vulnerabilities (whether permanent or one-shot) without rolling marker components.

### Crate-owned, no `EffectStack<T>` involvement

`impl PassiveEffect for DamageBoostConfig` and `impl PassiveEffect for VulnerableConfig` are already deleted (W3). Neither config participates in `EffectStack<T>` aggregation. `DamageBoostStack` and `VulnerableStack` are the sole aggregators, owned by `rantzsoft_dmg`, and aggregated in-place via `aggregate_persistent()` / `aggregate_and_consume_one_shots()`. This todo extends those crate-owned stack types — no `EffectStack<DamageBoostConfig>` / `EffectStack<VulnerableConfig>` plumbing exists to touch.

The `DamageBoostConfig` / `VulnerableConfig` `Fireable`/`Reversible` shims forward to `stack.add()` / `stack.remove_by_source()` on the crate-owned components. Those shims need new entry points so callers can install filtered entries.

## Design

### The filter

Every `DamageBoostStack` AND `VulnerableStack` entry gains a field — name TBD, candidates: `applies_to: Option<SourceId>`, `source_filter: Option<SourceId>`, `scope: Option<SourceId>`. **NOT** named `source` since the existing entry already has a `source: SourceId` (attribution of who placed the boost/vulnerability — for debug / dedup).

The same field shape applies to both stacks. The aggregation logic is structurally identical between the two crates' apply systems.

### Apply rule

For each entry, when `apply_damage_boosts::<T>` is processing a `DamageDealt<T>` (boost side, dealer's stack) or `apply_vulnerable::<T>` is processing a `DamageDealt<T>` (vulnerability side, victim's stack):

| Entry filter | Emission source | Apply? | Consume on apply? |
|---|---|---|---|
| `None` | anything | yes | yes (one-shot) / no (persistent) |
| `Some(f)` | `Some(s)` and `f` matches `s` | yes | yes (one-shot) / no (persistent) |
| `Some(f)` | `Some(s)` and `f` does NOT match `s` | no | **no** (preserve) |
| `Some(f)` | `None` | no | no (preserve) |

Key: a non-matching emission never consumes a one-shot. The one-shot waits for the right emission. This is what callers actually want — "amplify the next *burnout* damage event", not "amplify the next event of any kind, but only if that event happens to be burnout". The same rule governs vulnerability: "this cell takes extra damage from *burnout* — but only burnout".

### Match predicate

Start with structural equality after instance-stripping — `protocol:burnout` matches both `protocol:burnout` and `protocol:burnout:instance:N`. Prefix-match-by-namespace can come later if a real caller needs it. Don't over-engineer the predicate before the first user. The predicate lives on `SourceId` (or a free function in the crate) so both stacks share it.

### Backward compatibility

`filter: None` is the default — the legacy `add()` and `add_one_shot()` API stays as-is on both stacks and produces filterless entries. Existing callers don't break. New API (mirrored on both stacks): `add_filtered(amount, filter: SourceId)` and `add_one_shot_filtered(amount, filter: SourceId)`, OR a builder pattern.

The `Fireable` impls for `DamageBoostConfig` and `VulnerableConfig` continue to call the unfiltered `add()` (preserves existing RON semantics). Filtered entries enter via direct stack manipulation from game-side mechanics that opt in.

### Game-side migration (separate, optional)

Once the crate supports filtering symmetrically, game-side mechanics can migrate:

- **Damage-boost markers** — `BurnoutDamageBoost`, `RiskyDamageBoost`, `DebtCashOut` are candidates to retire in favor of `DamageBoostStack::add_one_shot_filtered`.
- **Vulnerability markers** — audit for any analogous game-side markers; if none today, the filtered `VulnerableStack::add_filtered` is just available for future mechanics.

Migration work is per-mechanic, not in scope for the crate change. List candidates so future cleanup work can find them.

## Scope
- **In:** `rantzsoft_dmg` crate change. Add filter field to BOTH `DamageBoostStack` entries AND `VulnerableStack` entries. Extend BOTH stacks' APIs with filtered variants (persistent + one-shot). Update `apply_damage_boosts` AND `apply_vulnerable` to honor the filter symmetrically. Ensure non-matching emissions don't consume one-shots from either stack. Tests covering the four rows of the table above for BOTH stacks.
- **Out:** Migrating game-side `BurnoutDamageBoost` / `RiskyDamageBoost` / `DebtCashOut` markers to filtered stack entries (separate per-mechanic todos if desired). Out: prefix matching predicate. Out: any change to `EffectStack<T>` (those configs no longer aggregate via that path).

## Dependencies
- **Depends on:** none currently — `apply_damage_boosts` lives in `DmgSystems::ApplyDamageBoosts` and `apply_vulnerable` in `DmgSystems::ApplyVulnerable`, both upstream of the rest of the chain.
- **Blocks:** future mechanics that want source-scoped persistent or one-shot boosts/vulnerabilities (no current blocker).

## Notes
- The existing field on stack entries (probably named `source`) is the *attributor* — who placed this boost/vulnerability. The new filter is the *applies-to source* — what kind of emission this entry applies to. They're independent fields. The naming must make this distinction clear in both stacks' APIs.
- The match predicate should live in one place (likely a method on `SourceId` or a free function in the crate) so the two stacks can't drift in how they interpret a filter.
- No `EffectStack<DamageBoostConfig>` / `EffectStack<VulnerableConfig>` exists to update — those `PassiveEffect` impls are already deleted.

## Status
`ready` — design captured for both stacks. Both stacks land together to keep the API symmetric and avoid the half-migrated state.
