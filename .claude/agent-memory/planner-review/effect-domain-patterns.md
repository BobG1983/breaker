---
name: effect-domain-patterns
description: Patterns and borrow rules for effect fire/reverse functions, lint constraints, normal conventions, and recurring spec pitfalls
type: project
---

## Effect Domain — fire()/reverse() Patterns

### Correct EntityWorldMut conditional-insert pattern (Bevy 0.18)
Shield effect uses this safe borrow pattern:
```rust
if let Some(mut comp) = world.get_mut::<Comp>(entity) {
    // mutate comp
} else {
    world.entity_mut(entity).insert(Comp::default());
}
```
Each borrow is a temporary that drops before the next borrow begins — no named `EntityWorldMut` variable held across a subsequent `world.get_mut` call.

### BAD pattern (borrow-checker failure):
```rust
let mut entity_mut = world.entity_mut(entity);  // named binding holds borrow
if !entity_mut.contains::<Comp>() {
    entity_mut.insert(Comp::default());
}
let mut active = world.get_mut::<Comp>(entity).unwrap(); // FAIL: entity_mut still in scope
```
This fails to compile because `entity_mut` holds a mutable borrow of `world` past the `world.get_mut` call.

### Lint constraints
- `unwrap_used = "warn"` and `expect_used = "warn"` in workspace Cargo.toml
- Avoid `.unwrap()` and `.expect()` in production code — use `if let` or `?` instead
- Writer-code must use lint-safe patterns even when specs show `.unwrap()`

## Test Name Consistency
- Always cross-check test names between test spec and code spec (Failing Tests section)
- A mismatch in test function name means writer-code will look for a test that doesn't exist at the RED gate

## TDD Workflow Note
- Code spec Failing Tests section should only name tests that ALREADY EXIST in the codebase (written by writer-tests)
- Code spec should NOT contain instructions about what writer-tests should do — that phase is already complete by the time writer-code runs
- A note like "the test will need to be updated by writer-tests" in a code spec is a spec error: either redundant or a workflow violation

**Why:** Discovered during wave1a-stat-boost-lazy-init review (2026-03-30).
**How to apply:** Flag any code spec that includes writer-tests instructions or contains borrow patterns that assign `EntityWorldMut` to a named variable before a subsequent `world.get_mut` call.

## Effects That Read From Non-Triggering Entity

When an effect reads data from a different entity than the one `fire()` is called on (e.g., MirrorProtocol reads position and BoundEffects from the PRIMARY BOLT, not the breaker), two common spec errors arise:

1. **Spawn position source confusion**: Code spec may default to using `entity_position(world, entity)` (the triggering entity) when the spec actually requires using the primary bolt's position. These must be consistent across test spec and code spec.
2. **Early-return-on-despawn contradiction**: If fire() is supposed to succeed even when the triggering entity is despawned (because it reads from another entity), the early-return guard `if world.get_entity(entity).is_err() { return; }` directly contradicts that behavior. Both specs must agree on whether fire() is entity-agnostic or entity-dependent.
3. **BoundEffects source proof gap**: When the test spec says BoundEffects are inherited from entity B (not A), the test setup must give B non-empty BoundEffects AND give A different (or empty) BoundEffects, to prove the source is B. Setting only B's BoundEffects doesn't disambiguate.

**Why:** Discovered during wave2d-mirror-protocol review (2026-03-30).

## Query Optionality for New Components in Existing Systems

When a new feature adds new components to an existing ECS query, and those components are NOT spawned on entities tested by EXISTING tests:
- Making the new components required (`&'static T`) in the query causes all existing test entities (which lack the component) to silently fall off the query — systems will not run for them, tests will still pass but for the wrong reason (no assertion failure, system never executes).
- The correct pattern is to make feature-specific components optional: `Option<(&'static T, &'static U)>` as a tuple option, and guard the new code path on `if let Some((t, u)) = optional_data`.
- Always check whether existing tests in the file spawn the new required components. If not, optionality is required.

**Why:** Discovered during wave2e-flashstep-teleport review (2026-03-30). `DashQuery` expansion with required `Position2D`/`BreakerWidth` would silently invalidate all 13 existing dash tests.
**How to apply:** When reviewing code specs that expand an existing query, check the existing test file's spawn helper. If it doesn't include the new components, flag required vs optional as a BLOCKING issue.

