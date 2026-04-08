---
name: cells/builder dead_code warnings — builder API not fully consumed
description: CellBuilder typestate API has unused methods/types because spawn_cells_from_grid only uses a subset of the builder's capabilities
type: project
---

As of 2026-04-08, the `cells/builder/` typestate API (CellBuilder<P,D,H,V>) has persistent `dead_code` warnings because `spawn_cells_from_grid` only calls the methods it needs right now. The full API surface is scaffolded for future cell types.

## Affected symbols

**terminal.rs:**
- `CellBuilder<HasPosition, HasDimensions, HasHealth, Headless>::build` and `spawn` — Headless variant unused
- `CellBuilder<HasPosition, HasDimensions, HasHealth, Rendered>::build` — terminal `build` for Rendered variant unused (only `spawn` path is called)

**transitions.rs:**
- `CellBuilder::hp` — unused (hp set via `override_hp` or default path)
- `CellBuilder::headless` — Headless visual state unused
- `CellBuilder::required_to_clear`, `damage_visuals`, `with_effects`, `with_behavior`, `with_behaviors`, `color_rgb` — optional builder methods not yet consumed by any call site

**types.rs:**
- `Headless` struct — visual state marker, not yet used

## Status

These are **warnings only**, not errors. They will clear as more cell types and builder usages are added. Do NOT suppress with `#[allow(dead_code)]` — the lint config is intentional.

The question "are the dead_code warnings in cells/builder/ gone now that spawn_cells_from_grid calls the builder?" was asked on 2026-04-08. Answer: NO — many builder methods are still unused. `spawn_cells_from_grid` calls the builder but only exercises a subset of its API.
