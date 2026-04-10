# Name
GameEntity

# Syntax
```rust
trait GameEntity: Component {}

impl GameEntity for Bolt {}
impl GameEntity for Cell {}
impl GameEntity for Wall {}
impl GameEntity for Breaker {}
```

# Description
Marker trait for entity types that participate in the death pipeline. Used as a generic bound on `DamageDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, and `apply_damage<T>`.

Each impl creates a separate Bevy message queue — `DamageDealt<Cell>` and `DamageDealt<Bolt>` are independent message types.

DO NOT add GameEntity to types that are not top-level game entities. Components like `CellHealth` or `PrimaryBolt` are not game entities.
