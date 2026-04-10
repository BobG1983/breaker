# Name
TimeExpires

# Parameters
`f32` — seconds.

# Description
Internal countdown trigger. A timer counts down from the specified value each tick. When it reaches zero, the trigger fires on the Owner and the entry is consumed. Not typically written in chip RON directly — it's generated internally by Until desugaring. `Until(TimeExpires(3.0), Fire(SpeedBoost(1.5)))` means "speed boost for 3 seconds then reverse."
