# Team Mode

The standard agent-team operating mode for this project. Read alongside the canonical rules — this file documents only what differs from one-shot standard mode and how the wave-coordinator + slot agents interact.

## When team mode is active

- `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` is set in `.claude/settings.json`
- A team config exists at `~/.claude-work/teams/breaker-team/config.json` with a populated `members` array (snapshot at `.claude/teams/config.json`)
- The orchestrator is registered as `team-lead` in that config
- A `wave-coordinator` member exists in the config — this is the dispatcher
- Persistent per-agent briefings live at `.claude/teams/briefings/<name>.md`

If those conditions don't hold, the standard rules apply unchanged. To get into team mode, run `/spawn-team`.

## What stays the same

- **TDD cycle**: test spec → review → writer-tests → review → RED → code spec → review → writer-code → GREEN → REFACTOR. Hard rules from `tdd.md` apply (writer-tests writes only tests; writer-code writes only production code; only runner-cargo runs cargo).
- **Verification tiers**: Basic / Standard / Full as defined in `verification-tiers.md`.
- **Spec formats**: `spec-format-tests.md` and `spec-format-code.md` are still authoritative for what specs must contain.
- **Game terminology, code standards, architecture, project context**: unchanged.
- **Per-agent definitions** in `.claude/agents/<name>.md`: unchanged. Slot agents inherit from their `subagent_type` def.

## What changes

### Agents are persistent, not one-shot

Standard mode: each phase spawns a fresh agent via `Agent({...})`. The agent reads its briefing from the prompt, does the task, returns results, terminates.

Team mode: `/spawn-team` runs `TeamCreate` and spawns each member ONCE via `Agent({team_name, name, subagent_type, prompt})`. Members stay alive across waves and accumulate context (codebase shape, prior decisions, helper patterns) in their conversation history and stable memory.

### Slot agents

Roles that may need to run in parallel across sub-waves are spawned as **slots**: `writer-tests-1`, `writer-tests-2`, `writer-tests-3` (and the same `-1/-2/-3` pattern for `writer-code`, `planning-writer-specs-tests`, `planning-writer-specs-code`, `planning-reviewer-specs-tests`, `planning-reviewer-specs-code`, `reviewer-tests`).

Slot count default: 3 per slottable role. Override at spawn via `/spawn-team --slots writer-code:5`. Each slot inherits from the same `subagent_type` agent definition; the slot suffix is in the `name` only. Slot 1 vs slot 2 are interchangeable — pick whichever is `idle` when dispatching.

Non-slottable roles (one each): `team-lead`, `wave-coordinator`, `runner-cargo`, the Standard tier reviewers (completeness / correctness / quality / architecture / performance), researchers, guards, debugger.

### Wave dispatch lives in wave-coordinator, not orchestrator

Standard mode: orchestrator drives every kickoff per wave.

Team mode: `wave-coordinator` reads the plan once at spawn, persists the parsed wave graph to `.claude/agent-memory/wave-coordinator/plan-state.md`, and dispatches kickoffs to writer slots based on milestone events. The orchestrator (`team-lead`) only sees:
- The initial `dispatch_first_wave` request from team-lead → coordinator (after the user starts implementation)
- Standard tier triggers (orchestrator launches Standard tier reviewers after coordinator reports `green_gate_pass`)
- Full tier triggers (before merge)
- Escalations: circuit-breaks, contradictions, missing tools, plan wrongness, invalidated speculation
- Plan-revision authorizations

The coordinator's full state machine and trigger table are in `.claude/teams/briefings/wave-coordinator.md` and `.claude/agents/wave-coordinator.md`.

### Parallel sub-waves

When a plan declares parallel sub-waves (e.g., Wave 4 with `parallel: [4A, 4B, 4C]` per `plan-format.md`), the coordinator dispatches all sub-waves to different slot agents in one turn. Each sub-wave's pipeline (test-spec → review → writer-tests → review → RED → ...) runs independently. RED and GREEN gates BATCH across sub-waves: a single runner-cargo run after ALL sub-waves are ready, not one per sub-wave.

Sub-wave letter (`4A`, `4B`, `4C`) appears in:
- Spec file paths (`.claude/specs/wave4a-<feature>-tests.md`)
- Kickoff messages (in the wave header line)
- Reply summaries to team-lead (so session-state rows update on the right sub-wave)

### Speculative spec drafting (N+1)

When a wave reaches `green-gate` (writer-code dispatched, GREEN pending) and the plan permits speculation (`no_speculation: false` on the next wave), the coordinator MAY dispatch the next wave's test spec to a free `planning-writer-specs-tests-<slot>` with a `.draft.md` suffix on the spec file path.

If GREEN passes: the speculative draft is "promoted" — the writer drops the suffix and the reviewer is notified. If GREEN fails AND the failure matches an `abandonment_trigger` from the next wave's plan header: the coordinator messages the speculative writer with `abandon_draft`, the wave returns to `pending`, and team-lead is notified.

Speculation is opt-out at the plan level. Add `no_speculation: true` to a wave that should never be drafted speculatively (high abandonment risk or expensive discard).

