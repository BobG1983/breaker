# Name
Killed

# Parameters
`EntityKind` — what type of entity was killed.

# Description
Fires on the entity that caused the death. This is the killer's perspective — "I just killed something." `Killed(Cell)` on a bolt means "I just killed a cell." `Killed(Any)` means "I killed anything." Does NOT fire for environmental deaths (no killer entity). Use for kill-reward effects like speed boosts on kill.
