/*!
Player events
*/

use snow2d::{input::Key, utils::arena::Index};

use rlbox::rl::grid2d::*;

use crate::{
    ctrl::rogue::tick::{Event, EventContext, EventResult, GenAnim},
    data::world::actor::{Actor, Relation},
};

use super::*;

/// TODO: delegating the process to FSM?
#[derive(Debug)]
pub struct InteractWithActor {
    pub from: Index<Actor>,
    pub to: Index<Actor>,
}

impl GenAnim for InteractWithActor {}

impl Event for InteractWithActor {
    fn run(&self, _ecx: &mut EventContext) -> EventResult {
        EventResult::Finish
    }
}

#[derive(Debug)]
pub struct Interact {
    pub actor: Index<Actor>,
    pub dir: Dir8,
}

impl GenAnim for Interact {}

impl Event for Interact {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        let actor = &ecx.world.entities[self.actor];
        let pos = actor.pos + Vec2i::from(self.dir);

        if let Some((target_ix, target)) = ecx.world.entities.iter().find(|(_i, e)| e.pos == pos) {
            match target.relation {
                Relation::Friendly => EventResult::chain(InteractWithActor {
                    from: self.actor,
                    to: target_ix,
                }),
                Relation::Hostile => EventResult::chain(MeleeAttack {
                    actor: self.actor,
                    dir: Some(self.dir),
                }),
            }
        } else {
            EventResult::chain(NotConsumeTurn { actor: self.actor })
        }
    }
}

/// Walk or change direction and chain [`PlayerTurn`]
#[derive(Debug)]
pub struct PlayerWalk {
    pub actor: Index<Actor>,
    pub dir: Dir8,
}

impl GenAnim for PlayerWalk {}

impl Event for PlayerWalk {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        let EventContext { world, vi, .. } = ecx;

        let actor = &mut world.entities[self.actor];
        let pos = actor.pos + Vec2i::from(self.dir.signs_i32());
        drop(actor);

        let is_rotate_only = vi.turn.is_down();

        if is_rotate_only || world.is_blocked(pos) {
            EventResult::chain(ChangeDir {
                actor: self.actor,
                dir: self.dir,
            })
        } else {
            let actor = &world.entities[self.actor];

            EventResult::chain(Move {
                actor: self.actor,
                mcx: MoveContext::Walk,
                from_pos: actor.pos,
                to_pos: pos,
                from_dir: actor.dir,
                to_dir: self.dir,
            })
        }
    }
}

/// Interactive command for player input. TODO: Extract the handler
#[derive(Debug)]
pub struct PlayerTurn {
    pub actor: Index<Actor>,
}

impl GenAnim for PlayerTurn {}

impl PlayerTurn {
    /// Find he only actor that is at an adjacent cell to the controlled actor
    fn find_only_neighbor(&self, ecx: &EventContext) -> Option<Dir8> {
        let mut res = Option::<Dir8>::None;

        let origin = ecx.world.entities[self.actor].pos;
        for (_ix, e) in &ecx.world.entities {
            let dvec = e.pos - origin;
            if dvec.len_king() != 1 {
                continue;
            }

            if res.is_some() {
                return None;
            }

            res = Dir8::from_signs([Sign::from_i32(dvec.x), Sign::from_i32(dvec.y)]);
        }

        res
    }
}

impl Event for PlayerTurn {
    fn run(&self, ecx: &mut EventContext) -> EventResult {
        let (select, turn, rest, dir, enter) = (
            ecx.vi.select.is_pressed(),
            ecx.vi.turn.is_pressed(),
            ecx.vi.rest.is_pressed(),
            ecx.vi.dir.dir8_down(),
            ecx.ice.input.kbd.is_key_down(Key::Enter),
        );

        if select {
            return EventResult::chain(Interact {
                actor: self.actor,
                dir: ecx.world.entities[self.actor].dir,
            });
        }

        if turn {
            if let Some(dir) = self.find_only_neighbor(ecx) {
                return EventResult::chain(ChangeDir {
                    actor: self.actor,
                    dir,
                });
            }
        }

        if rest {
            return EventResult::chain(RestOneTurn { actor: self.actor });
        }

        if let Some(dir) = dir {
            let is_shift_down = ecx
                .ice
                .input
                .kbd
                .is_any_key_down(&[Key::LShift, Key::RShift]);

            return if is_shift_down {
                EventResult::chain(ChangeDir {
                    actor: self.actor,
                    dir,
                })
            } else {
                // walk
                EventResult::chain(PlayerWalk {
                    actor: self.actor,
                    dir,
                })
            };
        }

        if enter {
            return EventResult::chain(MeleeAttack {
                actor: self.actor,
                dir: None,
            });
        }

        EventResult::GotoNextFrame
    }
}
