/*!
`grue2d` is the global game states for SnowRL
*/

// use generator (unstable Rust)
#![feature(generators, generator_trait)]

pub mod render;
pub mod rl;
pub mod vi;

pub(crate) mod utils;

use {
    rlbox::ui::Ui,
    rokol::{
        app::{self as ra, RApp},
        gfx as rg, Rokol,
    },
    snow2d::Ice,
    std::{any::TypeId, collections::HashMap},
};

use crate::{
    render::WorldRenderer,
    rl::{script::ScriptRef, turn::anim::AnimPlayer, world::World},
    vi::VInput,
};

pub extern crate rokol;

pub extern crate snow2d;

pub extern crate rlbox;

pub extern crate hot_crate;
pub trait Plugin: std::fmt::Debug {}

/// Runs [`RApp`], which provides 60 FPS fixed-timestep game loop
pub fn run<App: RApp, AppConstructor: FnOnce(Rokol) -> App>(
    rokol: Rokol,
    constructor: AppConstructor,
) -> rokol::Result {
    rokol.run(&mut Runner {
        init_rokol: Some(rokol.clone()),
        init: Some(constructor),
        x: None,
    })
}

/// Creates [`RApp`] _after_ creating `rokol::gfx` contexts
struct Runner<T: RApp, F: FnOnce(Rokol) -> T> {
    init_rokol: Option<Rokol>,
    /// Use `Option` for lazy initialization
    init: Option<F>,
    /// Use `Option` for lazy initialization
    x: Option<T>,
}

impl<T: RApp, F: FnOnce(Rokol) -> T> rokol::app::RApp for Runner<T, F> {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());
        let f = self.init.take().unwrap();
        self.x = Some(f(self.init_rokol.take().unwrap()));
    }

    fn event(&mut self, ev: &ra::Event) {
        if let Some(x) = self.x.as_mut() {
            x.event(ev);
        }
    }

    fn frame(&mut self) {
        if let Some(x) = self.x.as_mut() {
            x.frame();
        }
    }
}

/// Runs a [`Fsm`] with some [`Global`] data
///
/// [`Fsm`] fsm::Fsm
/// [`Global`] fsm::Global
#[derive(Debug)]
pub struct GlueRl {
    pub gl: Global,
    pub fsm: Fsm,
}

impl GlueRl {
    pub fn new(gl: Global, fsm: Fsm) -> Self {
        Self { gl, fsm }
    }
}

/// Thread-local global game states
///
/// TODO: consider using `Global<T>` for additional contexts
#[derive(Debug)]
pub struct Global {
    pub world: World,
    pub ice: Ice,
    pub world_render: WorldRenderer,
    pub vi: VInput,
    pub ui: Ui,
    /// Roguelike game animations
    pub anims: AnimPlayer,
    /// TODO: extract it to user data
    pub script_to_play: Option<ScriptRef>,
}

impl Global {
    pub fn event(&mut self, ev: &ra::Event) {
        self.ice.event(ev);
    }

    /// Called before updating the FSM (game state)
    pub fn pre_update(&mut self) {
        self.ice.pre_update();
        self.vi.update(&self.ice.input, self.ice.dt);
        self.world.update(&mut self.ice);
    }

    /// Called after updating the FSM (game state)
    pub fn post_update(&mut self) {
        // TODO: don't hard code player detection
        let player = &self.world.entities.get_by_slot(0).unwrap().1;
        self.world
            .shadow
            .post_update(self.ice.dt, &self.world.map.rlmap, player);

        self.world_render.post_update(&self.world, self.ice.dt);

        self.ui.update(self.ice.dt);
    }

    pub fn pre_render(&mut self) {
        self.ice.pre_render();
    }

    pub fn on_end_frame(&mut self) {
        self.ice.on_end_frame();
    }
}

/// Game state lifecycle
pub trait GameState: std::fmt::Debug {
    fn on_enter(&mut self, _gl: &mut Global) {}
    fn on_exit(&mut self, _gl: &mut Global) {}
    // TODO: use proper name
    fn on_stop(&mut self, _gl: &mut Global) {}

    fn event(&mut self, _ev: &ra::Event, _gl: &mut Global) {}
    fn update(&mut self, _gl: &mut Global) -> StateReturn;
}

/// Return value of [`GameState::update`]
#[derive(Debug)]
pub enum StateReturn {
    /// Run every command in this frame. Call update in next frame
    NextFrame(Vec<StateCommand>),
    /// Run every command in this frame. Call update in this frame
    ThisFrame(Vec<StateCommand>),
}

impl StateReturn {
    pub fn into_cmds(self) -> Vec<StateCommand> {
        match self {
            Self::NextFrame(cmds) => cmds,
            Self::ThisFrame(cmds) => cmds,
        }
    }
}

/// Command in [`StateReturn`]
#[derive(Debug)]
pub enum StateCommand {
    Insert(TypeId, Box<dyn GameState>),
    Pop,
    PopAndRemove,
    Push(TypeId),
}

impl StateCommand {
    pub fn insert<T: GameState + 'static>(state: T) -> Self {
        Self::Insert(TypeId::of::<T>(), Box::new(state))
    }
}

/// Stack-based finite state machine
#[derive(Debug)]
pub struct Fsm {
    states: HashMap<TypeId, Box<dyn GameState>>,
    stack: Vec<TypeId>,
}

impl Default for Fsm {
    fn default() -> Self {
        Self {
            states: HashMap::with_capacity(10),
            stack: Vec::with_capacity(10),
        }
    }
}

impl Fsm {
    pub fn update(&mut self, gl: &mut Global) {
        loop {
            let id = self.stack.last().unwrap();
            let state = self.states.get_mut(id).unwrap();
            let res = state.update(gl);

            let finish = matches!(res, StateReturn::NextFrame(_));

            for cmd in res.into_cmds() {
                self.run_cmd(cmd, gl);
            }

            if finish {
                break;
            }
        }
    }

    fn run_cmd(&mut self, cmd: StateCommand, gl: &mut Global) {
        match cmd {
            StateCommand::Insert(typeid, state) => {
                self.states.insert(typeid, state);
            }
            StateCommand::Pop => {
                let _ = self.stack.pop().unwrap();
            }
            StateCommand::Push(typeid) => {
                self.push_id(typeid, gl);
            }
            StateCommand::PopAndRemove => {
                let typeid = self.stack.pop().unwrap();
                self.states.remove(&typeid).unwrap();
            }
        }
    }
}

/// State management
impl Fsm {
    pub fn insert<T: GameState + 'static>(&mut self, state: T) -> Option<Box<dyn GameState>> {
        self.states.insert(TypeId::of::<T>(), Box::new(state))
    }

    pub fn insert_default<T: GameState + 'static + Default>(
        &mut self,
    ) -> Option<Box<dyn GameState>> {
        self.states
            .insert(TypeId::of::<T>(), Box::new(T::default()))
    }

    pub fn push<T: GameState + 'static>(&mut self, gl: &mut Global) {
        let id = TypeId::of::<T>();
        self.push_id(id, gl);
    }

    pub fn push_id(&mut self, id: TypeId, gl: &mut Global) {
        if let Some(last_id) = self.stack.last() {
            let last = self.states.get_mut(last_id).unwrap();
            last.on_stop(gl);
        }

        let new = self.states.get_mut(&id).unwrap();
        new.on_enter(gl);

        self.stack.push(id);
    }

    pub fn pop(&mut self, gl: &mut Global) {
        let last_id = self.stack.last().unwrap();
        let last = self.states.get_mut(last_id).unwrap();
        last.on_exit(gl);

        self.stack.pop();
    }
}
