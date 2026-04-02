# Idiom Research: Enforcing Required Enum Variants via Trait

## Context

The `rantzsoft_lifecycle` crate needs a `ScreenLifecycle` trait that generic systems can use
to drive state machines forward through a fixed phase sequence. Every screen state enum
(`NodeState`, `ChipSelectState`, `MenuState`, etc.) must expose the standard phases
(`Loading`, `AnimateIn`, `AnimateOut`, `Teardown`) and one screen-specific "active" phase
(`Playing`, `Selecting`, `Setup`, etc.).

These enums already derive Bevy's `States` or `SubStates`. The question is how to layer
`ScreenLifecycle` on top so:
1. A generic system can call `S::loading()` to get the right variant.
2. Missing a required variant is caught at compile time, not at runtime.

---

## Fundamental Constraint — Rust Cannot Require Specific Variant Names

Traits in Rust cannot express "this enum must have a variant named `Loading`." The language
has no "variant types" feature (tracked in rust-lang/lang-team#122; not scheduled for stable).
Every approach here is a workaround. The workaround you choose determines where the
enforcement happens: at compile time via the type system, at compile time via a proc macro,
or not at all.

---

## Approach 1 — Associated Methods (Recommended)

### Pattern

```rust
// In rantzsoft_lifecycle
pub trait ScreenLifecycle: States {
    fn loading()    -> Self;
    fn animate_in() -> Self;
    fn animate_out() -> Self;
    fn teardown()   -> Self;
}
```

Each enum provides the required variant for each phase:

```rust
// In breaker-game
impl ScreenLifecycle for NodeState {
    fn loading()    -> Self { Self::Loading }
    fn animate_in() -> Self { Self::AnimateIn }
    fn animate_out() -> Self { Self::AnimateOut }
    fn teardown()   -> Self { Self::Teardown }
}

impl ScreenLifecycle for ChipSelectState {
    fn loading()    -> Self { Self::Loading }
    fn animate_in() -> Self { Self::AnimateIn }
    fn animate_out() -> Self { Self::AnimateOut }
    fn teardown()   -> Self { Self::Teardown }
}
```

Generic systems use `S::loading()`:

```rust
fn advance_to_animate_in<S: ScreenLifecycle>(
    state: Res<State<S>>,
    mut next: ResMut<NextState<S>>,
    mut events: EventReader<PhaseComplete>,
) {
    for _ in events.read() {
        if *state == S::loading() {
            next.set(S::animate_in());
        }
    }
}
```

### Compile-Time Safety

Medium. The trait bounds force an impl to exist, so forgetting to `impl ScreenLifecycle` is
a compiler error. However, the wrong variant can be returned silently — nothing stops
`fn loading() -> Self { Self::AnimateIn }`. The enforcement is structural, not nominal.

In practice this is acceptable: the mapping is a one-liner per variant. A wrong mapping
would produce wrong behavior that tests would catch (the generic advancing system would skip
phases), not a silent correctness hole.

### Ergonomics

Very low boilerplate. Four lines per enum. No macros. No additional crates.

### Interaction With `States`

`States` bounds are: `'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug`.
`ScreenLifecycle: States` is a supertrait — the blanket impl `States` already gives you all
these. The only thing to check is that `fn loading() -> Self` does not conflict with any
`States` method — it does not, because `States` defines no methods (only the
`DEPENDENCY_DEPTH` associated constant, which has a default value).

**Conclusion: no conflict. `ScreenLifecycle: States` compiles and works as expected.**

### The "Active" Phase

The active phase (`Playing`, `Selecting`, `Setup`) differs per screen and the generic system
does not need it. Leave it out of the trait. Each screen's specific plugin drives the
active phase directly via its own systems. The generic advancing system only needs to know
about the phases it crosses.

If future generic systems do need it, add it:

```rust
fn active() -> Self;
```

Do not add it now — YAGNI until there is a concrete second use case.

---

## Approach 2 — Derive Macro (Avoids Boilerplate, Adds Enforcement)

### Pattern

A derive macro reads the enum's variant names and:
1. Validates that `Loading`, `AnimateIn`, `AnimateOut`, `Teardown` all exist.
2. Emits a `syn::Error` with a clear span if any are missing.
3. Emits the `impl ScreenLifecycle` automatically.

```rust
#[derive(States, ScreenLifecycle)]
enum NodeState {
    Loading,
    AnimateIn,
    Playing,
    AnimateOut,
    Teardown,
}
```

Generates (conceptually):
```rust
impl ScreenLifecycle for NodeState {
    fn loading()    -> Self { Self::Loading }
    fn animate_in() -> Self { Self::AnimateIn }
    fn animate_out() -> Self { Self::AnimateOut }
    fn teardown()   -> Self { Self::Teardown }
}
```

Missing `Teardown`:
```
error: ScreenLifecycle: enum `NodeState` is missing required variant `Teardown`
 --> src/shared/node_state.rs:5:10
  |
5 | enum NodeState {
  |      ^^^^^^^^
```

### Implementation in proc macro terms

```rust
// In rantzsoft_lifecycle_derive (a new proc-macro crate)
const REQUIRED: &[&str] = &["Loading", "AnimateIn", "AnimateOut", "Teardown"];

#[proc_macro_derive(ScreenLifecycle)]
pub fn derive_screen_lifecycle(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let syn::Data::Enum(data) = &input.data else {
        return syn::Error::new(name.span(), "ScreenLifecycle only supports enums")
            .to_compile_error()
            .into();
    };

    let variant_names: Vec<String> = data.variants.iter()
        .map(|v| v.ident.to_string())
        .collect();

    for required in REQUIRED {
        if !variant_names.contains(&required.to_string()) {
            return syn::Error::new(
                name.span(),
                format!("ScreenLifecycle: enum `{}` is missing required variant `{}`", name, required),
            )
            .to_compile_error()
            .into();
        }
    }

    let expanded = quote! {
        impl ::rantzsoft_lifecycle::ScreenLifecycle for #name {
            fn loading()    -> Self { Self::Loading }
            fn animate_in() -> Self { Self::AnimateIn }
            fn animate_out() -> Self { Self::AnimateOut }
            fn teardown()   -> Self { Self::Teardown }
        }
    };

    TokenStream::from(expanded)
}
```

### Compile-Time Safety

High. Missing variants are caught at the derive site, with a clear error message pointing to
the enum definition. Wrong mappings cannot happen — the macro hardcodes the variant name to
match the method.

### Ergonomics

Minimal callsite: one `#[derive]` attribute. No impl block to write. The cost is a new
`rantzsoft_lifecycle_derive` proc-macro crate (mirroring the existing
`rantzsoft_defaults_derive` precedent — the project already knows this pattern).

### Interaction With `States`

No conflict. The derive macro emits `impl ScreenLifecycle` independently of whatever
`States` or `SubStates` derive does. Bevy's derives use `#[proc_macro_derive(States)]` and
`#[proc_macro_derive(SubStates)]` — they do not conflict with a custom derive.

### The "Active" Phase

The derive macro should NOT attempt to detect the active phase. The active variant name
differs per screen and is not in a fixed list. The macro only validates and maps the four
known phases.

---

## Approach 3 — Enum Variant as Associated Type

### Pattern

```rust
trait ScreenLifecycle: States {
    type ActivePhase; // ???
}
```

This does not work. There is no way for an enum variant to be a type. `NodeState::Playing`
is a value, not a type. This would require Rust's unimplemented "variant types" feature.

The closest workable version would be a separate "active phase" newtype per screen, but that
introduces a second type with no benefit — the generic system doesn't use the active phase
anyway (see above). Discard this approach.

---

## Recommendation

**Use Approach 1 (associated methods) now. Add Approach 2 (derive macro) when the trait
stabilizes and there are 3+ implementors.**

### Reasoning

The project has two screen-level sub-states today (`NodeState`, `ChipSelectState`) and will
gain `MenuState` from this migration — three total. Four-line manual impls are readable and
unambiguous. The boilerplate cost of Approach 1 at three implementors is 12 lines of
mechanical code.

Approach 2 adds nominal compile-time safety (wrong variant name → hard error) but requires
a new proc-macro crate with its own `Cargo.toml`, `syn`, `quote`, `proc-macro2` dependencies,
and workspace registration. The project has this precedent (`rantzsoft_defaults_derive`) but
adding it for 12 lines of boilerplate violates the project's explicit YAGNI rule from
`docs/architecture/standards.md`:

> No abstractions, generics, or indirection until there's a concrete second use case.
> Three similar lines > premature abstraction.

Three impls are not a problem. If the design expands to 6–8 screen state types, revisit.
At that point the derive macro is clearly worth it and the trait API will be validated.

A middle path that captures most of Approach 2's benefit without a new crate: add a
`debug_assert!` in the trait's generic advancing system that the enum actually transitions
when `S::loading()` is set. This catches wrong-variant mapping in dev builds without
compile-time machinery.

---

## Alternatives Considered

| Approach | Why Not |
|---|---|
| `macro_rules!` exhaustiveness check | Validates variants are mentioned in a match, but does not generate the impl. Adds noise without reducing boilerplate. Useful only as a test helper. |
| Enum variant as associated type | Requires unimplemented Rust feature (variant types). Discard. |
| Separate `LoadingState`, `AnimateInState` newtype structs | Each screen would have 4+ separate state types instead of 1. SubStates nesting becomes impractical. Discarded. |
| Single flat `LifecyclePhase` enum (not per-screen) | Loses type-level distinction between screens. Can't use `in_state(NodeState::Playing)` guards. `SubStates` source type can't encode it. Discarded. |
| Runtime `match self` + string map | No compile-time enforcement at all. Discard. |

---

## Codebase Precedent

- `breaker-game/src/shared/game_state.rs` — `States` derive on a plain enum. No additional trait.
- `breaker-game/src/shared/playing_state.rs` — `SubStates` derive, `#[source(...)]` attribute.
  Shows how the project currently handles sub-state typing.
- `rantzsoft_defaults_derive/src/lib.rs` — The project knows how to write proc-macro crates
  with `syn`/`quote`. This is the reference if Approach 2 becomes warranted.
- `breaker-game/src/screen/systems/cleanup.rs:6` — `cleanup_entities<T: Component>` shows
  the existing pattern for generic systems parameterised on a type. `ScreenLifecycle: States`
  follows the same shape: a supertrait bound that enables generic system registration.

---

## Open Questions for the Implementor

1. **`fn active() -> Self`?** The design doc says each screen has one "active" phase whose
   name differs per screen. If the generic advancing system from `ChipSelectState::AnimateIn`
   to the active phase needs to be generic, you need `fn active() -> Self`. If each screen
   writes its own advance-to-active system, you do not. Decide before implementing.

2. **Where does `ScreenLifecycle` live?** It should be in `rantzsoft_lifecycle` since that
   crate owns the generic phase logic. The impls go in `breaker-game/src/shared/` alongside
   the state definitions (same pattern as `PlayingState` lives next to `GameState`).

3. **`PhaseComplete<S>` message and `ScreenLifecycle`?** If the generic advancing system is
   parameterised on `S: ScreenLifecycle`, it needs a generic message type. Ensure
   `PhaseComplete<S>` (or whatever the message is) is also in `rantzsoft_lifecycle` and
   does not carry game vocabulary.
