/*!
Builtin roguelike game events

Every change to the roguelike game world should be caused by primitive event. That's good for
visualization, flexibility and simplicity. Examples:

1) `MeleeAttack` → `Attack` → `Hit` → `GiveDamage`. For each step, we can stop processing our game
and play animation. Most combat events result in `GiveDamage` and we can simply reused the same
event handler.

2) `Heal` → `ApplyHeal`. `Heal` may be overridden by `ZombieRule` to `GiveDamage` if the healed
entity is a zombie.
*/

mod primitive;
pub use self::primitive::*;

mod high;
pub use self::high::*;

mod player;
pub use self::player::*;
