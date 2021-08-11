/*!
Game states
*/

pub use gui::fsm::{self, State, StateCell};

use std::any::TypeId;

use gui::{core, Data};

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

    fn on_enter(&mut self, _cell: &StateCell<Self::Data>, _data: &mut Data) {
        //
    }

    fn on_exit(&mut self, _cell: &StateCell<Self::Data>, _data: &mut Data) {
        //
    }

    fn on_stop(&mut self, _cell: &StateCell<Self::Data>, _data: &mut Data) {
        //
    }

    fn event(&mut self, _cell: &StateCell<Self::Data>, _data: &mut Data) {
        //
    }

    fn update(&mut self, _cell: &StateCell<Self::Data>, data: &mut Data) -> StateReturn {
        let mut states = vec![];

        let res = core::sys::tick(&mut data.system);
        if !res.tree.is_empty() {
            states.push(StateCommand::Push(TypeId::of::<GuiSync>()));
        }
        if let Some(_gui) = res.gui {
            states.push(StateCommand::Push(TypeId::of::<PlayerState>()));
        }

        StateReturn::ThisFrame(states)
    }
}

/// State for syncing the view model to the internal model
pub struct GuiSync {
    //
}

/// State for controlling the player
#[derive(Debug, Default)]
pub struct PlayerState {
    //
}
