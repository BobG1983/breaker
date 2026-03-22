---
name: pattern_onexit_stale_resource
description: Systems on OnExit(StateA) that read resources set during StateB (the NEXT state) will read stale values
type: feedback
---

Systems scheduled on `OnExit(StateA)` run BEFORE the player interacts with StateB. If the player sets a resource value during StateB (e.g., entering a seed on RunSetup), systems that read that resource on `OnExit(StateA)` will see the PREVIOUS value.

**Why:** OnExit fires on the transition OUT of a state, which happens on the frame the state changes. The next state's logic hasn't run yet. Found during 4i spec review — `reset_run_state` on `OnExit(MainMenu)` captures `RunSeed` before the user enters it on `RunSetup`.

**How to apply:** When a spec proposes capturing or reading a resource in an OnExit/OnEnter system, verify that the resource has been set by the time that system runs. Check the state machine flow: which state sets the resource, and which state transition triggers the reader?
