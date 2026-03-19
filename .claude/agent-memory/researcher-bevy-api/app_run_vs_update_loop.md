---
name: App::run() vs manual update loop — WinitPlugin event loop
description: Verified source-level analysis of App::run() behavior, the self-replacement trick, WinitPlugin runner mechanics, and why manual update() cannot drive a windowed app
type: reference
---

# App::run() vs Manual Update Loop in Bevy 0.18.1

Verified from raw source at `github.com/bevyengine/bevy/tree/v0.18.0`.

## App::run() — exact source

```rust
// crates/bevy_app/src/app.rs
pub fn run(&mut self) -> AppExit {
    if self.is_building_plugins() {
        panic!("App::run() was called while a plugin was building.");
    }
    let runner = core::mem::replace(&mut self.runner, Box::new(run_once));
    let app = core::mem::replace(self, App::empty());
    (runner)(app)
}
```

**Critical**: `run()` does `mem::replace(self, App::empty())` — the original App is MOVED
into the runner function. The caller's `app` variable is now an `App::empty()` shell after
`run()` returns. The world is GONE from the caller's perspective.

RunnerFn type: `Box<dyn FnOnce(App) -> AppExit>` — takes App by value (owned).

## The default headless runner (run_once)

```rust
fn run_once(mut app: App) -> AppExit {
    while app.plugins_state() == PluginsState::Adding {
        bevy_tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
    app.update();
    app.should_exit().unwrap_or(AppExit::Success)
}
```

Used by MinimalPlugins/ScheduleRunnerPlugin in run-once mode.

## The WinitPlugin runner (winit_runner)

```rust
// crates/bevy_winit/src/state.rs
pub fn winit_runner(mut app: App, event_loop: EventLoop<WinitUserEvent>) -> AppExit {
    if app.plugins_state() == PluginsState::Ready {
        app.finish();
        app.cleanup();
    }
    let runner_state = WinitAppRunnerState::new(app);  // App moved inside runner_state

    // Non-WASM: blocks until the OS event loop exits
    if let Err(err) = event_loop.run_app(&mut runner_state) { ... }

    runner_state.app_exit.unwrap_or_else(|| AppExit::error())
}
```

The App is moved into `WinitAppRunnerState` which implements winit's `ApplicationHandler` trait.
The OS calls `run_app()` methods (like `about_to_wait`, `new_events`, `window_event`) —
these are the ONLY path by which winit events get into Bevy's world.

`app.update()` is called from inside `WinitAppRunnerState::run_app_update()`, which is
triggered by winit callbacks (specifically `redraw_requested` / `about_to_wait`).

## Why manual `finish(); cleanup(); loop { update(); }` CANNOT drive a windowed app

A manual update loop bypasses the entire winit `ApplicationHandler` machinery:

1. **No OS event pump**: Winit window events (mouse, keyboard, resize, close) are delivered ONLY
   through the `ApplicationHandler` trait methods triggered by `EventLoop::run_app()`. A raw
   `app.update()` loop never calls these methods — the OS event queue builds up indefinitely.

2. **No window creation**: WinitPlugin creates windows in response to winit events. Without
   `EventLoop::run_app()`, the window is never created and `WindowPlugin` systems never fire.

3. **No render frame**: The render sub-app's extract/render loop is triggered from within
   `run_app_update()` inside `WinitAppRunnerState`. A manual `app.update()` does run the
   main schedule, but the render pipeline won't receive OS redraw requests.

4. **Blocked on non-WASM**: `EventLoop::run_app()` is a BLOCKING call that owns the thread
   until the OS event loop exits. You cannot call it and then call `app.update()` yourself —
   the loop IS the frame driver.

## What happens after `app.run()` returns

After `app.run()` returns, `self` has been replaced with `App::empty()` — an app with no
world, no plugins, no schedules. Any attempt to call `app.world()` or `app.update()` on the
original `app` variable after `run()` returns will operate on this empty app, not the
real world. The world was moved into the runner closure and is destroyed when the runner returns.

There is NO supported way to "retain world access after run() exits" for windowed apps.

## Pattern: accessing world state before the window opens

If you need world access for initialization/inspection, do it BEFORE calling `app.run()`:
- Use `app.world_mut()` directly before `run()`
- Use startup systems (OnEnter, Startup schedule)
- Use a custom runner via `app.set_runner()`

## Pattern: using a custom runner to retain the App

If you write a custom runner that receives `App` by value, you OWN the app for the duration.
You can call `app.finish(); app.cleanup(); app.update(); ...` as you wish and then return.
But you cannot integrate this with WinitPlugin's OS event handling.

## Conclusion for windowed (DefaultPlugins) apps

**You MUST use `app.run()`.** There is no alternative for windowed apps.
The WinitPlugin runner installs itself via `set_runner`, so when `run()` is called it passes
the App into the winit event loop — which is the only way window events, rendering, and input
reach Bevy's ECS.

For headless/scenario runner use cases, use `ScheduleRunnerPlugin` + `.disable::<WinitPlugin>()`
and call `app.run()`. The ScheduleRunner receives the App by value, calls `finish()/cleanup()/update()`,
and returns — no OS event loop involved.
