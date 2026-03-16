---
name: review-fix-review-findings-2026-03-16
description: Architecture review of fix/review-findings branch — animate_fade_out move, FadeOut shared pattern, multiplier precedence
type: project
---

# Architecture Review: fix/review-findings branch (2026-03-16)

## Verdict: PASS — 0 critical violations, 1 observation

---

## Question 1: animate_fade_out move from bolt to UI domain

**Finding: ARCHITECTURALLY SOUND — correct domain placement**

The `animate_fade_out` system was previously in `src/bolt/systems/bolt_lost_feedback.rs` as a co-located function. It has been extracted to `src/ui/systems/animate_fade_out.rs` with its own file following canonical layout.

**Why this is correct:**

1. `animate_fade_out` operates on `FadeOut + TextColor` entities spawned by **multiple domains** (bolt's `spawn_bolt_lost_text`, breaker's `spawn_bump_grade_text` and `spawn_whiff_text`). A system serving multiple producers should NOT live in any single producer's domain — it belongs in a shared consumer domain.

2. The UI domain is the correct home because:
   - The system is purely visual (alpha animation + despawn)
   - It runs in the `Update` schedule, which is the visual/UI schedule per `docs/architecture/ordering.md`
   - It is gated on `PlayingState::Active`, consistent with other UI systems
   - UI already owns HUD rendering, which is the same category of visual concern

3. The system does NOT create cross-domain coupling because:
   - It reads only `shared::FadeOut` (a shared passive type) and Bevy's `TextColor`
   - It does not import any domain-specific types
   - No domain needs to import UI to use FadeOut — they just spawn entities with the shared component

4. **Canonical layout verified:**
   - `src/ui/systems/animate_fade_out.rs` — one system per file
   - `src/ui/systems/mod.rs` — routing-only, exports `animate_fade_out`
   - `src/ui/plugin.rs` — registers the system in `Update` schedule
   - Tests are in-module `#[cfg(test)]` block with comprehensive coverage (6 tests)

**No boundary violation.** No domain was forced to import `crate::ui::*` in production code. The bolt test file (`bolt_lost_feedback.rs`, line 33) does import `crate::ui::systems::animate_fade_out` but only in `#[cfg(test)]` context to wire up a test app — this is acceptable (test helpers are not production coupling).

---

## Question 2: UI domain owning a generic fade-out system for cross-domain entities

**Finding: ACCEPTABLE — follows established ECS read-only query pattern**

The pattern is: domains spawn entities with `shared::FadeOut` + `TextColor`, and the UI domain's `animate_fade_out` queries ALL such entities generically. This is the correct approach.

**Why:**
- `FadeOut` is defined in `shared.rs` as a passive component, consistent with the shared.rs rule: "passive types only: state enums, cleanup markers, and playfield configuration" (the doc comment on `shared.rs` line 1-3). FadeOut is a marker/lifecycle component similar in nature to `CleanupOnNodeExit`.
- The system queries `(Entity, &mut FadeOut, &mut TextColor)` — both are owned by the entity itself, not by any domain. No cross-domain mutation occurs.
- This is the same pattern as `cleanup_entities<T>` which operates on cleanup markers spawned by multiple domains.

**No messages needed.** No domain communicates to UI about the fade-out; domains just spawn entities with the right components and the UI system picks them up. This is component-driven composition, not hidden coupling.

---

## Question 3: BumpPerfectMultiplier/BumpWeakMultiplier insert_if_new precedence

**Finding: CORRECT — ordering guarantees proper precedence**

The execution chain on `OnEnter(GameState::Playing)`:

```
apply_archetype_config_overrides  .before(init_breaker_params)
init_breaker_params               .after(spawn_breaker)
init_archetype                    .after(init_breaker_params)
```

From `src/breaker/behaviors/plugin.rs` lines 41-44 and `src/breaker/plugin.rs` lines 37-39.

**How the precedence works:**

1. `init_breaker_params` runs FIRST (among the two multiplier-related systems). It uses `.insert_if_new((BumpPerfectMultiplier(1.0), BumpWeakMultiplier(1.0)))` (line 73). On a fresh breaker entity, these identity defaults are inserted.

2. `init_archetype` runs AFTER `init_breaker_params`. It calls `apply_bolt_speed_boosts()` which uses `.insert(BumpPerfectMultiplier(multiplier))` — a normal `.insert()`, NOT `.insert_if_new()`. This overwrites the defaults with archetype-specific values.

**This ordering is correct.** The `insert_if_new` in step 1 provides fallback defaults. The `.insert()` in step 2 overwrites them with archetype values. If no archetype specifies speed boosts, the identity defaults persist. The test `archetype_multipliers_not_overwritten` (init_breaker_params.rs line 223) verifies the reverse case where multipliers are already present when init_breaker_params runs.

**One subtle detail worth noting:** On the second node (persisted breaker), `init_breaker_params` is skipped entirely due to the `Without<BreakerMaxSpeed>` guard (line 25). The multiplier components from the first node persist. `init_archetype` is also skipped via `Without<LivesCount>` (line 67). So second-node multipliers are carried over — this is correct for run-persistent behavior.

---

## Additional Observations (non-blocking)

### Observation 1: bolt test imports UI system

`src/bolt/systems/bolt_lost_feedback.rs` line 33: `use crate::ui::systems::animate_fade_out;`

This is a test-only import (`#[cfg(test)]` block) to wire up `animate_fade_out` in the test app alongside `spawn_bolt_lost_text`. **Not a violation** — test code may reference other domains to build realistic test scenarios. However, if the test only needs to verify that the entity spawns with `FadeOut`, it could drop the animate_fade_out dependency and just check component presence. The existing tests do both (spawn verification AND despawn verification), which requires the UI system. This is a judgment call — the current approach tests the full lifecycle, which is reasonable.

### Observation 2: FadeOut as shared type

`FadeOut` in `shared.rs` fits the established pattern of passive types (alongside `CleanupOnNodeExit`, `CleanupOnRunEnd`). It is a lifecycle/animation marker, not a domain-specific component. Its placement in shared.rs is correct.

---

## Files Reviewed

- `/Users/bgardner/dev/brickbreaker/src/ui/systems/animate_fade_out.rs` — new system, 6 tests
- `/Users/bgardner/dev/brickbreaker/src/ui/systems/mod.rs` — routing-only, correct
- `/Users/bgardner/dev/brickbreaker/src/ui/plugin.rs` — system registration in Update
- `/Users/bgardner/dev/brickbreaker/src/ui/components.rs` — no new components needed
- `/Users/bgardner/dev/brickbreaker/src/bolt/systems/bolt_lost_feedback.rs` — animate_fade_out removed from production, kept in tests
- `/Users/bgardner/dev/brickbreaker/src/bolt/systems/mod.rs` — no longer exports animate_fade_out
- `/Users/bgardner/dev/brickbreaker/src/bolt/plugin.rs` — no longer registers animate_fade_out
- `/Users/bgardner/dev/brickbreaker/src/breaker/systems/init_breaker_params.rs` — insert_if_new for multipliers
- `/Users/bgardner/dev/brickbreaker/src/breaker/behaviors/init.rs` — init_archetype with .insert() override
- `/Users/bgardner/dev/brickbreaker/src/breaker/behaviors/plugin.rs` — ordering chain verified
- `/Users/bgardner/dev/brickbreaker/src/breaker/behaviors/consequences/bolt_speed_boost.rs` — .insert() not .insert_if_new()
- `/Users/bgardner/dev/brickbreaker/src/shared.rs` — FadeOut definition
