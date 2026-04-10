# Name
NodeEndOccurred

# When it fires
The current node ends. All cells are cleared or the node is otherwise complete.

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
NodeEndOccurred signals the end of the current node. Use this for effects that should trigger or clean up at node completion.

No participant context — node lifecycle events have no participants.
DO NOT use On(...) inside a NodeEndOccurred tree — there are no participants to resolve.
