/*!
Game states
*/

use std::any::TypeId;

use snow2d::utils::arena::Slot;

use model::{evs::*, EventTree};

use gui::{content::*, prelude::*};

pub type StateReturn = fsm::StateReturn<God>;
pub type StateCommand = fsm::StateCommand<God>;

/// State for ticking the internal game state
#[derive(Debug, Default)]
pub struct TickState;

impl State for TickState {
    type Data = God;

    fn update(&mut self, god: &mut God, cell: &StateCell<Self::Data>) -> StateReturn {
        log::trace!("tick-update");
        let mut states = vec![];

        let res = rlcore::sys::tick(&mut god.sys);
        if !res.tree.is_empty() {
            let sync = cell.get_mut::<GuiSync>().unwrap();
            sync.tree = res.tree;
            states.push(StateCommand::Push(TypeId::of::<GuiSync>()));
        }

        // handle GUI event
        if let Some(tag) = res.gui {
            // TODO: const match
            match tag.as_str() {
                x if x == PlayerAi::GUI => {
                    let pl = cell.get_mut::<PlayerState>().unwrap();
                    let pl_ix = god.sys.model().entities.upgrade(Slot::from_raw(0)).unwrap();
                    pl.entity = Some(pl_ix);

                    states.push(StateCommand::Push(TypeId::of::<PlayerState>()))
                }
                _ => {
                    panic!("Unknown UI event: {}", tag.as_str());
                }
            }
        }

        StateReturn::ThisFrame(states)
    }
}

/// State for syncing the view model to the internal model
#[derive(Debug, Default)]
pub struct GuiSync {
    // TODO: Swap buffer
    tree: EventTree,
}

impl State for GuiSync {
    type Data = God;

    fn update(&mut self, _god: &mut God, _cell: &StateCell<Self::Data>) -> StateReturn {
        log::trace!("gui sync");
        StateReturn::NextFrame(vec![])
    }
}

fn find_only_neighbor(enitiy: Index<EntityModel>, model: &Model) -> Option<Dir8> {
    let mut res = Option::<Dir8>::None;

    let origin = model.entities[enitiy].pos;
    for (_ix, e) in &model.entities {
        let delta = e.pos - origin;
        if delta.len_king() != 1 {
            continue;
        }

        if res.is_some() {
            return None;
        }

        res = Dir8::from_signs([Sign::from_i32(delta.x), Sign::from_i32(delta.y)]);
    }

    res
}

/// State for controlling the player
#[derive(Debug, Default)]
pub struct PlayerState {
    entity: Option<Index<EntityModel>>,
}

impl State for PlayerState {
    type Data = God;

    fn on_enter(&mut self, _data: &mut Self::Data, _cell: &StateCell<Self::Data>) {
        assert!(self.entity.is_some());
    }

    fn on_exit(&mut self, _data: &mut Self::Data, _cell: &StateCell<Self::Data>) {
        self.entity = None;
    }

    fn update(&mut self, god: &mut God, _cell: &StateCell<Self::Data>) -> StateReturn {
        if let Some(ev) = self.logic(god) {
            // TODO: Come back when it doesn't consume turn by looking into the tree?
            god.sys.publish(ev);
            StateReturn::ThisFrame(vec![StateCommand::Pop])
        } else {
            StateReturn::NextFrame(vec![])
        }
    }
}

impl PlayerState {
    // TODO: apply change
    fn logic(&self, god: &mut God) -> Option<DynEvent> {
        let vi = &god.res.vi;
        let (select, turn, rest, dir) = (
            vi.select.is_pressed(),
            vi.turn.is_pressed(),
            vi.rest.is_pressed(),
            vi.dir.dir8_down(),
        );

        let entity = self.entity.unwrap();

        if select {
            return Some(Box::new(Interact { entity, dir: None }));
        }

        if turn {
            if let Some(dir) = self::find_only_neighbor(entity, god.sys.model()) {
                let chg = chg::DirChange {
                    entity,
                    dir,
                    kind: chg::DirChangeKind::Smooth,
                };

                // NOTE: DirChange is visualized automatically!
                god.sys
                    .make_immediate_change(&mut god.gui.vm, &chg.upcast());
            }
        }

        if rest {
            let ev = RestOneTurn { entity };
            return Some(Box::new(ev));
        }

        if let Some(dir) = dir {
            if god.res.vi.turn.is_down() {
                // change direction without consuming turn
                let chg = chg::DirChange {
                    entity,
                    dir,
                    kind: chg::DirChangeKind::Smooth,
                };

                // NOTE: DirChange is visualized automatically!
                god.sys
                    .make_immediate_change(&mut god.gui.vm, &chg.upcast());
            } else {
                // walk
                let ev = PlayerWalk { entity, dir };
                return Some(Box::new(ev));
            };
        }

        None
    }
}
