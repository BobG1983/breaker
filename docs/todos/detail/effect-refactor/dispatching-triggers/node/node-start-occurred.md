# Name
NodeStartOccurred

# When it fires
A new node begins. The playfield is set up and gameplay is about to start.

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
NodeStartOccurred signals the beginning of a new node. Use this for effects that should activate or reset at the start of each node.

No participant context — node lifecycle events have no participants.
DO NOT use On(...) inside a NodeStartOccurred tree — there are no participants to resolve.
