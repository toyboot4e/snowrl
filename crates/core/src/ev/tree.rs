/*!
Event tree
*/

use std::ops::Deref;

use crate::ev::Event;

/// [`EventTree`] builder
#[derive(Debug)]
pub struct EventBuilder {
    root: EventTree,
}

impl Default for EventBuilder {
    fn default() -> Self {
        Self {
            root: EventTree {
                elems: Default::default(),
            },
        }
    }
}

impl EventBuilder {
    pub fn begin_node(&mut self, node: Node) {
        //
    }

    pub fn checkpoint(&self) -> CheckPoint {
        CheckPoint(self.root.elems.len() as u32)
    }

    pub fn start_node_at(&self) -> CheckPoint {
        CheckPoint(self.root.elems.len() as u32)
    }

    pub fn is_empty(&self) -> bool {
        self.root.elems.is_empty()
    }

    pub fn build(self) -> EventTree {
        self.root
    }
}

/// Root of event tree
#[derive(Debug, Default)]
pub struct EventTree {
    elems: Vec<Elem>,
}

impl EventTree {
    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }
}

/// Subtree of [`EventTree`]
#[derive(Debug)]
pub struct Node {
    children: Vec<Token>,
}

/// Node | Token
#[derive(Debug)]
pub enum Elem {
    Node(Node),
    Token(Token),
}

#[derive(Debug)]
pub struct Token {
    ev: Box<dyn Event>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CheckPoint(u32);

/// Event builder + read-only access to model
#[derive(Debug)]
pub struct EventSystem<'a, M> {
    builder: EventBuilder,
    model: &'a mut M,
}

impl<'a, M> EventSystem<'a, M> {
    pub fn new(model: &'a mut M) -> Self {
        Self {
            builder: EventBuilder::default(),
            model,
        }
    }
}

impl<'a, M> Deref for EventSystem<'a, M> {
    type Target = M;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}
