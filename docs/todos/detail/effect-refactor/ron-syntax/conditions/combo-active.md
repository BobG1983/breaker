# Name
ComboActive

# Parameters
- `u32`: The number of consecutive perfect bumps required to activate

# Description
True while the player's consecutive perfect bump streak is at or above the given count. Becomes true when the streak reaches the threshold, becomes false when a non-perfect bump breaks the streak (resetting it to zero).

During(ComboActive(3), Fire(SpeedBoost(2.0))) means "double speed while you've maintained a streak of 3 or more consecutive perfect bumps." One early or late bump breaks the streak and removes the boost. Hit 3 perfects in a row again and the boost reactivates.

Different ComboActive thresholds are independent — ComboActive(3) and ComboActive(5) on the same entity activate and deactivate at their own thresholds.
