# Seeded Determinism

**Decision**: Introduced in Phase 4. Run seed drives all randomness.

## Model

- User-selectable seed on the RunSetup screen (or random if not specified)
- Same seed = same node sequence, same chip offerings, same cell layouts
- `GameRng` (ChaCha8Rng) seeded per-run; serves as root entropy for deriving per-domain sub-seeds. Gameplay systems draw from domain-specific resources (`NodeSequenceRng`, `BoltRng`, `ChipRng`, `ProtocolRng`, `HazardRng`, `EffectBaseSeed`) — not from `GameRng` directly. See `docs/architecture/rng.md`.
- FixedUpdate physics ensures deterministic simulation across hardware

## Rationale

Seeds were deferred from Phase 2 because they're meaningless with only 3 hand-authored layouts. Phase 4 introduces procedural node sequences and chip offerings — seeds become meaningful.

Deterministic runs enable: seed sharing ("try my seed"), competitive play, bug reproduction, and scenario testing.

## Retrofit Cost

Accepted. Systems were plumbed through `GameRng` initially; Wave 0 of the node-sequencing refactor migrated all call sites to domain-specific RNG resources. The original cost was traded against the premature complexity of seeding a system with nothing to seed.
