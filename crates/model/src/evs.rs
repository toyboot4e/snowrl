/*!
Events
*/

use derivative::Derivative;

use snow2d::{input::Dir8, utils::arena::Index};

use rlcore::{
    ev::{hub::EventHubBuilder, Event},
    grid2d::Vec2i,
    sys::UiEvent,
};

use crate::{entity::EntityModel, GameSystem};

#[derive(Debug, Clone)]
pub struct PosChange {
    pub actor: Index<EntityModel>,
    pub to: Vec2i,
    pub kind: PosChangeKind,
}

impl Event for PosChange {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PosChangeKind {
    Walk,
    Teleport,
}

#[derive(Debug, Clone)]
pub struct DirChange {
    pub actor: Index<EntityModel>,
    pub to: Dir8,
    pub kind: PosChangeKind,
}

impl Event for DirChange {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DirChangeKind {
    Immediate,
    Smooth,
}

pub struct PlayerCommand;

impl UiEvent for PlayerCommand {}

pub fn init(builder: &mut EventHubBuilder<GameSystem>) {
    builder.ev::<PosChange>().ev::<DirChange>();
    builder.hnd(Box::new(|ev: &PosChange, model| None));
}
