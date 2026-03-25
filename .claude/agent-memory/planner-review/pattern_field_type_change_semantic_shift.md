---
name: Field type change causes semantic logic shift
description: When a struct field changes from a simple type to a compound type (e.g., leaf enum -> tree enum), code that wraps/unwraps the old value must change its logic, not just its types
type: feedback
---

When a field changes from a "leaf" type to a "tree" type (e.g., `Option<TriggerChain>` storing a leaf to `Option<EffectNode>` storing a full tree), the consuming code's logic changes:

Old: `if let Some(leaf) = &def.field { chains.push(Wrapper(vec![leaf.clone()])); }`
New: `if let Some(tree) = &def.field { chains.push(tree.clone()); }`

**Why:** Specs that only describe the type change ("field changes from X to Y") miss the logic change. Writer-code will change the types but keep the wrapping logic, producing double-wrapped trees.

**How to apply:** When reviewing type changes on struct fields, check every consuming site for logic that wraps/unwraps the old type. If the new type already contains what the wrapping logic was adding, the wrapping must be removed. Flag this as a semantic change requiring its own behavior test.