## Redundant Component vs. Existing Data Already In-System

When a code spec proposes a new component to track state that is ALREADY encoded in an existing component's field (e.g., `BreakerTilt.ease_start` already encodes last-dash direction via its sign), flag this as scope creep. The redundant component:
1. Requires initialization in every spawn site (`spawn_or_reuse_breaker`, `reset_breaker`)
2. Must be kept in sync with the authoritative source
3. Adds noise to the query
4. Creates a three-way divergence risk (spec mechanism, test setup, initialization sites)

Check that test spec and code spec agree on THE SAME detection/storage mechanism. If they don't, it's a BLOCKING cross-spec conflict.

**Why:** Discovered during wave2e-flashstep-teleport review (2026-03-30). `LastDashDirection` was proposed by code spec but `BreakerTilt.ease_start` already carries the same information and is already reset by `reset_breaker`.

## Collision Normal Conventions — Two Different Systems, Opposite Semantics

The project has two different normal conventions that must NOT be unified under a single helper:

### CCD normal (rantzsoft_physics2d `SweepHit.normal`, `RayHit.normal`)
"Outward face normal at the contact point" — points AWAY from the hit surface TOWARD the approaching bolt.
- Bolt moving +Y hits cell BOTTOM face: normal = `Vec2::NEG_Y` → `ImpactSide::Bottom`
- Bolt moving -Y hits cell TOP face: normal = `Vec2::Y` → `ImpactSide::Top`
- Bolt moving +X hits cell LEFT face: normal = `Vec2::NEG_X` → `ImpactSide::Left`
- Bolt moving -X hits cell RIGHT face: normal = `Vec2::X` → `ImpactSide::Right`

Mapping: `NEG_Y → Bottom`, `Y → Top`, `NEG_X → Left`, `X → Right`

### Push-out normal (bolt_wall_collision `faces` array)
The normal is the push-out DIRECTION — points AWAY from the surface toward where the bolt should go.
- Bolt inside ceiling wall, pushed DOWN to bottom face: normal = `Vec2::NEG_Y` → `ImpactSide::Top`
- Bolt inside left wall, pushed RIGHT to right face: normal = `Vec2::X` → `ImpactSide::Left`

Mapping: `NEG_Y → Top`, `Y → Bottom`, `NEG_X → Right`, `X → Left` (opposite Y semantics for top/bottom)

Wait — actually re-verify: left wall scenario: wall at x=-5, half=5, bolt at x=-2. Nearest face = RIGHT face of wall (inner edge). Normal = `Vec2::X` (pushed right). ImpactSide = `Left` (hit the left wall). So `Vec2::X` → `Left` for wall. And for CCD: `Vec2::X` → `Right` (bolt hit right face of cell). THESE ARE OPPOSITE.

**Rule:** The single `impact_side_from_normal` helper CANNOT serve both wall and cell collision. Two separate mappings are needed. Any spec that proposes a unified helper is wrong.

**How to apply:** When a spec proposes `impact_side_from_normal` used in both wall collision and cell CCD, flag it as BLOCKING. Each system needs its own inline mapping or separate helpers with different logic.

**Why:** Discovered during wave2g-last-impact review (2026-03-30). Wall collision uses push-out direction (toward bolt escape); CCD uses surface outward normal (toward incoming bolt). These are semantically opposite for left/right and top/bottom surfaces.

## SpawnBolts inherit=true Query Pattern (Bevy 0.18)

`world.query_filtered::<&BoundEffects, (With<Bolt>, Without<ExtraBolt>)>()` returns entities that both HAVE `Bolt`, DO NOT have `ExtraBolt`, AND HAVE `BoundEffects`. An entity with `Bolt`, no `ExtraBolt`, but NO `BoundEffects` component is invisible to this query. The result is `None` — same as "no primary bolt at all." These two cases are semantically distinct but collapse to the same outcome under this query.

When reviewing specs for this kind of query: the test for "primary bolt exists but has no BoundEffects component" produces the same result as "no primary bolt" — both valid to fold into a single test. But note the ambiguity if future tests try to distinguish them.

