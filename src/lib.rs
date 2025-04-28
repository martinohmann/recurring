#![no_std]
#![allow(missing_docs)] // @TODO(mohmann): enable warnings once API is fleshed out.
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

extern crate alloc;

mod error;
mod event;
pub mod repeat;
mod series;

use core::ops::Range;

pub use self::error::Error;
pub use self::event::Event;
pub use self::series::{Iter, Series, SeriesWith};
use jiff::civil::DateTime;

pub trait Repeat {
    fn next_event(&self, instant: DateTime) -> Option<DateTime>;

    fn previous_event(&self, instant: DateTime) -> Option<DateTime>;

    fn is_aligned_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> bool;

    fn align_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime>;
}
