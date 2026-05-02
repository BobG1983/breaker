---
name: spawn-team
description: Spawn (or re-spawn) the breaker-team agent team — the persistent set of writer / reviewer / runner / coordinator / guard agents that drive implementation in team mode. Call this before /start-dev or /implement when no team exists, or when briefings/agent definitions have changed and the team needs re-rolling.
---

# Spawn Team

Single entry point for team setup. Validates briefings, deletes the stale team config, calls TeamCreate, fires Agent spawns for every team member, and snapshots the live config back into the project.

Invoking this skill ALWAYS produces a fresh team. Existing in-flight agents are shut down via `shutdown_request` before respawn so their conversation history is wiped — every spawn is clean.

## When to Use

- No team config exists at `~/.claude-work/teams/breaker-team/config.json`
- The team config exists but has zero `members` (stale snapshot from a prior run)
- Briefings, agent definitions, or rules have materially changed and the in-memory team needs to pick them up
- After a coordinator / writer slot count change (e.g., adding `writer-tests-2`, `writer-tests-3`)
- After a `/spawn-team --reset` user request

## When NOT to Use

- When the team is alive and working — do NOT re-roll mid-wave; finish the wave first
- When you only need to update one agent — use a targeted shutdown_request + Agent re-spawn instead

## Source of truth

`.claude/teams/config.json` (project-local, committed) is the **manifest** for the team. It declares every member, its `agentType` (subagent_type), its `model`, its `color`, and its spawn `prompt`. To change the team — add a slot, swap a model, retire an agent — edit the local config and re-run `/spawn-team`.

The runtime config at `~/.claude-work/teams/breaker-team/config.json` (global) is rebuilt from the local manifest every time this skill runs. Don't edit it by hand.

## Procedure

### Step 1 — Validate

