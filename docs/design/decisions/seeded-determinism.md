# Seeded Determinism

**Decision**: Introduced in Phase 4. Run seed drives all randomness.

## Model

- User-selectable seed on the RunSetup screen (or random if not specified)
- Same seed = same node sequence, same chip offerings, same cell layouts
- `GameRng` (ChaCha8Rng) seeded per-run, used for all gameplay randomness
- FixedUpdate physics ensures deterministic simulation across hardware

## Rationale

Seeds were deferred from Phase 2 because they're meaningless with only 3 hand-authored layouts. Phase 4 introduces procedural node sequences and chip offerings — seeds become meaningful.

Deterministic runs enable: seed sharing ("try my seed"), competitive play, bug reproduction, and scenario testing.

## Retrofit Cost

Accepted. Existing systems (node selection, chip offerings) need to be plumbed through `GameRng` rather than using ad-hoc randomness. This is a known cost, traded against the premature complexity of seeding a system with nothing to seed.
