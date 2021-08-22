/*!
Events
*/

use snow2d::{input::Dir8, utils::arena::Index};

use rlcore::{
    ev::{hub::EventHubBuilder, Event},
    grid2d::Vec2i,
};

use crate::{entity::EntityModel, GameSystem};

/// Registers model events and default event handlers to [`EventHubBuilder`]
pub fn builder_plugin(builder: &mut EventHubBuilder<GameSystem>) {
    builder.ev_with(Box::new(|ev: &PosChange, model| {
        let entity = &mut model.entities[ev.entity];
        entity.pos = ev.pos;
        if let Some(dir) = ev.dir.clone() {
            entity.dir = dir;
        }
        None
    }));
    builder.ev_with(Box::new(|ev: &DirChange, model| {
        let entity = &mut model.entities[ev.entity];
        entity.dir = ev.dir;
        None
    }));
    // TODO: PlayerWalk event
    builder.ev_with(Box::new(|_ev: &RestOneTurn, _model| None));
}

#[derive(Debug, Clone)]
pub struct PosChange {
    pub entity: Index<EntityModel>,
    pub pos: Vec2i,
    pub dir: Option<Dir8>,
    pub kind: PosChangeKind,
}

impl Event for PosChange {}

#[derive(Debug, Clone)]
pub struct PlayerWalk {
    pub entity: Index<EntityModel>,
    pub dir: Dir8,
}

impl Event for PlayerWalk {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PosChangeKind {
    Walk,
    Teleport,
}

#[derive(Debug, Clone)]
pub struct DirChange {
    pub entity: Index<EntityModel>,
    pub dir: Dir8,
    pub kind: DirChangeKind,
}

impl Event for DirChange {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DirChangeKind {
    Immediate,
    Smooth,
}

#[derive(Debug, Clone)]
pub struct RestOneTurn {
    pub entity: Index<EntityModel>,
}

impl Event for RestOneTurn {}
