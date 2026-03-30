# rantzsoft_* Crates

## Always Use — Never Bypass

Game code (`breaker-game`) must ALWAYS use `rantzsoft_*` APIs for spatial queries, collision detection, physics, and constraint logic. NEVER write custom spatial or physics code in the game crate.

- If the crate **lacks a capability** that's within its domain — **add it to the crate**.
- If the crate's API **is not performant enough** — **improve the crate's performance**.
- If the capability is **outside the crate's domain** — ask before putting it anywhere.

This applies to specs too — specs must mandate `rantzsoft_*` usage and must never spec custom spatial/physics logic in game code.

## Zero Game Knowledge

`rantzsoft_*` crates must contain **ZERO** game-specific code:

- No references to bolt, breaker, cell, node, bump, flux, or any game vocabulary from `docs/design/terminology/`
- No references to `breaker-game` types, messages, or resources
- No game-specific enums, constants, or configurations
- Only generic 2D spatial/physics/config types and systems

## Naming

- Crate directories: `rantzsoft_<name>/` at workspace root
- Plugin names: `Rantz<Name>Plugin` (e.g., `RantzSpatial2dPlugin`, `RantzPhysics2dPlugin`)
- Root plugins for any `rantzsoft_*` crate follow this convention

## Architecture

- Each `rantzsoft_*` crate is a Bevy plugin that can be used by any 2D game
- Game-specific wiring (e.g., which enum implements `DrawLayer`) lives in `breaker-game`
- Use traits and generics where the game needs to provide types (e.g., `trait DrawLayer`)
- Currently workspace members; will be extracted to separate repos when reuse is needed

## Dependencies

- `rantzsoft_*` crates may depend on each other (e.g., `rantzsoft_physics2d` depends on `rantzsoft_spatial2d`)
- `rantzsoft_*` crates must NEVER depend on `breaker-game` or any `breaker-*` crate
- `breaker-game` depends on `rantzsoft_*` crates, not the other way around
