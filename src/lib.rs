#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::struct_field_names)]

extern crate alloc;

mod error;
mod event;
pub mod pattern;
mod range;
pub mod series;

use core::ops::{Bound, Range, RangeBounds, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
pub use error::Error;
pub use event::Event;
use jiff::civil::{Date, DateTime, time};
use jiff::{ToSpan, Zoned};
use pattern::Combined;
pub use range::DateTimeRange;
#[doc(inline)]
pub use series::Series;

mod private {
    pub trait Sealed {}
}

/// A trait for recurrence patterns.
///
/// Values implementing `Pattern` are passed to [`Series::new`][Series::new] or [`Series::try_new`]
/// to build a new series of recurring events.
///
/// Since values implementing this trait must uphold some invariants to ensure correctness it is
/// sealed to prevent implementing it outside of this crate.
///
/// There is usually no need to interact with this trait directly. Use the functionality provided
/// by [`Series`] instead because it is more convenient.
///
/// The [`pattern`] module contains implementations of various recurrence patterns.
pub trait Pattern: private::Sealed + Clone {
    /// Find the next `DateTime` after `instant` within a range.
    ///
    /// This must always returns a datetime that is strictly larger than `instant` or `None` if
    /// the next event would be greater or equal to the range's end. If `instant` happens before
    /// the range's start, this must return the first event within the range.
    fn next_after(&self, instant: DateTime, range: DateTimeRange) -> Option<DateTime>;

    /// Find the previous `DateTime` before `instant` within a range.
    ///
    /// This must always returns a datetime that is strictly smaller than `instant` or `None` if
    /// the previous event would be less than the range's start. If `instant` happens after
    /// the range's end, this must return the last event within the range.
    fn previous_before(&self, instant: DateTime, range: DateTimeRange) -> Option<DateTime>;

    /// Find a `DateTime` closest to `instant` within a range.
    ///
    /// The returned datetime may happen before, after and exactly at `instant`. This must only
    /// return `None` if there is no event within the range.
    fn closest_to(&self, instant: DateTime, range: DateTimeRange) -> Option<DateTime>;
}

/// A trait for combining values implementing [`Pattern`] into more complex recurrence patterns.
///
/// This trait has a blanket implementation for any type implementing `Pattern`.
///
/// # Example
///
/// ```
/// use recurring::{Combine, pattern::cron};
///
/// let daily_at_noon = cron().hour(12).minute(0).second(0);
/// let daily_at_midnight = cron().hour(0).minute(0).second(0);
/// let first_of_month_at_six = cron().day(1).hour(6).minute(0).second(0);
///
/// let combined = daily_at_noon
///     .and(daily_at_midnight)
///     .and(first_of_month_at_six);
/// ```
pub trait Combine: Pattern + Sized {
    /// Combine `Self` with another `Pattern`.
    ///
    /// This allows building more complex recurrence patterns.
    ///
    /// See the documentation of the [`Combine`] trait for usage examples.
    #[must_use]
    fn and<P: Pattern>(self, other: P) -> Combined<Self, P> {
        Combined::new(self, other)
    }
}

impl<T: Pattern> Combine for T {}

/// A trait for converting values representing points in time into a [`Series`].
pub trait ToSeries {
    /// Converts a value to a [`Series`] with the given recurrence [`Pattern`].
    ///
    /// # Errors
    ///
    /// Returns an error if the value cannot be converted into a valid `Series`.
    fn to_series<P: Pattern>(&self, pattern: P) -> Result<Series<P>, Error>;
}

impl ToSeries for Event {
    /// Converts an `Event` to a `Series` with the given recurrence [`Pattern`].
    ///
    /// # Errors
    ///
    /// Returns an error if the event duration cannot be represented as a [`Span`][jiff::Span] or
    /// if the events' `start` is `DateTime::MAX`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::{Event, ToSeries, pattern::hourly};
    ///
    /// let date = date(2025, 1, 1);
    /// let start = date.at(0, 0, 0, 0);
    /// let end = date.at(0, 30, 0, 0);
    ///
    /// let event = Event::new(start, end);
    /// let series = event.to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::new(date.at(0, 0, 0, 0), date.at(0, 30, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::new(date.at(2, 0, 0, 0), date.at(2, 30, 0, 0))));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    fn to_series<P: Pattern>(&self, pattern: P) -> Result<Series<P>, Error> {
        Series::builder(self.start().., pattern)
            .event_duration(self.duration())
            .build()
    }
}

impl ToSeries for DateTime {
    /// Converts a `DateTime` to a `Series` with the given recurrence [`Pattern`].
    ///
    /// # Errors
    ///
    /// Returns an error if the datetime is `DateTime::MAX`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::datetime;
    /// use recurring::{Event, ToSeries, pattern::hourly};
    ///
    /// let series = datetime(2025, 1, 1, 0, 0, 0, 0).to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(datetime(2025, 1, 1, 2, 0, 0, 0))));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    fn to_series<P: Pattern>(&self, pattern: P) -> Result<Series<P>, Error> {
        Series::try_new(*self.., pattern)
    }
}

