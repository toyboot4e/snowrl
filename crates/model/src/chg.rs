/*!
Mutations to the game model

Each change to the game model is synced to another game model owned by view, which is used for
visualization. This pattern almost completely decouples time of model and time of view.
*/

use std::fmt;

use snow2d::utils::arena::Index;

use rlcore::grid2d::*;

use crate::{entity::EntityModel, Model};

// pub enum Change

macro_rules! impl_from {
    ($ty:ident $(,)?) => {
        impl From<$ty> for Change {
            fn from(chg: $ty) -> Self {
                Self::$ty(chg)
            }
        }

        impl $ty {
            pub fn upcast(self) -> Change {
                Change::from(self)
            }
        }
    };

    ($ty:ident, $($x:ident),* $(,)?) => {
        impl_from!($ty);
        impl_from!($($x),*);
    };
}

macro_rules! def_change {
    ($($ty:ident),* $(,)?) => {
        /// One of the mutation types to the game model
        #[derive(Debug)]
        pub enum Change {
            $(
                $ty($ty),
            )*
        }

        impl_from!($($ty),*);
    }
}

def_change!(PosChange, DirChange, OpaqueChange);

impl Change {
    /// NOTE: Change will not be chained other changes
    pub fn apply(&self, mdl: &mut Model) {
        match self {
            Self::PosChange(chg) => {
                let ent = &mut mdl.ents[chg.ent];
                ent.pos = chg.pos;
                ent.dir = chg.dir.unwrap_or(ent.dir);
            }
            Self::DirChange(chg) => {
                let ent = &mut mdl.ents[chg.ent];
                ent.dir = chg.dir;
            }
            Self::OpaqueChange(chg) => {
                (chg.proc)(mdl);
            }
        }
    }
}

/// Changes entity's position and optionally direction
#[derive(Debug, Clone)]
pub struct PosChange {
    pub ent: Index<EntityModel>,
    pub pos: Vec2i,
    pub dir: Option<Dir8>,
    pub kind: PosChangeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PosChangeKind {
    Walk,
    Teleport,
}

/// Changes entity's direction
#[derive(Debug, Clone)]
pub struct DirChange {
    pub ent: Index<EntityModel>,
    pub dir: Dir8,
    pub kind: DirChangeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DirChangeKind {
    Immediate,
    Smooth,
}

/// Change to the game world by a closure
pub struct OpaqueChange {
    pub proc: Box<dyn Fn(&mut Model)>,
}

impl fmt::Debug for OpaqueChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpaqueChange").finish()
    }
}
