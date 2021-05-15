/*!
Passive data structures. They don't work to each other; they just update themselves.

The point is to make flat data structure. Agents that work on data is put in another module so that
[`Data`] doesn't have to nest big structs.

[`Data`]: ../Data
*/

pub mod res;
pub mod world;
