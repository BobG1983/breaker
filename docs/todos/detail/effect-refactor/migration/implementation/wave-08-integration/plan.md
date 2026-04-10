# Wave 8: Integration Tests (RED → GREEN)

## Specs
- [spec-tests.md](spec-tests.md) — Behavioral test spec
- No code spec — GREEN phase fixes whatever the integration tests expose

## Goal
Write integration tests exercising cross-domain flows end-to-end, then fix any issues.

## RED phase — write tests

### Suggested integration tests

- **Bump → effect**: BoundEffects with When(Bumped, Fire(SpeedBoost)). Send BumpPerformed. Verify EffectStack populated.
- **Multi-stage arming**: When(Bumped, When(Impacted(Cell), Fire(Explode))). Bump arms inner When. Impact fires Explode.
- **Bumped twice**: When(Bumped, When(Bumped, Fire(SpeedBoost))). First bump → nothing. Second bump → SpeedBoost.
- **Until timer**: Until(TimeExpires(0.5), Fire(SpeedBoost)). Fires immediately. Tick 0.5s. Reverses.
- **During condition**: During(NodeActive, Fire(SpeedBoost)). Enter Playing → fires. Exit Playing → reverses.
- **Once consumption**: Once(Bumped, Fire(Shockwave)). First bump → shockwave. Second bump → nothing.
- **Sequence ordering**: When(Bumped, Sequence([Fire(SpeedBoost), Fire(DamageBoost)])). Both stacks populated.
- **On redirection**: When(Bumped, On(Bump(Breaker), Fire(SpeedBoost))). SpeedBoost on breaker, not bolt.
- **Death pipeline e2e**: Cell Hp(1), DamageDealt → detect → KillYourself → Destroyed → despawn.
- **Die bypass**: When(Bumped, Fire(Die)). KillYourself sent directly, no Hp change.
- **Cascade delay**: Death-triggered DamageDealt NOT processed same frame, IS processed next frame.
- **Global + On**: DeathOccurred(Cell) with On(Death(Killer), Fire(SpeedBoost)). SpeedBoost on killer.
- **Passive stacking**: Two sources stamp SpeedBoost. Remove one. Aggregate reflects remainder.
- **Spawn watcher**: Register watcher for Bolt. Spawn bolt. Verify tree stamped.

## RED gate
All tests compile. Tests that expose issues fail.

## GREEN phase — fix
Fix system ordering, command flush timing, missing components, or other integration issues.

## GREEN gate
All integration tests pass. All previous tests still pass.

## Docs to read
- `effect-refactor/walking-effects/walking-algorithm.md`
- `effect-refactor/walking-effects/when.md` — arming
- `effect-refactor/walking-effects/until.md` — timer lifecycle
- `effect-refactor/evaluating-conditions/evaluate-conditions.md` — During lifecycle
- `effect-refactor/dispatching-triggers/dispatch-algorithm.md` — context table
- `effect-refactor/migration/plugin-wiring/system-set-ordering.md` — frame ordering
- `unified-death-pipeline/migration/plugin-wiring/system-set-ordering.md` — death pipeline ordering
