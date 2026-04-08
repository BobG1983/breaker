---
name: Time<Virtual> pausing hazard — Out transitions and FixedUpdate dependency
description: Out transitions leave Time<Virtual> paused; FixedUpdate-dependent systems (check_spawn_complete, all_animate_in_complete) hang if reached while paused
type: project
---

`TransitionType::Out` sets `unpause_at_end: false` in `begin_transition` (rantzsoft_stateflow). Every `FadeOut`-backed state transition pauses `Time<Virtual>` and NEVER unpauses it.

Only `TransitionType::In` and `TransitionType::OutIn` unpauses (`unpause_at_end: true`).

Affected routes (as of feature/scenario-runner-wiring):
- `GameState::Loading → Menu`: FadeIn (`In` type) — DOES unpause. Safe.
- `MenuState::Main → StartGame`: FadeOut (`Out` type) — does NOT unpause.
- `GameState::Menu → Run` (via Teardown chain): FadeOut — does NOT unpause.
- `GameState::Run → Menu`: FadeOut — does NOT unpause.

After `GameState::Run → Menu` FadeOut completes, `Time<Virtual>` is PAUSED and there is no system to unpause it before the next run. When the next run starts:
- `RunState::Setup → Node` FadeIn is `TransitionType::In` — applies state change NOW, `NodeState::Loading` entered while `Time<Virtual>` is paused
- Spawn signals sent in `OnEnter(NodeState::Loading)` expire (~2 Update frames)
- `check_spawn_complete` (FixedUpdate, no state guard) never reads them
- `NodeState::Loading → AnimateIn` never fires
- Game stuck with blank screen

**Why:** The AnimateIn route change (`AnimateIn → Playing` from `.when(|_| true)` to message-triggered) makes this worse: with the old pass-through, even if `AnimateIn` were somehow reached, `dispatch_condition_routes` (Update, unaffected by `Time<Virtual>`) would immediately exit it. With message-triggered, `all_animate_in_complete` requires FixedUpdate.

**How to apply:** When analyzing any hang or stuck-state bug involving runs or menu returns, check whether `Time<Virtual>` is paused. The `Out`-type transitions are the culprits. The fix is to unpause on `OnEnter(GameState::Menu)` or `OnEnter(MenuState::Main)`, or to change the `Run → Menu` route to `OutIn` type.
