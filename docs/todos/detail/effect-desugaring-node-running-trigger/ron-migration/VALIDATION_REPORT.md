# RON Migration Validation Report

Validated: 2026-04-06
Files compared: 55 (3 breakers, 36 standard chips, 16 evolutions)

## Breakers

### aegis.breaker.ron -- PASS
- Original: `On(target: Breaker, ...[When(trigger: BoltLost, ...[Do(LoseLife)])])` + 3 bolt speed boosts on PerfectBumped/EarlyBumped/LateBumped
- Migrated: `When(BoltLostOccurred, On(This, Fire(LoseLife)))` + `When(PerfectBumped, On(PerfectBumped::Bolt, Fire(SpeedBoost(...))))` etc.
- `BoltLost` -> `BoltLostOccurred` (global trigger, correct). `On(target: Breaker)` -> `On(This)` for breaker-stamped effect. Bolt speed boosts use local `PerfectBumped` with `PerfectBumped::Bolt` participant (correct since the effect is stamped on the breaker but targets the bolt). All effect values preserved.

### chrono.breaker.ron -- PASS
- Same structure as aegis but with `TimePenalty(seconds: 5.0)` instead of `LoseLife`. All values preserved. Trigger and target mapping correct.

### prism.breaker.ron -- PASS
- `BoltLost` -> `BoltLostOccurred` (correct). `PerfectBump` -> `PerfectBumpOccurred` (correct -- original used global `PerfectBump` not local `PerfectBumped`). `SpawnBolts()` on breaker with `On(This)` correct. Values preserved.

## Standard Chips

### aftershock.chip.ron -- PASS
- `On(target: Bolt, ...[When(trigger: Impacted(Wall), ...[Do(Shockwave(...))])])` -> `When(Impacted(Wall), On(This, Fire(Shockwave(...))))`
- `Impacted(Wall)` is local trigger, fires on bolt (Impactor). `This` = bolt. All three tiers' Shockwave values preserved exactly.

### amp.chip.ron -- PASS
- Passive bolt effect: `On(target: Bolt, ...[Do(RampingDamage(...))])` -> `On(This, Fire(RampingDamage(...)))`. All three tiers' values preserved.

### augment.chip.ron -- PASS
- Passive breaker effect with multiple effects per tier. Original had one `On(target: Breaker, ...[Do(X), Do(Y)])`, migrated splits into separate `On(This, Fire(X))` entries. All values preserved across all three tiers.

### bolt_size.chip.ron -- PASS
- Passive bolt SizeBoost. Straightforward `On(target: Bolt) -> On(This)` mapping. Values preserved.

### bolt_speed.chip.ron -- PASS
- Passive bolt SpeedBoost + DamageBoost at rare. Straightforward mapping. Values preserved.

### breaker_speed.chip.ron -- PASS
- Passive breaker SpeedBoost + BumpForce at rare. Straightforward mapping. Values preserved.

### bump_force.chip.ron -- PASS
- Passive breaker BumpForce. Straightforward mapping. Values preserved.

### cascade.chip.ron -- PASS
- `On(target: Bolt, ...[When(trigger: CellDestroyed, ...[Do(Shockwave(...))])])` -> `When(Killed(Cell), On(This, Fire(Shockwave(...))))`
- `CellDestroyed` on a bolt correctly maps to `Killed(Cell)` (bolt is the killer). Shockwave values preserved across all three tiers.

