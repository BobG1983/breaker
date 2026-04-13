# Effect v3 Audit Resolution

## Summary
Close all findings from the `effect_v3/` audit (3 P0, 17 P1, 7 P2, 11 P3) plus ~150 missing tests, then merge feature/effect-system-refactor to develop.

## Context

A full manual audit of `breaker-game/src/effect_v3/` against the canonical design docs in `docs/todos/detail/effect-refactor/` was completed 2026-04-13 and catalogued at `.claude/state/audit/` (README, P0-critical, P1-behavior-gaps, P2-divergences, P3-cosmetic, test-coverage, matched, SUMMARY). It produced 35 findings plus an estimated 150–200 missing tests.

Why this matters:
- **Ship-blocking.** Two P0 findings (Shield, SecondWind) describe effects that spawn marker-only entities with no collision body — they do nothing at runtime. One P0 (asset pipeline) leaves the chip catalog empty at runtime because RON files are in pre-refactor syntax.
- **Attribution loss.** Multiple P1 findings show damage-dealing systems dropping `EffectSourceChip`, breaking chip-kill credit, flux rewards, and combat-log UI.
- **Dead code and stubs.** Walker has two no-op stubs (`walking/during.rs`, `walking/route.rs`) that enable silent feature drops for nested `During`/`When`/`On`.
- **Test coverage gaps.** 9 effects have zero tests; 5 more have partial coverage; the `time` trigger category has no tests; 9 of 10 bump bridges are untested.

The full execution plan lives at `~/.claude/plans/noble-pondering-lerdorf.md` (approved 2026-04-13). This file captures the essentials so the project todo is self-contained if the user-level plan file is lost.

## Scope

**In:**
- All 35 audit findings (P0 + P1 + P2 + P3)
- ~150–200 missing tests per `audit/test-coverage.md`
- Doc sweep to align `docs/todos/detail/effect-refactor/` with implemented behavior
- Chip asset migration from `assets/v3/chips/` → `assets/chips/`

**Out (explicitly):**
- `docs/todos/detail/unified-death-pipeline/` findings — separate audit target
- Downstream consumers beyond what effect_v3 touches

## Design Decisions (Resolved)

All 5 pre-work decisions are resolved and bake into the waves:

1. **Shield/SecondWind wall construction** — Lift `#[cfg(test)]` gate on `WallBuilder::floor()` and the `Floor` struct. Both effects spawn via `Wall::builder().floor(&playfield).spawn(commands)`. Floor promotes to production API.

2. **ChainLightning / Shockwave damage resolution** — Keep snapshot-at-fire semantics. Damage computed once from `BoltBaseDamage * damage_mult * DamageBoost.aggregate()` at fire time; subsequent mid-flight buff/debuff changes do NOT affect later hits from the same emission. Doc language revised to match.

3. **Nested conditions (P1-9 / P1-15)** — Support all 4 nested shapes recursively using `BoundEffects` mutations. **No new component needed** — the existing `walk_effects` / `evaluate_tree` pipeline already handles every `Tree` variant generically, so any runtime mutation of `BoundEffects` is automatically picked up by every trigger bridge.
   - **Shape A — `When(X, During(Cond, inner))`** (install-on-trigger, idempotent): First fire of `X` installs a new `Tree::During` entry into `BoundEffects` with a scope-path-suffixed source key (e.g., `"chip_siege#installed[0]"`). Subsequent fires are no-ops. Installed During is polled by `evaluate_conditions` as a normal top-level During.
   - **Shape B — `Until(X, During(Cond, inner))`** (interval-bound): During installed into `BoundEffects` on initial Until walk with a unique scope-path key. Polls normally while Until is alive. When `X` fires: remove the installed entry + call `reverse_all_by_source_dispatch` to unwind currently-applied inner effects.
   - **Shape C — `During(Cond, When(Trigger, Fire(reversible)))`** (armed scoped trigger): When Cond enters true, install `(source#armed[path], Tree::When(Trigger, Tree::Fire(reversible_as_EffectType)))` into `BoundEffects`. Every trigger bridge picks it up automatically — no bridge modifications needed. When Cond exits true, remove the entry + call `reverse_all_by_source_dispatch` to unwind every push made with the armed source tag.
   - **Shape D — `During(Cond, On(Participant, Fire(reversible)))`** — same as C with `Tree::On(...)` as the installed entry.

   **Runtime additions** (smaller than initially drafted):
   - Recursive `evaluate_conditions` (collects `Tree::During` at any depth, keys `DuringActive` by scope-path-suffixed source strings)
   - Implemented `walking/during.rs::evaluate_during` (Shape A install-into-BoundEffects, idempotent)
   - Implemented `walking/until.rs` nested-During handling (Shape B install + teardown)
   - **New `Reversible::reverse_all_by_source(entity, source, world)` trait method** with a default implementation that delegates to `reverse`. Contract: "unwinds every state change made under `source`, regardless of config."
   - **Overrides needed (10 of 16):**
     - 8 passive configs: one-liner `retain` on `EffectStack<Self>`
     - **Attraction**: `retain` on `ActiveAttractions`
     - **Anchor**: remove singleton markers + `retain` on `EffectStack<PiercingConfig>` — **fixes P2-5**
   - **No override needed (6 of 16)** — use default delegation:
     - FlashStep, Pulse, Shield, SecondWind, CircuitBreaker, EntropyEngine — all singleton state (marker or component present/absent). Fire is idempotent; a single reverse call restores the "absent" state regardless of how many armed fires preceded it.
   - **New dispatch helper** `reverse_all_by_source_dispatch` in `dispatch.rs`.
   - `EffectStack` itself stays unchanged.
   - **Authoring note**: Shape C/D with singleton-state non-passive inners (e.g., `During(Cond, When(X, Fire(Pulse)))`) is accepted but semantically unusual — first armed fire installs the marker, subsequent fires are no-ops. Passive reversibles give clearer stacking semantics.

