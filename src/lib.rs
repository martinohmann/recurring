#![allow(missing_docs)] // @TODO(mohmann): enable warnings once API is fleshed out.
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

mod event;
pub mod repeat;
mod series;

pub use self::event::Event;
pub use self::series::{Iter, Series};
use jiff::civil::DateTime;

pub trait Repeat {
    fn next_event(&self, instant: DateTime) -> Option<DateTime>;

    fn previous_event(&self, instant: DateTime) -> Option<DateTime>;

    fn aligns_with_series(&self, instant: DateTime, series_start: DateTime) -> bool;

    fn align_to_series(&self, instant: DateTime, series_start: DateTime) -> Option<DateTime>;
}