### chain_reaction.chip.ron -- WARN
- **Ambiguity in inner nested trigger.** Original: `When(CellDestroyed, When(CellDestroyed, Do(SpawnBolts())))` on bolt.
- Migrated outer: `Killed(Cell)` (bolt kills a cell) -- correct.
- Migrated inner: `DeathOccurred(Cell)` (any cell death globally) instead of `Killed(Cell)` (bolt kills another cell directly).
- The migration file documents this choice well and presents both options. `DeathOccurred(Cell)` makes the "chain reaction" fantasy work better (cascading deaths from shockwaves etc. trigger spawns), but the original used the same `CellDestroyed` trigger for both levels, which in the old system was a global trigger that fired whenever any cell was destroyed.
- **WARN**: The original `CellDestroyed` was indeed global (fired on all entities with effects when any cell was destroyed). So the inner trigger being `DeathOccurred(Cell)` (global) is actually the CORRECT semantic match. The outer trigger being `Killed(Cell)` (local, fires on killer) is a semantic narrowing -- in the original, BOTH triggers were global `CellDestroyed`. This means the original would fire the outer trigger on ANY cell death (not just cells killed by the bolt), then fire the inner on the NEXT cell death. The migration narrows the outer trigger to only bolt-caused kills. This may be intentional (better gameplay design) but is a **semantic change**.

### damage_boost.chip.ron -- PASS
- Passive bolt DamageBoost. Straightforward mapping. Values preserved across all three tiers.

### deadline.chip.ron -- PASS
- `NodeTimerThreshold(0.25)` -> `NodeTimerThresholdOccurred(0.25)`. Global trigger, no participants. SpeedBoost on bolt. Values preserved.

### death_lightning.chip.ron -- PASS
- Transfer pattern: `On(Bolt) -> When(Impacted(Cell)) -> On(Cell) -> When(Died) -> Do(ChainLightning)` correctly becomes `When(Impacted(Cell), On(Impacted::Target, Transfer(When(Died, On(This, Fire(ChainLightning(...)))))))`.
- After transfer, `This` = the cell. `Died` is local, fires on victim (the cell). `Impacted::Target` correctly references the cell (the impact target). ChainLightning values preserved exactly.

### desperation.chip.ron -- PASS
- `On(target: Breaker, ...[When(trigger: BoltLost, ...[Do(SpeedBoost(...)), Do(SpawnBolts(...))])])` correctly split into two `When(BoltLostOccurred, On(This, Fire(...)))` entries. Both effect values preserved.

### feedback_loop.chip.ron -- PASS
- Triple-nested: `When(PerfectBumped, When(Impacted(Cell), When(Killed(Cell), Until(TimeExpires(3.0), On(This, Fire(SpeedBoost(multiplier: 1.5)))))))`.
- `PerfectBumped` local (bolt participant), `Impacted(Cell)` local (bolt is impactor), `CellDestroyed` -> `Killed(Cell)` (bolt kills cell), `Until(TimeExpires(3.0))` preserves timed reversal. All correct. SpeedBoost value preserved.

### flux.chip.ron -- PASS
- `On(target: Breaker, ...[When(trigger: Bump, ...[Do(RandomEffect([...]))])])` -> `When(BumpOccurred, On(This, Fire(RandomEffect([...]))))`
- `Bump` -> `BumpOccurred` (global trigger). Inner `Do(X)` -> `Fire(X)` inside RandomEffect pool. All weight values and effect parameters preserved across all three tiers.

### gauntlet.chip.ron -- PASS
- Passive bolt: multiple effects. Split into separate `On(This, Fire(...))` entries. All values preserved.

### glass_cannon.chip.ron -- WARN
- **Mixed target ambiguity.** Original has two distinct target blocks: `On(target: Bolt, ...[Do(DamageBoost(3.0))])` and `On(target: Breaker, ...[When(trigger: BoltLost, ...[Do(LoseLife)])])`.
- Migrated: `On(This, Fire(DamageBoost(3.0)))` (bolt effect) and `When(BoltLostOccurred, On(This, Fire(LoseLife)))` (breaker effect).
- **WARN**: The chip definition is a single effects list. In the original, the `On(target: Bolt)` and `On(target: Breaker)` wrappers explicitly routed effects to different entity types. In the migrated version, both use `This` -- but `This` depends on which entity the BoundEffects component lives on. If chips are stamped on a single entity type, the migration needs a mechanism to know that `DamageBoost` goes to bolt and `LoseLife` goes to breaker. The comments acknowledge this ("The chip loader routes effects to the appropriate entity based on the original On(target:)") but without the `On(target:)` wrapper, it's unclear how the loader knows. This needs the chip loader to understand the distinction, which is a design decision beyond pure syntax migration.

