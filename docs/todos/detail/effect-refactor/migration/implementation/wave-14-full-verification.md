# Wave 14: Full Verification Tier

## Goal
Pass the Full Verification Tier (pre-merge gate).

## Agents (all in parallel, in addition to Standard Tier)
- **runner-scenarios** — automated gameplay testing under chaos input
- **guard-security** — unsafe blocks, deserialization, supply chain risks
- **guard-docs** — documentation drift from code
- **guard-game-design** — mechanic changes against design pillars
- **guard-dependencies** — unused/outdated/duplicate deps, license compliance
- **guard-agent-memory** — stale/duplicated memories
- **reviewer-file-length** — finds oversized files, produces split specs
- **reviewer-scenarios** — scenario coverage gaps

## Process
1. Run all agents in parallel
2. Route failures per `routing-failures.md`
3. Fix → Basic → Standard → Full → repeat until clean
4. ALL findings must be fixed (not deferred) per `verification-tiers.md`
