# Messages — Inter-System Communication

Systems are decoupled through Bevy 0.18 messages (`#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`). The breaker plugin doesn't import audio. The cell plugin doesn't import upgrades. Messages connect them.

**Example message types:**

| Message | Sent By | Consumed By |
|---------|---------|-------------|
| `BoltHitBreaker` | physics | audio, upgrades, UI |
| `BoltHitCell` | physics | upgrades, cells, audio |
| `CellDestroyed` | cells | run (progress tracking), upgrades (overclock triggers), audio |
| `BoltLost` | physics | breaker (applies penalty per breaker trait) |
| `NodeCleared` | run | state machine, UI |
| `UpgradeSelected` | UI | upgrades (apply effects) |
| `BumpPerformed { grade }` | breaker | audio, upgrades (overclock triggers) |
| `TimerExpired` | run | state machine |