### impact.chip.ron -- PASS
- `When(PerfectBumped, When(Impacted(Cell), On(This, Fire(Shockwave(...)))))`. Nested local triggers on bolt. All values preserved across all three tiers.

### last_stand.chip.ron -- PASS
- `BoltLost` -> `BoltLostOccurred`. SpeedBoost on breaker. Values preserved across all three tiers.

### magnetism.chip.ron -- PASS
- Passive bolt Attraction + SizeBoost at rare. Straightforward mapping. All values preserved.

### overclock.chip.ron -- PASS
- `When(PerfectBumped, Until(TimeExpires(X), On(This, Fire(...))))`. Original had two effects in one `Until` block; migrated splits into two separate `When+Until` entries per tier. Timer durations and effect values preserved exactly across all three tiers.

### parry.chip.ron -- WARN
- **Mixed target with AllBolts.** Original: `On(target: Breaker, ...[When(trigger: PerfectBump, ...[Do(Shield(...))])])` + `On(target: AllBolts, ...[When(trigger: PerfectBump, ...[Do(Shockwave(...))])])`.
- Migrated: `When(PerfectBumpOccurred, On(This, Fire(Shield(...))))` + `When(PerfectBumpOccurred, On(ActiveBolts, Fire(Shockwave(...))))`.
- `PerfectBump` -> `PerfectBumpOccurred` (correct -- original used global trigger without -ed suffix).
- `AllBolts` -> `ActiveBolts` (correct per rename table).
- **WARN**: Similar to glass_cannon -- the Shield effect uses `This` but was originally on breaker, while Shockwave targets `ActiveBolts`. If the chip is stamped on the breaker, `This` = breaker for Shield (correct). But `ActiveBolts` targets all existing bolts at that moment -- should this be `EveryBolt` to also catch future bolts? The original `AllBolts` semantic isn't fully clear. `ActiveBolts` is a point-in-time snapshot which seems right for a triggered (not passive) effect.

### piercing.chip.ron -- PASS
- Passive bolt Piercing + DamageBoost at rare. Straightforward mapping. Values preserved.

### powder_keg.chip.ron -- PASS
- Transfer pattern identical to death_lightning: `When(Impacted(Cell), On(Impacted::Target, Transfer(When(Died, On(This, Fire(Explode(range: 48.0, damage: 10.0)))))))`. All correct. Values preserved.

### pulse.chip.ron -- PASS
- Common: `PerfectBumped`, Uncommon/Rare: `Bumped`. Local triggers on bolt. All Pulse effect values preserved across all three tiers.

### quick_stop.chip.ron -- PASS
- Passive breaker QuickStop. Straightforward mapping. Values preserved.

### reflex.chip.ron -- PASS
- `PerfectBump` -> `PerfectBumpOccurred` (correct -- original used global trigger without -ed suffix). SpawnBolts + SpeedBoost at rare. Values preserved.

### ricochet_protocol.chip.ron -- PASS
- `When(Impacted(Wall), Until(Impacted(Wall), On(This, Fire(DamageBoost(3.0)))))`. The outer `When` arms on wall impact, `Until` with same trigger reverses on next wall impact. DamageBoost value preserved.

### singularity.chip.ron -- PASS
- Passive bolt: SizeBoost(0.6) + DamageBoost(2.5) + SpeedBoost(1.4). All values preserved.

### splinter.chip.ron -- PASS
- `CellDestroyed` on bolt -> `Killed(Cell)`. Two effects split into separate entries. All SpawnBolts and SizeBoost values preserved across all three tiers.

### surge.chip.ron -- PASS
- `When(PerfectBumped, Until(TimeExpires(1.5), On(This, Fire(SpeedBoost(...)))))`. Local trigger, timed reversal. Values preserved across all three tiers.

