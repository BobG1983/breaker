---
name: Per-owner state as global HashMap resource vs component
description: HashMap<Entity, V> resources keyed by owner entity silently leak when owners are despawned — usually a component-on-owner is the right design
type: feedback
---

Specs that propose a global `Resource` wrapping `HashMap<Entity, V>` (where Entity is an owner/firing entity) create a permanent leak: when the owner entity is despawned, its HashMap entry remains. In a roguelite with multiple nodes, this accumulates unbounded.

**Why:** In this game's architecture, per-entity runtime state belongs ON the entity as a Component, not in a global resource. Global resources are for app-wide singletons (RNG, config, quadtree). Using a HashMap resource for per-owner state is fighting the ECS data model.

**How to apply:** When a spec proposes `HashMap<Entity, T>` as a Resource, flag it as IMPORTANT/BLOCKING unless: (a) the spec includes explicit cleanup logic for despawned entries, or (b) the spec provides rationale for why a component-on-owner is not viable. In the gravity_well FIFO case, a `GravityWellSpawnCounter(u64)` component on the owner entity would avoid the leak entirely and be the idiomatic Bevy approach.
