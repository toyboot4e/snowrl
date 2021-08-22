/*!
Game states
*/

use std::any::TypeId;

use snow2d::utils::arena::Slot;

use model::evs::*;

use gui::{content::*, prelude::*};

pub type StateReturn = fsm::StateReturn<Data>;
pub type StateCommand = fsm::StateCommand<Data>;

/// State for ticking the internal game state
#[derive(Debug, Default)]
pub struct TickState {
    // game_loop: GameLoop,
    current_frame_count: u64,
    last_frame_on_tick: u64,
}

impl State for TickState {
    type Data = Data;

    fn on_enter(&mut self, _data: &mut Data, _cell: &StateCell<Self::Data>) {
        //
    }

    fn on_exit(&mut self, _data: &mut Data, _cell: &StateCell<Self::Data>) {
        //
    }

    fn on_stop(&mut self, _data: &mut Data, _cell: &StateCell<Self::Data>) {
        //
    }

    fn event(&mut self, _data: &mut Data, _cell: &StateCell<Self::Data>) {
        //
    }

    fn update(&mut self, data: &mut Data, cell: &StateCell<Self::Data>) -> StateReturn {
        log::trace!("tick-update");
        let mut states = vec![];

        let res = rlcore::sys::tick(&mut data.system);
        if !res.tree.is_empty() {
            states.push(StateCommand::Push(TypeId::of::<GuiSync>()));
        }
        if let Some(tag) = res.gui {
            match tag.as_str() {
                "gui::content::PlayerAi" => {
                    let pl = cell.get_mut::<PlayerState>().unwrap();
                    let pl_ix = data
                        .system
                        .model
                        .entities
                        .upgrade(Slot::from_raw(0))
                        .unwrap();
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
    //
}

impl State for GuiSync {
    type Data = Data;

    fn update(&mut self, _data: &mut Data, _cell: &StateCell<Self::Data>) -> StateReturn {
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

impl PlayerState {
    fn logic(&self, data: &mut Data) -> Option<DynEvent> {
        let vi = &data.res.vi;
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
            if let Some(dir) = self::find_only_neighbor(entity, &mut data.system.model) {
                return Some(Box::new(DirChange {
                    entity,
                    dir,
                    kind: DirChangeKind::Smooth,
                }));
            }
        }

        if rest {
            return Some(Box::new(RestOneTurn { entity }));
        }

        if let Some(dir) = dir {
            let ev: DynEvent = if data.res.vi.turn.is_down() {
                Box::new(DirChange {
                    entity,
                    dir,
                    kind: DirChangeKind::Smooth,
                })
            } else {
                // walk
                Box::new(PlayerWalk { entity, dir })
            };

            return Some(ev);
        }

        None
    }
}

impl State for PlayerState {
    type Data = Data;

    fn on_enter(&mut self, _data: &mut Self::Data, _cell: &StateCell<Self::Data>) {
        assert!(self.entity.is_some());
    }

    fn on_exit(&mut self, _data: &mut Self::Data, _cell: &StateCell<Self::Data>) {
        self.entity = None;
    }

    fn update(&mut self, data: &mut Data, _cell: &StateCell<Self::Data>) -> StateReturn {
        if let Some(_ev) = self.logic(data) {
            // TODO: push event to the game system
            StateReturn::ThisFrame(vec![StateCommand::Pop])
        } else {
            StateReturn::NextFrame(vec![])
        }
    }
}