**Why:** Discovered during wave3b-spawn-bolts-inherit review (2026-03-30).

## Push/Pop in ECS Systems vs. &mut World Functions

When a code spec proposes push/pop into a `Vec<f32>` component (e.g., `ActiveBumpForces`) from within a Bevy SYSTEM (not from a `fire()`/`reverse()` function that takes `&mut World`):

1. **The system cannot use `world.get_mut()` directly** — systems don't take `&mut World` as a parameter.
2. **Commands cannot push/pop** — `commands.entity(e).insert(ActiveBumpForces(...))` REPLACES the component, destroying existing entries from other effects.
3. **The correct approach** is to add `Option<&'static mut ActiveBumpForces>` to the system's query tuple, then mutate the `Vec` in-place via the query field. NOTE: `EffectiveBumpForce` was removed in the Effective* cache-removal refactor — use `ActiveBumpForces.total()` on demand instead.
4. **Lazy-insert when absent**: If the query field is `None`, use `commands.entity(e).insert(ActiveBumpForces(vec![value]))` only when no other effects have contributed — but this still loses existing values. The robust approach is to require `ActiveBumpForces` be always present on breaker entities (initialized at spawn), avoiding the conditional insert entirely.

Any code spec that says "push into `ActiveBumpForces` in `tick_anchor`" MUST specify that `AnchorTickQuery` is extended to include `Option<&'static mut ActiveBumpForces>`. Without this, the implementation has no way to read-modify-write the vec. Flag as BLOCKING if missing.

Also check import paths: `effect.rs` at `src/effect/effects/anchor/effect.rs` needs `super::super::bump_force` (not `super::bump_force`) to reach `src/effect/effects/bump_force.rs`.

**Why:** Discovered during wave2f-anchor-bump review (2026-03-30).

## System Signature Gaps — Parenthetical Parameters

When a code spec writes a formal system signature and then parenthetically notes "need X parameter" in the logic steps below, writer-code will implement only the formal signature. The parenthetical note is invisible.

Pattern to watch for:
```
fn maintain_foo(mut commands: Commands, mut active: ResMut<FooActive>, items: Query<Entity, With<Item>>)
```
followed by "step 3: despawn all existing FooBeam entities (need `beams: Query<Entity, With<FooBeam>>` parameter)."

**Rule:** Any parameter mentioned only in a step body and not in the formal signature is a BLOCKING spec error. Every parameter must appear in the formal signature at item-top.
**Why:** Discovered during wave3a-tether-chain review (2026-03-30). `chain_beams` query missing from `maintain_tether_chain` formal signature.

## Dispatch Function Splitting in fire.rs / reverse.rs

`EffectKind::fire()` is split across `fire()`, `fire_aoe_and_spawn()`, `fire_utility_and_spawn()`, `fire_breaker_effects()`. The `TetherBeam` arm is in `fire_breaker_effects()`. Similarly for `reverse_breaker_effects()`.

**Rule:** Code specs updating dispatch must name the exact sub-function — not just "the match arm in fire.rs."
**Why:** Discovered during wave3a-tether-chain review (2026-03-30).

## Helper-Based Tests Do Not Need Signature Updates

When fire()/reverse() signatures change, only tests calling them DIRECTLY need updating. Tests using direct-spawn helpers (e.g., `spawn_tether_beam()`) bypass fire() entirely — NO changes needed.

**Rule:** Code specs must distinguish which test files use helpers vs direct calls. "Update all tests" without this distinction will cause unnecessary edits. tick_damage_tests/ and tick_lifetime_tests.rs both use spawn_tether_beam() — they need zero updates.
**Why:** Discovered during wave3a-tether-chain review (2026-03-30).

## Ambiguous "OR" in Failing Tests File Paths

A code spec Failing Tests entry that says "4 tests in file_a.rs or file_b.rs" is BLOCKING. Writer-code reads Failing Tests to know where existing tests live. "Or" means the decision was never made.

**Rule:** Every Failing Tests entry must name exactly one file. Resolve ambiguity before launching writers.
**Why:** Discovered during wave3a-tether-chain review (2026-03-30).