4. **`NodeActive` doc language (P1-10)** — Delete "or paused" from `docs/todos/detail/effect-refactor/rust-types/enums/condition.md`. No `Paused` sub-state will be added.

5. **`NodeEndOccurred` timing (P1-16)** — Move from `OnExit(NodeState::Playing)` to `OnEnter(NodeState::Teardown)`. Ensures cells are gone before handlers run.

## Reusable Utilities — MUST USE

- **Wall spawning**: `Wall::builder()` at `breaker-game/src/walls/builder/core/transitions.rs:13`. Typestate builder handles Wall + Spatial + Position2D + Scale2D + Aabb2D + CollisionLayers + GameDrawLayer + effect stamping. Shield/SecondWind must route through this.
- **Test apps**: `TestAppBuilder` at `breaker-game/src/shared/test_utils/builder.rs`. Chain `.with_state_hierarchy()` + `.in_state_node_playing()` for stateful tests. Use `MessageCollector` + `tick()` helpers from same module. No hand-rolled `App::new()`.

## Waves

Execution sequencing (see plan file for full critical-files breakdown per wave):

```
Wave 0 (chip asset move — BLOCKING)                      quickfix
  ↓
Wave 1 ∥ Wave 2 ∥ Wave 3 ∥ Wave 5 ∥ Wave 6             parallel
         Wave 9 ∥ Wave 10 ∥ Wave 11
  ↓
Wave 4 (pulse — needs Wave 3 pattern)
  ↓
Wave 7 (nested condition shapes A–D — largest; consider sub-wave split)
  ↓
Wave 8 (test backfill — parallelized by module, 4–6 at a time)
  ↓
Full Verification Tier → writer-scenarios → /finish-dev → merge
```

- **Wave 0** — Move `assets/v3/chips/*` → `assets/chips/*`, delete `assets/v3/`, add chip-catalog load smoke test.
- **Wave 1** — Shield + SecondWind spawn via wall builder (Floor side); lift cfg-gate prerequisite; create `second_wind/systems.rs` (missing today).
- **Wave 2** — (a) Move NoBump emission to after retroactive late-bump window closes; (b) Move NodeEndOccurred bridge to `OnEnter(Teardown)`.
- **Wave 3** — Shockwave + TetherBeam source_chip attribution fixes; attach `EffectSourceChip` at tether spawn.
- **Wave 4** — Pulse structural separation: own `PulseRing*` components, own tick/damage systems, reads BoltBaseDamage + DamageBoost from emitter, carries EffectSourceChip.
- **Wave 5** — CircuitBreaker fires both Shockwave AND SpawnBolts on counter zero.
- **Wave 6** — TetherBeam `width` field added to config + doc + finalized-asset RON.
- **Wave 7** — Nested conditions (all 4 shapes) via `BoundEffects` mutations. Recursive `evaluate_conditions`, implemented `walking/during.rs` + `walking/until.rs`, new `Reversible::reverse_all_by_source` trait method with default delegation to `reverse`, overrides for 10 of 16 variants (8 passive one-liners + Attraction retain + Anchor singleton+piercing retain). The 6 remaining non-passives (FlashStep, Pulse, Shield, SecondWind, CircuitBreaker, EntropyEngine) use the default. New `reverse_all_by_source_dispatch` helper. Delete `walking/route.rs` dead code. No new component, no trigger bridge modifications, `EffectStack` unchanged. Absorbs P2-5 anchor magic-constant fix as a side effect.
- **Wave 8** — Test backfill (~150–200 tests), prioritized by risk. All tests use `TestAppBuilder`.
- **Wave 9** — Docs sweep: path drift fixes, snapshot-damage language revision, NodeActive cleanup, NodeEnd timing update, 4 nested shape docs, new `ScopePath` + `ArmedScopedTriggers` type docs.
- **Wave 10** — Call `register()` on all 30 effect configs (not just 12). P2-5 anchor magic constant fix is absorbed into Wave 7 via the new `reverse_all_by_source` contract.
- **Wave 11** — Grep suppression attributes (`#[allow]`, `#[expect]`) in effect_v3/ and clean up.

## Dependencies

- Depends on: item #1 **Effect system refactor** (the parent work that produced `effect_v3/`). That refactor has largely landed on `feature/effect-system-refactor`; this item closes its audit gaps before merge.
- Blocks: `feature/effect-system-refactor` merge to develop (Full Verification Tier must pass).

## Notes

- Audit files: `.claude/state/audit/{README,P0-critical,P1-behavior-gaps,P2-divergences,P3-cosmetic,test-coverage,matched,SUMMARY}.md`
- Approved plan file: `~/.claude/plans/noble-pondering-lerdorf.md`
- Current branch: `feature/effect-system-refactor`
- P1-14 `ComboStreak` resource doc remains unresolved — recommend deletion and reference `HighlightTracker.consecutive_perfect_bumps`, decide during Wave 9.
- Wave 7 is the largest individual wave; may warrant splitting into 7a (Reversible trait contract + 10 overrides + dispatch helper), 7b (recursive `evaluate_conditions` with scope-path keying + Shape C/D arm/disarm), 7c (walker Shape A install + Shape B install/teardown + dead code deletion) during the planning-writer-specs phase.
- **Open question from spec phase — closed**: 8 non-passive `ReversibleEffectType` variants enumerated; `reverse_all_by_source` default-delegates to `reverse` for 6 of them (singleton state) and overrides for Attraction + Anchor. P2-5 anchor fix falls out naturally.

## Status
`ready`