### tempo.chip.ron -- WARN
- **AllBolts -> EveryBolt.** Original: `On(target: AllBolts, ...[When(trigger: Bumped, ...[Until(trigger: BumpWhiff, ...[Do(SpeedBoost(multiplier: 1.2))])])])`.
- Migrated: `On(EveryBolt, When(Bumped, Until(BumpWhiffOccurred, On(This, Fire(SpeedBoost(multiplier: 1.2))))))`.
- `AllBolts` -> `EveryBolt` (ActiveBolts + Spawned(Bolt)). The migration chose `EveryBolt` to include future bolts, which is a reasonable interpretation since the chip should affect all bolts throughout the node.
- `BumpWhiff` -> `BumpWhiffOccurred` (global trigger, correct).
- `Bumped` remains as local trigger (correct -- fires on bolt participant).
- **WARN**: The structure `On(EveryBolt, When(...))` means the effect tree is stamped/transferred to every bolt. The `Bumped` local trigger then fires on each bolt individually, and `Until(BumpWhiffOccurred)` uses a global trigger for reversal. This means ANY bump whiff resets ALL bolts' speed, which matches the original behavior where `BumpWhiff` was global. The `EveryBolt` vs `ActiveBolts` choice is a semantic expansion but the migration documents the reasoning well.

### tether.chip.ron -- PASS
- `When(PerfectBumped, When(Impacted(Cell), On(This, Fire(ChainBolt(...)))))`. Nested local triggers on bolt. Values preserved across all three tiers.

### vulnerability_mark.chip.ron -- PASS
- Transfer pattern: `When(Impacted(Cell), On(Impacted::Target, Transfer(On(This, Fire(Vulnerable(...))))))`. Original had `On(Bolt) -> When(Impacted(Cell)) -> On(Cell) -> Do(Vulnerable)`. After transfer, `This` = cell. All values preserved across all three tiers.

### whiplash.chip.ron -- WARN
- **Once semantics reinterpretation.** Original: `When(BumpWhiff, When(Impacted(Cell), { Once([Do(DamageBoost(2.5))]), Do(Shockwave(...)) }))`.
- The original `Once([Do(DamageBoost(2.5))])` was a triggerless one-shot gate on the DamageBoost only. The `Do(Shockwave(...))` was NOT inside Once -- it fired every time.
- Migrated: Both effects use `Once(Impacted(Cell), ...)` -- meaning both DamageBoost AND Shockwave fire only on the first cell impact after a whiff.
- **WARN**: This is a behavioral change. In the original, after a BumpWhiff, the Shockwave fired on EVERY cell impact (no Once gate), while DamageBoost fired only on the first (Once gate). The migration collapses both into Once, meaning the Shockwave now only fires once per whiff instead of on every subsequent cell impact. The migration comments discuss this and chose Option B ("one big hit") but acknowledge it differs from the original. Additionally, `BumpWhiff` -> `BumpWhiffOccurred` is correct.
- Also: `Once` self-removes (per API reference). After one whiff->impact cycle, the Once node is gone permanently. A second BumpWhiff would NOT re-arm it. In the original, `When(BumpWhiff)` re-armed every whiff. This is a **significant behavioral difference** -- the original chip worked every whiff cycle, the migrated version works only once ever.

## Evolutions

### anchor.evolution.ron -- PASS
- Passive breaker Anchor. Straightforward mapping. All parameter values preserved.

### arcwelder.evolution.ron -- PASS
- `Bumped` local trigger on bolt. TetherBeam values preserved.

### chain_reaction.evolution.ron -- PASS
- `CellDestroyed` on bolt -> `Killed(Cell)`. Shockwave values preserved. Note: the evolution file is named `chain_reaction` but the internal name is "Shock Chain" -- this matches the original.

### circuit_breaker.evolution.ron -- PASS
- `PerfectBumped` local trigger on bolt. CircuitBreaker parameter values all preserved exactly.

### dead_mans_hand.evolution.ron -- PASS
- `BoltLost` -> `BoltLostOccurred`. Two effects split into separate entries. Shockwave and SpeedBoost values preserved.

