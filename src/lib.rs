#![allow(missing_docs)] // @TODO(mohmann): enable warnings once API is fleshed out.
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

mod event;
pub mod repeat;
mod series;

pub use self::event::Event;
pub use self::series::{Iter, Series};