impl ToSeries for Date {
    /// Converts a `Date` to a `Series` with the given recurrence [`Pattern`].
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
    /// use jiff::civil::date;
    /// use recurring::{Event, ToSeries, pattern::hourly};
    ///
    /// let series = date(2025, 1, 1).to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(2, 0, 0, 0))));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    fn to_series<P: Pattern>(&self, pattern: P) -> Result<Series<P>, Error> {
        self.to_datetime(time(0, 0, 0, 0)).to_series(pattern)
    }
}

impl ToSeries for Zoned {
    /// Converts a `Date` to a `Series` with the given recurrence [`Pattern`].
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
    /// use jiff::{Zoned, civil::date};
    /// use recurring::{Event, ToSeries, pattern::hourly};
    ///
    /// let zoned: Zoned = "2025-01-01 12:22[Europe/Berlin]".parse()?;
    ///
    /// let series = zoned.to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(12, 22, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(14, 22, 0, 0))));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    fn to_series<P: Pattern>(&self, pattern: P) -> Result<Series<P>, Error> {
        self.datetime().to_series(pattern)
    }
}

macro_rules! impl_range_to_series {
    ($($(#[doc = $doc:expr])+ $ty:ty,)+) => {
        $(
            impl ToSeries for $ty {
                /// Converts a `
                #[doc = stringify!($ty)]
                /// ` to a `Series` with the given recurrence [`Pattern`].
                ///
                /// # Example
                ///
                $(#[doc = $doc])*
                fn to_series<P: Pattern>(&self, pattern: P) -> Result<Series<P>, Error> {
                    Series::try_new(self.clone(), pattern)
                }
            }
        )*
    };
}

impl_range_to_series!(
    /// ```
    /// use jiff::civil::date;
    /// use recurring::{Event, ToSeries, pattern::hourly};
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let end = date(2025, 1, 1).at(4, 0, 0, 0);
    /// let series = (start..end).to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(2, 0, 0, 0))));
    /// assert_eq!(events.next(), None);
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    Range<DateTime>,
    /// ```
    /// use recurring::{Event, ToSeries, pattern::hourly};
    /// use jiff::civil::date;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let series = (start..).to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(2, 0, 0, 0))));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    RangeFrom<DateTime>,
    /// ```
    /// use recurring::{Event, ToSeries, pattern::hourly};
    /// use jiff::civil::date;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let end = date(2025, 1, 1).at(4, 0, 0, 0);
    /// let series = (start..=end).to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(2, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(4, 0, 0, 0))));
    /// assert_eq!(events.next(), None);
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    RangeInclusive<DateTime>,
    /// ```
    /// use recurring::{Event, ToSeries, pattern::hourly};
    /// use jiff::civil::{DateTime, date};
    ///
    /// let end = DateTime::MAX;
    /// let series = (..end).to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(date(-9999, 1, 1).at(0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(-9999, 1, 1).at(2, 0, 0, 0))));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    RangeTo<DateTime>,
    /// ```
    /// use recurring::{Event, ToSeries, pattern::hourly};
    /// use jiff::civil::date;
    ///
    /// let end = date(2025, 1, 1).at(4, 0, 0, 0);
    /// let series = (..=end).to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(date(-9999, 1, 1).at(0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(-9999, 1, 1).at(2, 0, 0, 0))));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    RangeToInclusive<DateTime>,
    /// ```
    /// use recurring::{Event, ToSeries, pattern::hourly};
    /// use jiff::civil::date;
    /// use core::ops::Bound;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let end = date(2025, 1, 1).at(4, 0, 0, 0);
    /// let series = (Bound::Included(start), Bound::Excluded(end)).to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(2, 0, 0, 0))));
    /// assert_eq!(events.next(), None);
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    (Bound<DateTime>, Bound<DateTime>),
);

/// @TODO(mohmann): replace with `core::ops::IntoBounds` once it's stable.
trait IntoBounds<T> {
    fn into_bounds(self) -> (Bound<T>, Bound<T>);
}

impl<B: RangeBounds<T>, T: Clone> IntoBounds<T> for B {
    fn into_bounds(self) -> (Bound<T>, Bound<T>) {
        (self.start_bound().cloned(), self.end_bound().cloned())
    }
}

// Tries to simplify arbitrary range bounds into a `DateTimeRange`.
fn try_simplify_range<B: RangeBounds<DateTime>>(bounds: B) -> Result<DateTimeRange, Error> {
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

    Ok(DateTimeRange::new(start, end))
}
