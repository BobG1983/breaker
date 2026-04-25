# SourceId

`SourceId` is the canonical attribution string used throughout the damage and
effect pipelines to identify what caused a given event. It is a thin wrapper
around `Cow<'static, str>` defined in `rantzsoft_dmg`, and is constructed
exclusively through the game-side `SourceIdExt` typestate builder
(`breaker-game/src/shared/source_id_ext.rs`).

## Why a builder

The crate-side `SourceId` is intentionally generic — it has no knowledge of
chips, protocols, hazards, or any other game vocabulary. Format strings live
on the game side so the namespace conventions can evolve without touching
`rantzsoft_dmg`. The builder is the single legal path to constructing
namespace-correct values: callers cannot accidentally produce a malformed
attribution string because the typestate forces them through one of the four
namespaces.

## Namespace formats

| Namespace | Builder | Format |
|-----------|---------|--------|
| Chip | `SourceId::chip(template)[.rarity(r)].build()` | `chip:<template>[:<rarity>]` |
| Protocol | `SourceId::protocol(kind)[.action(a)].build()` | `protocol:<name>[:<action>]` |
| Hazard | `SourceId::hazard(kind)[.instance(id)].build()` | `hazard:<name>[:<u64>]` |
| Armed | `<inner>.armed()` | `<inner>:armed` (suffix on any other format) |

`<rarity>` is the lowercase variant of `Rarity` (`common`, `rare`, `epic`,
etc). `<action>` is a free-form discriminator chosen by the protocol (e.g.
`burnout:amplify`). `<u64>` is the concrete hazard instance identifier so
multiple instances of the same hazard kind do not alias.

## Reader helpers

Format-aware readers live alongside the builder so format strings appear in
exactly one file:

- `SourceId::is_armed(&self) -> bool` — true when the suffix is `:armed`
- `SourceId::unwrap_armed(&self) -> Cow<'_, str>` — strip the armed suffix
- `SourceId::starts_with(&self, ns: &str) -> bool` — namespace check
- `SourceId::extract_hazard_instance(&self) -> Option<u64>` — parse the
  instance id from a hazard-format SourceId

Anything that needs to inspect a `SourceId` must go through these helpers
rather than parsing the string directly.

## Why the trait lives game-side

Putting the trait in `rantzsoft_dmg` would couple the crate to game-specific
enums (`ProtocolKind`, `HazardKind`, `Rarity`). The current split keeps
`rantzsoft_dmg` reusable for any 2D ECS game while letting the brickbreaker
project lock in its attribution conventions in one file.

See the W5 plan in `docs/todos/detail/port-to-rantzsoft-dmg.md` (now
historical) for the migration rationale.