Read `.claude/teams/config.json`. For every `members[i]` entry, verify:
- `name`, `agentType`, `model`, `prompt`, `color` are present
- `.claude/agents/<agentType>.md` exists and is non-empty (the role definition)
- `.claude/teams/briefings/<agentType>.md` exists and is non-empty (the persistent instruction surface — slot agents share their `agentType`'s briefing)

Also verify these support files exist:
- `.claude/rules/team-mode.md` (defines team-mode contracts)
- `.claude/rules/plan-format.md` (the wave-coordinator's plan parser contract)
- `.claude/rules/sub-agents.md` (the addressable team list)

If any are missing or empty, abort with the missing file list. Do NOT spawn a team with a malformed manifest.

### Step 2 — Tear down existing team (if any)

If `~/.claude-work/teams/breaker-team/config.json` exists with members:
1. Read each member name from the runtime config.
2. Send `shutdown_request` to each in parallel via `SendMessage`.
3. Wait for `shutdown_response` from each. Members that don't respond within 60s get force-cleared.
4. Delete `~/.claude-work/teams/breaker-team/config.json`.

The local manifest at `.claude/teams/config.json` is NEVER deleted — it's the source you're spawning from.

### Step 3 — Copy local manifest to global

Create the global team directory and copy the local manifest:

```bash
mkdir -p ~/.claude-work/teams/breaker-team
cp .claude/teams/config.json ~/.claude-work/teams/breaker-team/config.json
```

This seeds the global config with the team's declared structure. The runtime will reconcile each member's runtime fields (`agentId`, `joinedAt`, `tmuxPaneId`, `cwd`, `backendType`, `subscriptions`) when the corresponding `Agent()` call fires in Step 5.

### Step 4 — TeamCreate (idempotent)

Call `TeamCreate({ name: "breaker-team" })`. If the team already exists from Step 3's copy, this is a no-op; if the runtime needs additional bookkeeping, this triggers it.

### Step 5 — Spawn every member from the manifest

Iterate `members[]` from `.claude/teams/config.json`. Skip the `team-lead` entry (you ARE the team-lead — there's no separate process to spawn).

**Spawn order matters: `wave-coordinator` MUST be spawned LAST.** Reason: the coordinator dispatches kickoffs to writer/reviewer slot agents and runner-cargo. If it spawns before its targets exist, its first message could land on a non-existent recipient. By spawning last, every recipient is already idle and addressable.

Spawn the rest in batches of 6-8 in parallel (one tool message). Wait for confirmation idle pings before the next batch. The Agent call shape:

```
Agent({
  subagent_type: members[i].agentType,
  team_name: "breaker-team",
  name: members[i].name,
  model: members[i].model,
  prompt: members[i].prompt
})
```

Recommended batching:
1. **Batch A** — runner-cargo + 5 standard reviewers + debugger + 3 guards + 6 researchers (16 spawns, ~2 batches of 8)
2. **Batch B** — slot agents: 4× writer-tests + 4× reviewer-tests + 4× writer-code + 4× planning-writer-specs-tests + 4× planning-reviewer-specs-tests + 4× planning-writer-specs-code + 4× planning-reviewer-specs-code (28 spawns, ~4 batches of 7)
3. **Batch C (last, alone)** — wave-coordinator

After Batch C, the coordinator is alive and idle. It will read its plan-state.md memory if it has one, otherwise wait for `dispatch_first_wave` from team-lead.

### Step 6 — Snapshot back (optional)

After all spawns confirm, optionally copy the global config back to local to capture the runtime fields:

```bash
cp ~/.claude-work/teams/breaker-team/config.json .claude/teams/config.json
```

This commits the resulting agentIds and join timestamps for future reproducibility. Skip this step if you want to keep `.claude/teams/config.json` purely declarative.

### Step 7 — Verify

`grep '"name"' ~/.claude-work/teams/breaker-team/config.json | wc -l` should equal the manifest's `members[]` length + 1 (for the team key). If short, retry the missing spawns.

### Step 8 — Brief team-lead → coordinator

Send a `dispatch_first_wave` message to `wave-coordinator` ONLY when an active todo / plan exists (i.e., when called from `/start-dev` or `/implement`'s precondition). When called bare (`/spawn-team` standalone), leave the team idle.

## Editing the manifest

To add a slot agent: copy an existing slot entry, increment the suffix (`writer-tests-4` → `writer-tests-5`), update the spawn prompt's `name:` line. Re-run `/spawn-team`.

To swap a model: edit `members[i].model`. Re-run `/spawn-team`.

To retire a member: delete the entry. Re-run `/spawn-team` (the old member is shut down in Step 2 and not re-spawned).

To change a briefing without touching the manifest: edit `.claude/teams/briefings/<agentType>.md`. Live agents re-read on uncertainty. For an immediate refresh, send each affected agent a "re-read your briefing" message OR re-run `/spawn-team`.

## Default roster

The default breaker-team roster (declared in `.claude/teams/config.json`):

| Name | subagent_type | Purpose |
|------|--------------|---------|
| `team-lead` | (orchestrator — implicit, you are this role) | Drives skills; routes escalations |
| `wave-coordinator` | `wave-coordinator` | Plan parsing + wave dispatch |
| `planning-writer-specs-tests-1..4` | `planning-writer-specs-tests` | Test spec writer (4 slots) |
| `planning-reviewer-specs-tests-1..4` | `planning-reviewer-specs-tests` | Test spec reviewer (4 slots) |
| `planning-writer-specs-code-1..4` | `planning-writer-specs-code` | Code spec writer (4 slots) |
| `planning-reviewer-specs-code-1..4` | `planning-reviewer-specs-code` | Code spec reviewer (4 slots) |
| `writer-tests-1..4` | `writer-tests` | Failing tests writer (4 slots) |
| `reviewer-tests-1..4` | `reviewer-tests` | Test reviewer (4 slots) |
| `writer-code-1..4` | `writer-code` | Production code writer (4 slots) |
| `runner-cargo` | `runner-cargo` | All cargo execution |
| `reviewer-completeness` | `reviewer-completeness` | Standard tier — completeness |
| `reviewer-correctness` | `reviewer-correctness` | Standard tier — correctness |
| `reviewer-quality` | `reviewer-quality` | Standard tier — quality |
| `reviewer-architecture` | `reviewer-architecture` | Standard tier — architecture |
| `reviewer-performance` | `reviewer-performance` | Standard tier — performance |
| `debugger` | `debugger` | DEBUG protocol on /investigate |
| `researcher-bevy-api` | `researcher-bevy-api` | Bevy 0.18 API lookups (on demand) |
| `researcher-codebase` | `researcher-codebase` | End-to-end data flow (on demand) |
| `researcher-impact` | `researcher-impact` | Reference graph (on demand) |
| `researcher-system-dependencies` | `researcher-system-dependencies` | Query / message conflict map (on demand) |
| `researcher-rust` | `researcher-rust` | Rust idioms + compiler errors (on demand) |
| `researcher-crates` | `researcher-crates` | Crate evaluation (on demand) |
| `guard-game-design` | `guard-game-design` | Design-pillar consult (on demand) |
| `guard-docs` | `guard-docs` | Doc drift consult (on demand) |
| `guard-security` | `guard-security` | Security consult (on demand) |

Slot agents inherit their model + tools from `.claude/agents/<subagent_type>.md`. Slot suffix (`-1`, `-2`, `-3`) is registered as the `name`; the `subagent_type` stays the base name.

NOT included by default (kept as one-shot Agent calls when needed): `reviewer-file-length`, `reviewer-scenarios`, `writer-scenarios`, `runner-release`, `guard-dependencies`, `guard-agent-memory`. These run rarely and don't benefit from persistent context.

### Step 4 — TeamCreate

Call `TeamCreate({ name: "breaker-team" })`. This writes the new `~/.claude-work/teams/breaker-team/config.json` skeleton.

### Step 5 — Spawn each member

For every member in the roster, fire one `Agent` call:

```
Agent({
  subagent_type: <subagent_type>,
  team_name: "breaker-team",
  name: <name (with slot suffix if applicable)>,
  prompt: "You are `<name>` on `breaker-team`. Re-read your briefing at `.claude/teams/briefings/<subagent_type>.md`. Your slot suffix (if any) is in your name. Then idle until messaged. Do NOT initiate work."
})
```

Spawn in batches of 6-8 in parallel (one tool message). Wait for confirmation idle pings before the next batch — order doesn't matter, but you want to know all spawns succeeded before snapshotting.

### Step 6 — Snapshot

Copy `~/.claude-work/teams/breaker-team/config.json` → `.claude/teams/config.json` (project-local committed snapshot).

### Step 7 — Verify

`grep '"name"' .claude/teams/config.json | wc -l` should equal the roster count + 1 (for `breaker-team` itself). If short, retry the missing spawns.

### Step 8 — Brief team-lead → coordinator

Send a `dispatch_first_wave` message to `wave-coordinator` ONLY when an active todo / plan exists (i.e., when called from `/start-dev` or `/implement`'s precondition). When called bare (`/spawn-team` standalone), leave the team idle.

## Slot count override

Default: 3 slots per slottable role. Override per invocation:
- `/spawn-team --slots writer-code:5` — five writer-code slots
- `/spawn-team --slots writer-tests:1,writer-code:2` — fewer slots for projects with no parallelism opportunity

The override only changes the spawn count in Step 5; the agent definitions and briefings handle any slot count.

## Failure modes

- Briefing missing → abort, list missing files, no spawns
- Agent def missing → abort, list missing defs, no spawns
- TeamCreate fails → abort, surface the error to user
- Some spawns succeed, some fail → retry failed spawns; if still failing, abort and force-shutdown the partial team
- shutdown_request times out for old members → log force-clear and proceed (this is the `/spawn-team --reset` escape hatch path)
