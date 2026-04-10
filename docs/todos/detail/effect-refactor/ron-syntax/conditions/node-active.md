# Name
NodeActive

# Parameters
None

# Description
True while the current node is in play — from the moment the node starts until it is torn down. This spans both the playing and paused states, so pausing the game does not deactivate NodeActive effects.

This is the most common condition. During(NodeActive, Fire(SpeedBoost(1.5))) means "speed boost for the entire node, removed when the node ends." If the player moves to the next node, the boost reapplies when NodeActive becomes true again.
