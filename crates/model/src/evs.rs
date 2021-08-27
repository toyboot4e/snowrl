/*!
Event types
*/

use snow2d::{input::Dir8, utils::arena::Index};

use rlcore::{
    ev::{
        hub::{EventHubBuilder, HandleResult},
        Event,
    },
    grid2d::Vec2i,
};

use crate::{chg, entity::EntityModel, GameSystem};

/// Registers model events and default event handlers to [`EventHubBuilder`]
pub fn builder_plugin(builder: &mut EventHubBuilder<GameSystem>) {
    builder.ev_with(Box::new(|ev: &PlayerWalk, args| {
        let entity = &args.entities[ev.entity];
        let pos = entity.pos + Vec2i::from(ev.dir);

        let chg = chg::PosChange {
            entity: ev.entity,
            pos,
            dir: Some(ev.dir),
            kind: chg::PosChangeKind::Walk,
        };

        args.make_change(&chg.into());
        // no chain

        HandleResult::Handled
    }));

    builder.ev_with(Box::new(|_ev: &RestOneTurn, _args| HandleResult::Handled));
}

#[derive(Debug, Clone)]
pub struct PlayerWalk {
    pub entity: Index<EntityModel>,
    pub dir: Dir8,
}

impl Event for PlayerWalk {}

#[derive(Debug, Clone)]
pub struct RestOneTurn {
    pub entity: Index<EntityModel>,
}

impl Event for RestOneTurn {}
