# Brickbreaker Roguelite

Roguelite Arkanoid clone in Bevy 0.18 (Rust). See `docs/DESIGN.md` for design pillars, `docs/ARCHITECTURE.md` for technical decisions + code standards + testing approach, `docs/PLAN.md` for build roadmap, `docs/TERMINOLOGY.md` for game vocabulary.

## Build & Run

```
cargo dev                    # Dev build + run (dynamic linking)
cargo dtest                  # Run all tests (dynamic linking)
cargo dcheck                 # Type check (dynamic linking)
cargo dclippy                # Lint (dynamic linking)
cargo dbuild                 # Dev build without running (dynamic linking)
cargo run --release          # Release build
cargo fmt --check            # Format check
```

Dev builds use `.cargo/config.toml` aliases with `bevy/dynamic_linking` for fast compiles.

## Architecture

**Plugin-per-domain** with message-driven decoupling. Each domain plugin (breaker, bolt, cells, upgrades, run, physics, audio, ui, debug) owns its components, resources, and systems. Domains communicate only through Bevy 0.18 messages. See `docs/ARCHITECTURE.md` for full details, file tree, message table, and patterns.

## Terminology

All code identifiers MUST use game vocabulary (Breaker, Bolt, Cell, Node, Amp, Augment, Overclock, Bump, Flux). No generic terms. See `docs/TERMINOLOGY.md`.

## Decision Making

**ALWAYS ask before**:
- Creating new plugins, systems, or modules not in the architecture
- Choosing between component vs resource vs message for new data
- Any design decision not covered in `docs/PLAN.md`
- Architectural changes or refactors affecting multiple systems

**ALWAYS do**:
- Write tests FIRST for new game logic (see `docs/ARCHITECTURE.md` Testing — TDD)
- Create a feature branch before starting work (`feature/*`, `fix/*`, `refactor/*` off main)
- Commit with conventional commits (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`) after tests pass
- After merging a branch to main, delete it locally and from the remote

**Move freely on**:
- Implementation within existing system boundaries
- Adding tests
- Bug fixes with obvious solutions
- Updating RON data files

## Design Rules

See `docs/DESIGN.md` for the full set of non-negotiable design pillars. The key mechanical rules are in `docs/ARCHITECTURE.md` (bolt reflection, breaker state machine, bump grades).

## Agent Workflow

The main agent is the orchestrator. Invoke subagents automatically at these trigger points — do not wait to be asked.

| Trigger | Agent | Why |
|---------|-------|-----|
| Unfamiliar Bevy 0.18 API or pattern | **bevy-api-expert** | Verify before using — Bevy APIs change between versions |
| New system, plugin, or module added | **architecture-guard** | Validate plugin boundaries and message discipline |
| New gameplay mechanic or upgrade designed | **game-design-guard** | Validate against design pillars |
| Compiler errors that aren't obvious | **rust-error-decoder** | Translate diagnostics into actionable fixes |
| 3+ systems added to a plugin, or cross-plugin data flow | **system-dependency-mapper** | Detect ordering issues and conflicts |
| Feature complete, ready to commit | **test-runner** | Full validation suite (fmt, clippy, tests) |

**Post-implementation checklist** (run before considering a task done):
1. Run **test-runner**
2. Run `/simplify` on changed code
3. If new systems or plugins were added → run **architecture-guard**
4. If new gameplay mechanics were added → run **game-design-guard**
5. Commit to the feature branch with a conventional commit message
