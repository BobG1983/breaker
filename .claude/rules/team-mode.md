# Team Mode (Experimental)

Delta from the standard one-shot pipeline rules. Read alongside the canonical rules — this file documents only what changes under team mode.

## When team mode is active

- `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` is set in `.claude/settings.json`
- A team config exists at `~/.claude-work/teams/<team-name>/config.json` with a populated `members` array
- The orchestrator is registered as `team-lead` in that config (created by `TeamCreate`)
- Persistent per-agent briefings live at `.claude/teams/briefings/<name>.md` and a config snapshot at `.claude/teams/config.json`

If those conditions don't hold, the standard rules apply unchanged — `delegating-to-subagents.md`, `tdd.md`, `routing-failures.md`, etc. drive one-shot agent launches via `Agent({...})`.

## What stays the same

- **TDD cycle**: test spec → review → writer-tests → review → RED → code spec → review → writer-code → GREEN → REFACTOR. Hard rules from `tdd.md` apply (writer-tests writes only tests; writer-code writes only production code; only runners run cargo).
- **Verification tiers**: Basic / Standard / Full as defined in `verification-tiers.md`. Same agent set, same questions answered.
- **Spec formats**: `spec-format-tests.md` and `spec-format-code.md` are still authoritative for what specs must contain.
- **Game terminology, code standards, architecture, project context**: unchanged.
- **Per-agent definitions** in `.claude/agents/<name>.md`: unchanged. Team members inherit them at spawn.

## What changes

### Agents are persistent, not one-shot

Standard mode: each phase spawns a fresh agent via `Agent({subagent_type, prompt, run_in_background: true})`. The agent reads its briefing from the prompt, does the task, returns results, terminates.

Team mode: the orchestrator runs `TeamCreate` once, then spawns each member ONCE via `Agent({team_name, name, subagent_type, prompt})`. Members stay alive across waves and accumulate context (codebase shape, prior decisions, helper patterns) in their conversation history and stable memory.

Implication: members can carry hard-won knowledge from Wave 1 into Wave 5 without re-discovery.

### Messaging is peer-to-peer, not orchestrator-routed

Standard mode: orchestrator launches agent A → A returns → orchestrator launches agent B with A's output. Every transition flows through the orchestrator.

Team mode: agents message each other directly via `SendMessage(to:"<name>", ...)`. The orchestrator (`team-lead`) only sees milestones (gate results, escalations, circuit breaks). Within a wave, the spec → review revision loop, the writer → reviewer handoff, the runner trigger, and the failure routing all happen peer-to-peer.

The orchestrator's job in team mode: kick off waves, update session-state on milestones, route circuit breaks and ambiguity escalations.

### runner-cargo gates bake in Basic Verification Tier

Standard mode: RED gate = `cargo all-dtest` only. GREEN gate = same. Lint is a separate explicit verify step.

Team mode: every RED and GREEN gate runs `cargo fmt` → `cargo all-dclippy` → `cargo all-dtest` in order. PASS requires all three clean. This means Basic Verification Tier is implicit at every gate and there's no separate "after GREEN, run Basic" step.

### Failure routing skips the orchestrator

Standard mode: runner-cargo reports to orchestrator → orchestrator picks fixer per `routing-failures.md` → orchestrator launches fixer agent.

Team mode: runner-cargo replies directly to the appropriate fixer per its briefing's reply rules:
- RED gate FAIL → reply to `writer-tests` AND `team-lead`
- GREEN gate FAIL → reply to `writer-code` AND `team-lead`
- Other commands → reply to sender AND `team-lead`

`team-lead` is always copied so session-state stays accurate, but the fixer is messaged directly so they can start work without an orchestrator hop.

### Briefings are the source of truth, not the prompt

Standard mode: each agent's prompt IS the briefing — instructions are passed in the `prompt:` parameter and stay in the conversation.

Team mode: each member's prompt is short — it points to a briefing file at `.claude/teams/briefings/<name>.md` and instructs the agent to re-read on any uncertainty or after auto-compaction. The briefing has the trigger dispatch table, peer relationships, escalation rules, and memory paths. Updating a briefing changes future behavior for all currently-running members (they re-read on next uncertainty).

### Recovery protocol replaces compaction hooks

Standard mode: PostCompact hooks in settings.json restore orchestrator state.

Team mode: PostCompact hook is removed (it would misfire in member sessions). Each member's spawn prompt embeds a recovery protocol: if the agent loses certainty about its own name, it sends `SendMessage(to:"team-lead", "Recovery check: please tell me my assigned member name on <team-name> — I lost context.")` and waits for the lead to confirm; then re-reads its briefing from disk.

## Working in team mode — orchestrator checklist

When team mode is active, the orchestrator (you) does:

1. **Kickoffs**: send Wave N kickoff to the appropriate spec writer (or to a wave coordinator if one exists). Every kickoff message MUST include:
   - **Plan file path** (full absolute path)
   - **Plan phase + wave number/name** (e.g., "Phase 1, Wave 2 — builder + terminal integration")
   - **Todo detail file path** (`docs/todos/detail/<name>.md`)
   - **Spec output path** (`.claude/specs/<wave>-<feature>-{tests,code}.md`)
   - **Scope boundaries** — what's in this wave, what's not
   - **Decisions already settled** — anything the orchestrator has resolved upstream
   - **Prior-wave landed artifacts** — types, systems, messages already shipped that this wave builds on
   The same references apply to revision-loop messages — if you skip them on a re-prompt, the agent can't anchor against the canonical plan.
2. **Session-state updates**: after EVERY teammate notification, update `.claude/state/session-state.md` per `session-state.md` rule.
3. **Milestone routing**: when runner-cargo reports gate PASS/FAIL to you, update session-state and either kick off the next phase (PASS) or note the routed failure (FAIL — the runner already messaged the fixer).
4. **Standard tier triggers**: after GREEN PASS, trigger reviewer-completeness, reviewer-correctness, reviewer-quality, reviewer-architecture, reviewer-performance via SendMessage. Collect findings; route triage.
5. **Full tier triggers**: before merge, trigger guards + reviewer-scenarios + reviewer-file-length the same way.
6. **Escalations**: receive circuit-break notifications (3 attempts) and "ambiguity / new design decision" requests; resolve and reply.
7. **Briefing edits**: when you observe routing patterns going wrong, edit the briefings to clarify and message the affected agents to re-read.

## Trial status

Team mode is currently an experiment scoped to the phantom-breaker refactor (see `docs/todos/detail/phantom-breaker.md`). After the trial:
- If it works: this file becomes a permanent rule and the standard-mode rules are updated to acknowledge team-mode as a co-equal pattern.
- If it doesn't: this file and the team setup get removed; standard mode resumes.

Until then, this file is the only authoritative documentation of team-mode behavior.
