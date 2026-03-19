---
name: Query type aliases live in their domain
description: Query type aliases belong in domain/queries.rs, not inline in system files
type: feedback
---

Query type aliases must live in the owning domain's `queries.rs` file (e.g. Cell queries in `src/cells/queries.rs`, Bolt queries in `src/physics/queries.rs`).

**Why:** Keeps domain boundaries clean — a domain's query types are part of its public interface, not implementation details of individual systems.

**How to apply:** When creating a `type FooQuery = (...)` alias to satisfy clippy's `type_complexity` lint, place it in `<domain>/queries.rs` and import it from there. Never define query type aliases inline in system files.