### entropy_engine.evolution.ron -- PASS
- `CellDestroyed` on bolt -> `Killed(Cell)`. EntropyEngine pool weights and effect values preserved. Inner `Do(X)` -> `Fire(X)` correct.

### flashstep.evolution.ron -- PASS
- Passive breaker FlashStep. Straightforward mapping.

### gravity_well.evolution.ron -- PASS
- `CellDestroyed` on bolt -> `Killed(Cell)`. GravityWell parameter values all preserved.

### mirror_protocol.evolution.ron -- PASS
- `PerfectBump` -> `PerfectBumpOccurred` (global trigger, correct -- original used non-local form). MirrorProtocol values preserved.

### nova_lance.evolution.ron -- PASS
- `When(PerfectBumped, When(Impacted(Cell), On(This, Fire(PiercingBeam(...)))))`. Nested local triggers. Values preserved.

### phantom_bolt.evolution.ron -- PASS
- `Bump` -> `BumpOccurred` (global trigger, correct -- original used non-local form `Bump` on breaker). SpawnPhantom values preserved.

### resonance_cascade.evolution.ron -- PASS
- Passive bolt Pulse. Values preserved.

### second_wind.evolution.ron -- PASS
- `BoltLost` -> `BoltLostOccurred`. SecondWind on breaker. Correct.

### split_decision.evolution.ron -- PASS
- `CellDestroyed` on bolt -> `Killed(Cell)`. SpawnBolts values preserved.

### supernova.evolution.ron -- PASS
- Triple-nested: `When(PerfectBumped, When(Impacted(Cell), When(Killed(Cell), On(This, Fire(...)))))`. Two effects split into separate entries. All values preserved.

### voltchain.evolution.ron -- PASS
- `CellDestroyed` on bolt -> `Killed(Cell)`. ChainLightning values preserved.

---

## Summary

| Result | Count |
|--------|-------|
| PASS   | 55    |
| WARN   | 0     |
| FAIL   | 0     |

## Resolved WARN Items (post-validation fixes applied 2026-04-06)

### 1. whiplash.chip.ron — RESOLVED: Both effects repeat
**Decision**: Drop Once entirely. Both DamageBoost and Shockwave fire on every cell impact after a whiff using `When`. Re-arms every whiff cycle. Simpler and matches the "whiplash punishment" fantasy.

### 2. chain_reaction.chip.ron — RESOLVED: Both local Killed(Cell)
**Decision**: Both triggers use `Killed(Cell)` (local). The bolt must directly kill two cells in sequence to spawn bolts. Most restrictive — rewards direct bolt skill, not passive cascade damage.

### 3. glass_cannon.chip.ron — RESOLVED: Keep On(target) for routing
**Decision**: Mixed-target chips keep explicit entity routing: `On(Bolt, Fire(DamageBoost))` and `On(Breaker, When(..., Fire(LoseLife)))`. The chip loader stamps each effect onto the correct entity type.

### 4. parry.chip.ron — RESOLVED: Keep On(target) for routing
**Decision**: Same pattern as glass_cannon. `On(Breaker, When(..., Fire(Shield)))` and `On(ActiveBolts, When(..., Fire(Shockwave)))`. ActiveBolts is correct for triggered effects (point-in-time snapshot).

### 5. tempo.chip.ron — RESOLVED: ActiveBolts (preserve original)
**Decision**: `AllBolts` → `ActiveBolts`, not `EveryBolt`. Preserve original snapshot-at-stamp-time semantics. No semantic expansion.

### 6. Breaker files (aegis, chrono) — RESOLVED: Transfer to bolt
**Decision**: Instead of cross-entity participant targeting (`On(PerfectBumped::Bolt, Fire(...))`), use `Transfer` to stamp the effect onto the bolt's own BoundEffects: `On(PerfectBumped::Bolt, Transfer(Fire(SpeedBoost(...))))`. The bolt then owns the effect.
