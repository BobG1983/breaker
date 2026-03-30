# Adding a New Trigger

1. **Create** `effect/triggers/new_trigger.rs`:
   - Define `pub(crate) fn register(app: &mut App)` — adds the bridge system to the schedule
   - Define the bridge system — reads the appropriate game message, walks chains, queues commands
   - Determine scope (global or targeted) and On target resolution

2. **Add variant** to the `Trigger` enum in `effect/core/types/definitions.rs`.

3. **Call** `new_trigger::register(app)` in `effect/triggers/mod.rs`.

4. **RON files** can immediately use `When(trigger: NewTrigger, then: [...])`.
