# Tiers & Progression

## Tier Structure
A **tier** = 4 non-boss nodes followed by a boss node (5 nodes total per tier). Tiers start at 1. Committed to **8 tiers** for "run complete", with infinite continuation past tier 8.

## Node Type Progression
Node types escalate across tiers with a gradual 1-slot-per-tier ramp. Active nodes appear from tier 1 (no all-passive tier — starts warmer):

```
Tier 0: BBBB + Boss  (basic / tutorial — pre-scripted)
Tier 1: PPPA + Boss  (1 active from the start)
Tier 2: PPAA + Boss  (2 active)
Tier 3: PAAA + Boss  (3 active)
Tier 4: AAAA + Boss  (all active)
Tier 5: AAAV + Boss  (1 volatile mixed in)
Tier 6: AAVV + Boss  (2 volatile)
Tier 7: AVVV + Boss  (3 volatile)
Tier 8: VVVV + Boss  (all volatile — "run complete")
Tier 9+: VVVV + Boss (volatile — cell escalation + hazards ramp in)
```

**Volatile** = the escalation beyond active. Implies unpredictability and danger.

No post-volatile node type — three escalation axes (volatile nodes, cell escalation, hazards) are sufficient for infinite scaling.

## Tier 0 (Tutorial/Easy)
- Pre-scripted, super simple "basic" nodes
- Accessible via a special one-time protocol that moves you back a tier from tier 1
- Uses hardcoded simple layouts, not procedurally generated

## Difficulty Scaling
Within a tier, node design is modified based on tier difficulty. Higher tiers make "easier" node types harder by **cell type escalation** — swapping basic cells for tougher types.

## Infinite Scaling (Tier 9+)
Three mechanisms stack for infinite difficulty:
1. **Cell type escalation** — tougher cell types, more portal cells
2. **Hazards** — player picks from 3 random hazards per tier. Hazards can stack. See [Protocol & Hazard system design](../mod-system-design.md).
3. **Block tier escalation** — higher-tier blocks with harder compositions become available

## Tier Regression Protocol
A protocol that "moves you back a tier" — drops difficulty by 1, gives the player another tier of levels to earn rewards. The tier 0 variant can only appear once.

## Needs Detail
- Data structures for tier state (current tier, node index within tier, hazard stack)
- API for tier generation (when does a tier get generated? what triggers it?)
- How does the tier system integrate with existing run state?
- Tier regression mechanics — how does going back a tier interact with hazard stack?
