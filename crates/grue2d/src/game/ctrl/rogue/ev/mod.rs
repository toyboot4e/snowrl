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

use snow2d::{
    asset::AssetKey,
    input::Dir8,
    ui::Node,
    utils::{arena::Index, tyobj::*},
};

use anyhow::*;

use rlbox::{rl::grid2d::Vec2i, view::anim::*};

use crate::game::{
    data::{res::*, world::actor::Actor},
    Data,
};

pub fn play_sound<'a>(sound: AssetKey<'a>, data: &mut Data) -> Result<()> {
    let assets = &mut data.ice.assets;
    let audio = &data.ice.audio;

    assets.play(sound, audio)?;
    Ok(())
}

pub fn play_sound_preserve<'a>(sound: impl Into<AssetKey<'a>>, data: &mut Data) -> Result<()> {
    let assets = &mut data.ice.assets;
    let audio = &data.ice.audio;

    assets.play_preserve(sound, audio)?;
    Ok(())
}

pub fn run_dir_anim(id: impl Into<String>, pos: Vec2i, dir: Dir8, data: &mut Data) {
    let pos = rlbox::render::tiled::t2w_center(pos, &data.world.map.tiled);

    data.res.dir_anims.add({
        let anim_type = TypeObjectId::<DirAnimType>::from_raw(id.into())
            .try_retrieve()
            .unwrap();
        let state = DirAnimState::from_tyobj(&*anim_type);

        let layer = UiLayer::OnActors;
        let anim_layer = data.res.ui.layer_mut(layer);

        let node = anim_layer.nodes.add({
            let mut node = Node::from(state.current_frame());
            node.params.pos = pos;
            node
        });

        DirAnimEntry {
            node,
            layer,
            state,
            dir,
        }
    });
}

pub fn run_dir_anim_at_actor(id: impl Into<String>, actor: Index<Actor>, data: &mut Data) {
    let actor = &data.world.entities[actor];
    self::run_dir_anim(id, actor.pos, actor.dir, data);
}
