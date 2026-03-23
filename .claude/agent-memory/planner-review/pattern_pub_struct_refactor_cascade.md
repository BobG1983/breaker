---
name: Public struct field type changes cascade across crates
description: Changing a pub struct's inner type (e.g., Vec<A> to Vec<B>) breaks all construction and access sites, including in other workspace crates like breaker-scenario-runner
type: feedback
---

When a spec proposes changing the inner type of a `pub` struct (e.g., `ChipOffers(pub Vec<ChipDefinition>)` to `ChipOffers(pub Vec<ChipOffering>)`), the breakage cascades to:
1. All production code accessing the struct's fields (`.0`, pattern matching)
2. All test code constructing the struct (`ChipOffers(vec![...])`)
3. All code in OTHER workspace crates that import the struct via `pub use`

**Why:** The `ChipOffers` refactor in phase 4h affected 15+ sites across breaker-game and breaker-scenario-runner. Writer-code agents scoped to a single domain cannot modify files in other crates. The main agent must handle cross-crate cascade.

**How to apply:** When reviewing a spec that changes a `pub` struct's field types:
1. Grep for the struct name across the ENTIRE workspace (not just the domain)
2. Count construction sites (`StructName(vec![`, `StructName { field:`) and access sites (`.0`, `.field`)
3. Separate sites into: (a) same-domain (writer handles), (b) other-domain same-crate (main agent handles), (c) other-crate (main agent handles, often needs its own step)
4. If >10 total cascade sites, recommend splitting into: additive prep commit (new type + adapter methods) then breaking change commit
