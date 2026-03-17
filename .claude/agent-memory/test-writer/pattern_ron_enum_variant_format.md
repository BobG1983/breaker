---
name: RON enum variant deserialization format
description: The correct RON syntax for struct-like enum variants vs newtype wrapping
type: reference
---

## Named-field enum variants vs newtype wrapping in RON

### Named-field variant (Rust struct-like syntax)
```rust
enum Foo { Bar { x: i32, y: i32 } }
```
RON: `Bar(x: 1, y: 2)` — single parens, fields inline

### Newtype variant wrapping a struct
```rust
struct BarParams { x: i32, y: i32 }
enum Foo { Bar(BarParams) }
```
RON: `Bar((x: 1, y: 2))` — double parens, struct literal inside

## When the spec uses double-paren RON

If a spec shows `Chaos((seed: 42, action_prob: 0.3))`, that means the enum variant is a **newtype wrapping a struct** — not named fields directly in the variant. The types must use the wrapping pattern:

```rust
pub struct ChaosParams { pub seed: u64, pub action_prob: f32 }
pub enum InputStrategy { Chaos(ChaosParams), ... }
```

Named-field variants would require single-paren RON: `Chaos(seed: 42, action_prob: 0.3)`.

## Impact on test writing

When spec provides RON strings with double parens, write types to match (newtype structs).
When spec shows named-field type definitions, check if RON strings are single or double paren — they must be consistent. If inconsistent, flag as an ambiguity and choose the RON format as authoritative (it's what .ron files will actually contain).
