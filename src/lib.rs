#![no_std]
#![allow(missing_docs)] // @TODO(mohmann): enable warnings once API is fleshed out.
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

extern crate alloc;

mod error;
mod event;
pub mod repeat;
mod series;

use core::ops::{Bound, Range, RangeBounds};
pub use error::Error;
pub use event::Event;
use jiff::civil::{Date, DateTime, time};
use jiff::{ToSpan, Zoned};
use repeat::Combined;
pub use series::{Iter, Series, SeriesWith};

mod private {
    pub trait Sealed {}
}

/// A trait for generating repeating events.
///
/// Values implementing `Repeat` are passed to [`Series::new`][Series::new] or [`Series::try_new`]
/// to build a new series of events that can be queried.
///
/// Since values implementing this trait must uphold some invariants to ensure correctness it is
/// sealed to prevent implementing it outside of this crate.
///
/// There is usually no need to interact with this trait directly. Use the functionality provided
/// by [`Series`] instead because it is more convenient.
pub trait Repeat: private::Sealed {
    /// Find the next `DateTime` after `instant` within a range.
    ///
    /// This must always returns a datetime that is strictly larger than `instant` or `None` if
    /// the next event would be greater or equal to the range's end. If `instant` happens before
    /// the range's start, this must return the first event within the range.
    fn next_after(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime>;

    /// Find the previous `DateTime` before `instant` within a range.
    ///
    /// This must always returns a datetime that is strictly smaller than `instant` or `None` if
    /// the previous event would be less than the range's start. If `instant` happens after
    /// the range's end, this must return the last event within the range.
    fn previous_before(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime>;

    /// Find a `DateTime` closest to `instant` within a range.
    ///
    /// The returned datetime may happen before, after and exactly at `instant`. This must only
    /// return `None` if there is no event within the range.
    fn closest_to(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime>;
}

/// A trait for combining values implementing [`Repeat`] into more complex pattern.
///
/// This trait is implemented for any type that implements `Repeat`.
///
/// # Example
///
/// ```
/// use recurring::Combine;
/// use recurring::repeat::spec;
///
/// let daily_at_noon = spec().hour(12).minute(0).second(0);
/// let daily_at_midnight = spec().hour(0).minute(0).second(0);
/// let first_of_month_at_six = spec().day(1).hour(6).minute(0).second(0);
///
/// let combined_repeat = daily_at_noon
///     .and(daily_at_midnight)
///     .and(first_of_month_at_six);
/// ```
pub trait Combine: Repeat + Sized {
    /// Combine `Self` with another `Repeat`.
    ///
    /// This allows for building more complex repeat pattern.
    ///
    /// See the documentation of the [`Combine`] trait for usage examples.
    #[must_use]
    fn and<R: Repeat>(self, other: R) -> Combined<Self, R> {
        Combined::new(self, other)
    }
}

impl<T: Repeat> Combine for T {}

/// A trait for converting values representing points in time into a [`Series`].
pub trait ToSeries {
    /// Converts a value to a [`Series`] with the given [`Repeat`] interval.
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be converted into a valid `Series`.
    fn to_series<R: Repeat>(&self, repeat: R) -> Result<Series<R>, Error>;
}

impl ToSeries for Event {
    /// Converts an `Event` to a `Series` with the given [`Repeat`] interval.
    ///
    /// # Errors
    ///
    /// Returns an error if the event duration cannot be represented as a [`Span`][jiff::Span] or
    /// if the events' `start` is `DateTime::MAX`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use recurring::{Event, ToSeries, repeat::hourly};
    /// use jiff::civil::date;
    ///
    /// let date = date(2025, 1, 1);
    /// let start = date.at(0, 0, 0, 0);
    /// let end = date.at(0, 30, 0, 0);
    ///
    /// let event = Event::new(start, end)?;
    /// let series = event.to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::new(date.at(0, 0, 0, 0), date.at(0, 30, 0, 0))?));
    /// assert_eq!(events.next(), Some(Event::new(date.at(2, 0, 0, 0), date.at(2, 30, 0, 0))?));
    /// # Ok(())
    /// # }
    /// ```
    fn to_series<R: Repeat>(&self, repeat: R) -> Result<Series<R>, Error> {
        self.start()
            .to_series(repeat)?
            .with()
            .event_duration(self.duration())
            .build()
    }
}

impl ToSeries for DateTime {
    /// Converts a `DateTime` to a `Series` with the given [`Repeat`] interval.
    ///
    /// # Errors
    ///
    /// Returns an error if the datetime is `DateTime::MAX`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use recurring::{Event, ToSeries, repeat::hourly};
    /// use jiff::civil::datetime;
    ///
    /// let series = datetime(2025, 1, 1, 0, 0, 0, 0).to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(datetime(2025, 1, 1, 2, 0, 0, 0))));
    /// # Ok(())
    /// # }
    /// ```
    fn to_series<R: Repeat>(&self, repeat: R) -> Result<Series<R>, Error> {
        Series::try_new(*self.., repeat)
    }
}

impl ToSeries for Date {
    /// Converts a `Date` to a `Series` with the given [`Repeat`] interval.
    ///
    /// The resulting series always starts at midnight on the date `to_series` is called on.
    ///
    /// # Errors
    ///
    /// This method does not fail for `Date`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use recurring::{Event, ToSeries, repeat::hourly};
    /// use jiff::civil::{date, datetime};
    ///
    /// let series = date(2025, 1, 1).to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(datetime(2025, 1, 1, 2, 0, 0, 0))));
    /// # Ok(())
    /// # }
    /// ```
    fn to_series<R: Repeat>(&self, repeat: R) -> Result<Series<R>, Error> {
        self.to_datetime(time(0, 0, 0, 0)).to_series(repeat)
    }
}

impl ToSeries for Zoned {
    /// Converts a `Date` to a `Series` with the given [`Repeat`] interval.
    ///
    /// The resulting series always starts at midnight on the date `to_series` is called on.
    ///
    /// # Errors
    ///
    /// Returns an error if the `Zoned`'s datetime is `DateTime::MAX`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use recurring::{Event, ToSeries, repeat::hourly};
    /// use jiff::{Zoned, civil::datetime};
    ///
    /// let zoned: Zoned = "2025-01-01 12:22[Europe/Berlin]".parse()?;
    ///
    /// let series = zoned.to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(datetime(2025, 1, 1, 12, 22, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(datetime(2025, 1, 1, 14, 22, 0, 0))));
    /// # Ok(())
    /// # }
    /// ```
    fn to_series<R: Repeat>(&self, repeat: R) -> Result<Series<R>, Error> {
        self.datetime().to_series(repeat)
    }
}

/// @TODO(mohmann): replace with `core::ops::IntoBounds` once it's stable.
trait IntoBounds<T> {
    fn into_bounds(self) -> (Bound<T>, Bound<T>);
}

impl<B: RangeBounds<T>, T: Clone> IntoBounds<T> for B {
    fn into_bounds(self) -> (Bound<T>, Bound<T>) {
        (self.start_bound().cloned(), self.end_bound().cloned())
    }
}

// Tries to simplify arbitrary range bounds into a `Range<DateTime>`.
fn try_simplify_range<B: RangeBounds<DateTime>>(bounds: B) -> Result<Range<DateTime>, Error> {
    let start = match bounds.start_bound() {
        Bound::Unbounded => DateTime::MIN,
        Bound::Included(start) => *start,
        Bound::Excluded(start) => start.checked_add(1.nanosecond())?,
    };

    let end = match bounds.end_bound() {
        Bound::Unbounded => DateTime::MAX,
        Bound::Included(end) => end.checked_add(1.nanosecond())?,
        Bound::Excluded(end) => *end,
    };

    Ok(Range { start, end })
}
