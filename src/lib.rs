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
use jiff::Zoned;
use jiff::civil::{Date, DateTime, time};

pub trait Repeat {
    fn next_event(&self, instant: DateTime) -> Option<DateTime>;

    fn previous_event(&self, instant: DateTime) -> Option<DateTime>;

    fn closest_event(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime>;
}

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
        let series = Series::try_new(self.start(), repeat)?;

        if let Some(duration) = self.duration() {
            series.with().event_duration(duration).build()
        } else {
            Ok(series)
        }
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
        Series::try_new(*self, repeat)
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