### Messaging is peer-to-peer, not orchestrator-routed

Within a wave: agents message each other directly via `SendMessage`. The orchestrator only sees milestones the coordinator surfaces.

| Phase transition | Who messages whom |
|---|---|
| test spec ready | planning-writer-specs-tests-<slot> → planning-reviewer-specs-tests-<slot> |
| test spec approved | planning-reviewer-specs-tests-<slot> → wave-coordinator |
| test spec needs revision | planning-reviewer-specs-tests-<slot> → planning-writer-specs-tests-<slot> |
| writer-tests done | writer-tests-<slot> → reviewer-tests-<slot> |
| tests reviewed | reviewer-tests-<slot> → wave-coordinator |
| RED gate request | wave-coordinator → runner-cargo |
| RED gate result | runner-cargo → fixer (writer-tests-<slot>) AND wave-coordinator AND team-lead |
| code spec ready | planning-writer-specs-code-<slot> → planning-reviewer-specs-code-<slot> |
| code spec approved | planning-reviewer-specs-code-<slot> → wave-coordinator |
| writer-code done | writer-code-<slot> → wave-coordinator (which then triggers runner-cargo) |
| GREEN gate result | runner-cargo → fixer (writer-code-<slot>) AND wave-coordinator AND team-lead |
| Standard tier triggered | team-lead → 5 reviewers (one message each) |
| Full tier triggered | team-lead → guards + reviewer-scenarios + reviewer-file-length |

### runner-cargo gates bake in Basic Verification Tier

Standard mode: RED gate = `cargo all-dtest` only. GREEN gate = same. Lint is a separate explicit verify step.

Team mode: every RED and GREEN gate runs `cargo fmt` → `cargo all-dclippy` → `cargo all-dtest` in order. PASS requires all three clean. This means Basic Verification Tier is implicit at every gate and there's no separate "after GREEN, run Basic" step.

### Failure routing skips the orchestrator

Per `routing-failures.md` (team-mode addendum): runner-cargo replies directly to the appropriate fixer per its briefing's reply rules:
- RED gate FAIL → reply to `writer-tests-<slot>` AND `wave-coordinator` AND `team-lead`
- GREEN gate FAIL → reply to `writer-code-<slot>` AND `wave-coordinator` AND `team-lead`
- Other commands → reply to sender AND `wave-coordinator` AND `team-lead`

### Briefings are the source of truth, not the prompt

Each member's spawn prompt is short — it points to a briefing file at `.claude/teams/briefings/<subagent_type>.md` and instructs the agent to re-read on uncertainty or auto-compaction. The briefing has the trigger dispatch table, peer relationships, escalation rules, and memory paths. Updating a briefing changes future behavior for all currently-running members (they re-read on next uncertainty).

### Recovery protocol replaces compaction hooks

Each member's spawn prompt embeds: if the agent loses certainty about its own name, it sends `SendMessage(to:"team-lead", "Recovery check: please tell me my assigned member name on breaker-team — I lost context.")` and waits for the lead to confirm; then re-reads its briefing from disk.

## Working in team mode — orchestrator (team-lead) checklist

The orchestrator does NOT dispatch waves anymore. The coordinator does. The orchestrator's responsibilities reduce to:

1. **Initial start signal**: when `/start-dev` or `/implement` confirms the plan + branch are ready, send `dispatch_first_wave` to `wave-coordinator` with the plan path and todo detail path.
2. **Session-state updates**: after EVERY teammate notification, update `.claude/state/session-state.md` per `session-state.md` rule. The coordinator surfaces wave-level milestones; the orchestrator translates those into the session-state schema.
3. **Standard tier triggers**: after coordinator reports `green_gate_pass`, trigger the 5 Standard tier reviewers via SendMessage in parallel. Collect findings; route triage. Reply `standard_tier_pass` to coordinator on clean.
4. **Full tier triggers**: before merge (`/finish-dev`), trigger guards + reviewer-scenarios + reviewer-file-length.
5. **Escalations**: handle circuit-breaks, contradictions, missing tools, plan wrongness, invalidated speculation. The coordinator escalates; the orchestrator decides.
6. **Plan revisions**: when a finding requires changing the plan, edit the plan file and send `plan_revision` to the coordinator with the changed wave list.
7. **Briefing / rule edits**: when you observe routing patterns going wrong, edit the briefings/rules and either send the affected agents a "re-read your briefing" message OR run `/spawn-team` to re-roll cleanly.
8. **Recovery responses**: when an agent sends a "what's my name" recovery check, reply with their full name (slot suffix included).

## Slot lifecycle

When a slot agent is `idle`, the coordinator can dispatch new work to it. When `working`, the coordinator queues additional kickoffs (logged in `plan-state.md`) and notifies team-lead.

If all slots for a role are saturated and a sub-wave needs dispatch, the coordinator pauses that sub-wave's branch and notifies team-lead — increasing slot count is a `/spawn-team --slots <role>:N` operation, not an in-flight change.

## Trial status

Team mode is the project's standard operating mode. Standard one-shot mode remains as a fallback for when the team cannot be spawned (e.g., on a fresh clone before `/spawn-team` runs).
