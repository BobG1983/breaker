# Planning Reviewer Specs Code Agent Memory

## Domain Knowledge
- [effect-domain-patterns.md](effect-domain-patterns.md) — Patterns for effect fire/reverse, EntityWorldMut borrow rules, lint constraints, spec pitfalls (dispatch split, normal conventions, inherit queries)

## Feedback — Spec Pitfalls
- [Counter resource state missing from ordering specs](feedback_counter_resource_state.md) — specs for monotonic counters must verify post-call resource state in addition to component value
- [Test/impl spec path mismatches](feedback_test_impl_path_mismatch.md) — parallel specs often disagree on whether tests go in tests.rs vs tests/fire_tests.rs; BLOCKING before writer-tests
- [Counter semantics pre vs post increment](patterns_counter_semantics.md) — pre/post increment confusion causes first-value off-by-one; flag "increment then use" descriptions unless starting value is -1

## Feedback — ECS Design
- [Per-owner state as global HashMap resource](patterns_per_owner_state.md) — HashMap<Entity,V> as Resource leaks on despawn; use component-on-owner instead
- [World borrow conflict in fire() functions](patterns_world_borrow_conflict.md) — holding mutable resource borrow while querying world fails; must drop resource borrow before query

## Session History
See [ephemeral/](ephemeral/) — not committed.
