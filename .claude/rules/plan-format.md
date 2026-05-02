# Plan Format

The contract that `wave-coordinator` parses to drive wave-by-wave dispatch. Plans live at `.claude/plans/<name>.md` (project-local, committed) per the `plansDirectory` setting in `.claude/settings.json` and are written by humans (or by `Plan` agent in plan mode).

A plan is a Markdown file. Each wave is introduced by a heading `### Wave N` (or `### Wave NX` for sub-waves like `4A`). Each wave heading is **immediately followed by a fenced YAML block** containing the wave's machine-readable header. Free-form prose follows.

## Required YAML fields

```markdown
### Wave 4 — Bolt collision gating + lifespan dispatch + afterimage migration

```yaml
id: 4
parallel: [4A, 4B, 4C]
blocks: [2]
no_speculation: false
abandonment_triggers:
  - "wave 2 GREEN fails on builder API surface"
  - "wave 2 GREEN fails on PhantomBreaker marker location"
scope:
  - "bolt collision Without<PhantomBreaker> gating (4A)"
  - "tick_phantom_breaker_lifespan + DespawnEntity dispatch (4B)"
  - "spawn_phantom_breaker.rs raw spawn → builder migration (4C)"
decisions:
  - "DespawnEntity confirmed live in rantzsoft_dmg crate"
  - "BoltImpactBreaker already has breaker: Entity field"
```

[free-form prose with file lists, test pointers, etc.]
```

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `id` | string or int | yes | Wave identifier matching the heading. For sub-wave headings (`### Wave 4A`), the id is the full sub-wave id (`4A`). |
| `parallel` | list of sub-wave ids | only on parent waves with sub-waves | Declares that the listed sub-waves run as a parallel batch. The parent wave header gets the `parallel:` field; each sub-wave gets its own `### Wave NX` heading + YAML block. |
| `blocks` | list of wave ids | yes | Waves that must reach `committed` before this wave can dispatch. Empty list = no dependencies. |
| `no_speculation` | bool | optional (default false) | If true, this wave may NOT be drafted speculatively even when predecessors are in `green-gate`. Use for waves with high abandonment risk or extreme cost to discard. |
| `abandonment_triggers` | list of strings | optional | Plain-English descriptions of GREEN-gate failures that invalidate speculative drafts of this wave. Coordinator checks these against `runner-cargo`'s failure summary to decide whether to send `abandon_draft`. |
| `scope` | list of strings | yes | High-level deliverables. Each item is a one-line description of what this wave produces. Used by the coordinator to populate kickoff messages. |
| `decisions` | list of strings | optional | Pre-resolved design decisions the writers should treat as settled. |

## Sub-wave headings

When a wave declares `parallel: [4A, 4B, 4C]`, each sub-wave gets its own heading and its own YAML block:

```markdown
### Wave 4A — Bolt collision gating

```yaml
id: 4A
blocks: [2]
scope:
  - "bolt_breaker_collision: secondary phantom_query for inline gating"
decisions:
  - "BoltImpactBreaker.breaker == entity is the join key"
```

[prose]

### Wave 4B — Lifespan dispatch
[YAML + prose...]

### Wave 4C — Afterimage migration
[YAML + prose...]
```

Sub-wave YAML blocks omit `parallel:` (they are leaves). Their `blocks:` list may include the parent wave's predecessors (e.g., `[2]`) or other sub-waves if there are intra-batch dependencies (e.g., 4C depends on 4A).

## Reading order

The coordinator reads the plan file once at spawn:
1. Walk all `### Wave` headings in order.
2. Parse each immediately-following ```yaml block.
3. Build a dependency graph from `blocks:` fields.
4. Detect parallel batches from `parallel:` fields on parent waves.
5. Persist parsed state to `.claude/agent-memory/wave-coordinator/plan-state.md`.

If parsing fails (missing YAML, malformed dependency graph, unknown wave id in `blocks:`), the coordinator escalates to team-lead with the parse error and waits.

## Backward compatibility

Plans without YAML wave headers are NOT parseable by the coordinator. The coordinator escalates "plan lacks parseable wave headers — please ask the user to add them or run with manual orchestrator dispatch." The team-lead can either patch the plan or fall back to the legacy orchestrator-dispatch flow (no parallel sub-waves, no speculation).

## Examples

See `.claude/plans/cosmic-yawning-porcupine.md` (the phantom-breaker plan) for a worked example after the format is adopted.
