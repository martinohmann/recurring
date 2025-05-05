use crate::{Error, Event, IntoBounds, Repeat, error::ErrorKind, try_simplify_range};
use core::ops::{Bound, Range, RangeBounds};
use jiff::{Span, civil::DateTime};

/// A series of recurring events.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// use jiff::civil::date;
/// use recurring::{Event, Series};
/// use recurring::repeat::hourly;
///
/// let start = date(2025, 1, 1).at(0, 0, 0, 0);
/// let end = date(2025, 1, 1).at(4, 0, 0, 0);
///
/// let series = Series::new(start..end, hourly(2));
///
/// let mut events = series.iter();
///
/// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
/// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(2, 0, 0, 0))));
/// assert_eq!(events.next(), None);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Series<R> {
    repeat: R,
    range: Range<DateTime>,
    event_duration: Span,
}

impl<R> Series<R>
where
    R: Repeat,
{
    /// Creates a new `Series` that produces events within the provided `range` according to the
    /// given `repeat` interval.
    ///
    /// To configure more aspects of the series call `.with()` on the constructed
    /// `Series` value. See the documentation of [`Series::with`] for more details.
    ///
    /// For a fallible alternative see [`Series::try_new`].
    ///
    /// # Panics
    ///
    /// Panics if the start or end of the range bounds would overflow `DateTime::MAX` after
    /// normalization or if `start` >= `end`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// # use recurring::Series;
    /// use recurring::repeat::hourly;
    ///
    /// let series = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2));
    /// ```
    pub fn new<B: RangeBounds<DateTime>>(range: B, repeat: R) -> Series<R> {
        let range = try_simplify_range(range).expect("range overflows DateTime::MAX");

        assert!(
            range.start < range.end,
            "invalid bounds: end must be greater than start"
        );

        Series {
            repeat,
            range,
            event_duration: Span::new(),
        }
    }

    /// Creates a new `Series` that produces events within the provided `range` according to the
    /// given `repeat` interval.
    ///
    /// To configure more aspects of the series call `.with()` on the constructed
    /// `Series` value. See the documentation of [`Series::with`] for more details.
    ///
    /// For an infallible alternative that panics instead see [`Series::new`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the start or end of the range bounds would overflow `DateTime::MAX`
    /// after normalization or if `start` >= `end`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::civil::{DateTime, date};
    /// # use recurring::Series;
    /// use recurring::repeat::hourly;
    ///
    /// assert!(Series::try_new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2)).is_ok());
    /// assert!(Series::try_new(DateTime::MAX.., hourly(2)).is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_new<B: RangeBounds<DateTime>>(range: B, repeat: R) -> Result<Series<R>, Error> {
        let range = try_simplify_range(range)?;
        if range.start >= range.end {
            return Err(Error::from(ErrorKind::InvalidBounds));
        }

        Ok(Series {
            repeat,
            range,
            event_duration: Span::new(),
        })
    }

    /// Creates a builder for constructing a new `Series` from the fields of this series.
    ///
    /// # Example
    ///
    /// Set an explict end date for the series and configure the duration of the individual events.
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # use recurring::Series;
    /// use jiff::ToSpan;
    /// use jiff::civil::date;
    /// use recurring::repeat::daily;
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let s2 = s1.with()
    ///     .end(date(2025, 2, 1).at(0, 0, 0, 0))
    ///     .event_duration(1.hour())
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with(self) -> SeriesWith<R> {
        SeriesWith::new(self)
    }

    /// Returns the range from series start (inclusive) to series end (exclusive).
    pub fn range(&self) -> &Range<DateTime> {
        &self.range
    }

    /// Returns the `DateTime` at which the series starts (inclusive).
    ///
    /// This is not necessarily the time of the first event in the series.
    pub fn start(&self) -> DateTime {
        self.range.start
    }

    /// Returns the `DateTime` at which the series ends (exclusive).
    ///
    /// Don't confuse this with the time of the last event in the series. It is merely an upper
    /// bound until which the series will yield events.
    pub fn end(&self) -> DateTime {
        self.range.end
    }

    /// Returns the duration of individual events in the series.
    ///
    /// If this is zero, events will not have an end date.
    pub fn event_duration(&self) -> Span {
        self.event_duration
    }

    /// Returns a reference to the `Repeat` used by the series to generate events.
    pub fn repeat(&self) -> &R {
        &self.repeat
    }

    /// Creates an iterator over the events in a the series.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::{Event, Series};
    /// use recurring::repeat::hourly;
    ///
    /// let series = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2));
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(2, 0, 0, 0))));
    /// ```
    pub fn iter(&self) -> Iter<'_, R> {
        Iter::new(self)
    }

    /// Gets the first event in the series.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::{Event, Series};
    /// use recurring::repeat::hourly;
    ///
    /// let series = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2));
    ///
    /// assert_eq!(series.first_event(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// ```
    pub fn first_event(&self) -> Option<Event> {
        self.get_event(self.range.start)
            .or_else(|| self.get_event_after(self.range.start))
    }

    /// Gets the last event in the series.
    ///
    /// If the series does not have an end, this method will return an event close to
    /// `DateTime::MAX`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::civil::date;
    /// use recurring::{Event, Series};
    /// use recurring::repeat::hourly;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let end = date(2026, 1, 1).at(0, 0, 0, 0);
    ///
    /// let series = Series::new(start..end, hourly(2));
    ///
    /// assert_eq!(series.last_event(), Some(Event::at(date(2025, 12, 31).at(22, 0, 0, 0))));
    /// # Ok(())
    /// # }
    /// ```
    pub fn last_event(&self) -> Option<Event> {
        self.get_event_before(self.range.end)
    }

    /// Returns `true` when the series contains an event starting at `instant`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// # use recurring::Series;
    /// use recurring::repeat::hourly;
    ///
    /// let series = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2));
    ///
    /// assert!(!series.contains_event(date(2025, 1, 1).at(0, 35, 0, 0)));
    /// assert!(series.contains_event(date(2025, 2, 10).at(12, 0, 0, 0)));
    /// ```
    pub fn contains_event(&self, instant: DateTime) -> bool {
        self.get_event(instant).is_some()
    }

    /// Gets an event in the series.
    ///
    /// Returns `Some(_)` if there's an event starting at `instant`, otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::{Event, Series};
    /// use recurring::repeat::hourly;
    ///
    /// let series = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2));
    ///
    /// assert!(series.get_event(date(2025, 1, 1).at(1, 0, 0, 0)).is_none());
    /// assert!(series.get_event(date(2026, 12, 31).at(14, 0, 0, 0)).is_some());
    /// ```
    pub fn get_event(&self, instant: DateTime) -> Option<Event> {
        let closest = self.repeat.closest_event(instant, &self.range)?;
        if closest == instant {
            return self.get_event_unchecked(instant);
        }

        None
    }

    /// Gets the event containing `instant`.
    ///
    /// Returns `None` if there's no event in the series that start at `instant` or contains it (if
    /// series events have a duration).
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::ToSpan;
    /// use jiff::civil::{date, DateTime};
    /// use recurring::{Event, Series};
    /// use recurring::repeat::hourly;
    ///
    /// let series_start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let series_end = date(2025, 2, 1).at(0, 0, 0, 0);
    /// let series = Series::new(series_start..series_end, hourly(1))
    ///     .with()
    ///     .event_duration(30.minutes())
    ///     .build()?;
    ///
    /// assert_eq!(
    ///     series.get_event_containing(series_start - 1.minute()),
    ///     None,
    /// );
    /// assert_eq!(
    ///     series.get_event_containing(series_start),
    ///     Some(Event::new(series_start, series_start + 30.minutes())?),
    /// );
    /// assert_eq!(
    ///     series.get_event_containing(series_start + 31.minutes()),
    ///     None,
    /// );
    /// assert_eq!(
    ///     series.get_event_containing(series_start + 1.hour().minutes(20)),
    ///     Some(Event::new(series_start + 1.hour(), series_start + 1.hour().minutes(30))?),
    /// );
    /// assert_eq!(
    ///     series.get_event_containing(series_end),
    ///     None,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_event_containing(&self, instant: DateTime) -> Option<Event> {
        self.get_event(instant)
            .or_else(|| self.get_event_before(instant))
            .filter(|event| event.contains(instant))
    }

    /// Gets the next event after `instant`.
    ///
    /// Returns `None` if `instant` is close to the series end and there's no more event between
    /// `instant` and the series end.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::ToSpan;
    /// use jiff::civil::{date, DateTime};
    /// use recurring::{Event, Series};
    /// use recurring::repeat::hourly;
    ///
    /// let series_start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let series_end = date(2025, 2, 1).at(0, 0, 0, 0);
    /// let series = Series::new(series_start..series_end, hourly(1));
    ///
    /// assert_eq!(
    ///     series.get_event_after(series_start - 1.minute()),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_event_after(series_start),
    ///     Some(Event::at(series_start + 1.hour())),
    /// );
    /// assert_eq!(
    ///     series.get_event_after(series_start + 1.minute()),
    ///     Some(Event::at(series_start + 1.hour())),
    /// );
    /// assert_eq!(
    ///     series.get_event_after(series_end - 1.hour().minutes(1)),
    ///     Some(Event::at(series_end - 1.hour())),
    /// );
    /// assert_eq!(
    ///     series.get_event_after(series_end - 1.hour()),
    ///     None,
    /// );
    /// assert_eq!(
    ///     series.get_event_after(series_end),
    ///     None,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_event_after(&self, instant: DateTime) -> Option<Event> {
        let closest = self.repeat.closest_event(instant, &self.range)?;
        if closest > instant {
            return self.get_event_unchecked(closest);
        }

        self.repeat
            .next_event(closest)
            .filter(|next| self.range.contains(next))
            .and_then(|next| self.get_event_unchecked(next))
    }

    /// Gets the previous event before `instant`.
    ///
    /// Returns `None` if `instant` is less than or equal to the series start.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::ToSpan;
    /// use jiff::civil::{date, DateTime};
    /// use recurring::{Event, Series};
    /// use recurring::repeat::hourly;
    ///
    /// let series_start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let series = Series::new(series_start.., hourly(1));
    ///
    /// assert_eq!(series.get_event_before(series_start), None);
    /// assert_eq!(series.get_event_before(series_start - 1.minute()), None);
    /// assert_eq!(
    ///     series.get_event_before(series_start + 29.minute()),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_event_before(series_start + 1.hour()),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_event_before(series_start + 1.hour().seconds(1)),
    ///     Some(Event::at(series_start + 1.hour())),
    /// );
    /// assert_eq!(
    ///     series.get_event_before(DateTime::MAX),
    ///     Some(Event::at(date(9999, 12, 31).at(23, 0, 0, 0))),
    /// );
    /// ```
    pub fn get_event_before(&self, instant: DateTime) -> Option<Event> {
        let closest = self.repeat.closest_event(instant, &self.range)?;
        if closest < instant {
            return self.get_event_unchecked(closest);
        }

        self.repeat
            .previous_event(closest)
            .filter(|previous| self.range.contains(previous))
            .and_then(|previous| self.get_event_unchecked(previous))
    }

    /// Gets the series event with the start time closest to `instant`.
    ///
    /// The returned event may start before, at or after `instant`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::ToSpan;
    /// use jiff::civil::date;
    /// use recurring::{Event, Series};
    /// use recurring::repeat::hourly;
    ///
    /// let series_start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let series = Series::new(series_start.., hourly(1));
    ///
    /// assert_eq!(
    ///     series.get_closest_event(series_start),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_closest_event(series_start - 1.minute()),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_closest_event(series_start + 29.minutes()),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_closest_event(series_start + 30.minutes()),
    ///     Some(Event::at(series_start + 1.hour())),
    /// );
    /// ```
    pub fn get_closest_event(&self, instant: DateTime) -> Option<Event> {
        self.repeat
            .closest_event(instant, &self.range)
            .and_then(|closest| self.get_event_unchecked(closest))
    }

    /// Get an event without any bound checks. The datetime at `start` is assumed to be aligned to
    /// the series and within the series start and end bounds.
    #[inline]
    fn get_event_unchecked(&self, start: DateTime) -> Option<Event> {
        if self.event_duration.is_positive() {
            let end = start.checked_add(self.event_duration).ok()?;
            Event::new(start, end).ok()
        } else {
            Some(Event::at(start))
        }
    }
}

impl<'a, R> IntoIterator for &'a Series<R>
where
    R: Repeat,
{
    type Item = Event;
    type IntoIter = Iter<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A builder for [`Series`] values.
///
/// Values of this type are produced by [`Series::with`]. `SeriesWith` can be
/// materialized into a `Series` by calling its [`.build()`](SeriesWith::build) method.
///
/// The builder allows to configure the following optional parameters for a `Series`:
///
/// - `start`: The datetime at which the series starts. This is not necessarily identical to the
///   start of the first event in the series.
/// - `end`: The datetime at which the series ends. The default is [`DateTime::MAX`], which means
///   that it only ends when events become unrepresentable as `DateTime`.
/// - `event_duration`: The [`Span`] of an individual event in the series. This could be minutes,
///   hours, days or any other duration the `Span` type supports. If `event_duration` is not set,
///   individual events will not have an end datetime and have an effective duration of zero.
/// - `repeat`: The repeat interval for the series.
#[derive(Debug, Clone)]
pub struct SeriesWith<R> {
    repeat: R,
    bounds: (Bound<DateTime>, Bound<DateTime>),
    event_duration: Span,
}

impl<R> SeriesWith<R>
where
    R: Repeat,
{
    /// Creates an new `SeriesWith` from a `Series`.
    fn new(series: Series<R>) -> SeriesWith<R> {
        SeriesWith {
            repeat: series.repeat,
            bounds: series.range.into_bounds(),
            event_duration: series.event_duration,
        }
    }

    /// Sets the range of the series.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::{ToSpan};
    /// use jiff::civil::date;
    /// use recurring::Series;
    /// use recurring::repeat::daily;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    ///
    /// let s1 = Series::new(start.., daily(1));
    ///
    /// let new_start = s1.start() + 1.day();
    /// let new_end = new_start + 1.month();
    ///
    /// // The new range includes `new_end`.
    /// let s2 = s1.with().range(new_start..=new_end).build()?;
    ///
    /// assert_eq!(s2.start(), new_start);
    ///
    /// // The since `new_end` is included in the series, it actually ends 1ns after that!
    /// assert_eq!(s2.end(), new_end + 1.nanosecond());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn range<B: RangeBounds<DateTime>>(mut self, range: B) -> SeriesWith<R> {
        self.bounds = range.into_bounds();
        self
    }

    /// Sets the start of the series.
    ///
    /// The start is inclusive and can (but does not have to) mark the first event from the series.
    /// This depends on the period of the series.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::civil::date;
    /// use recurring::Series;
    /// use recurring::repeat::daily;
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let start = date(2025, 2, 1).at(0, 0, 0, 0);
    ///
    /// let s2 = s1.with().start(start).build()?;
    ///
    /// assert_eq!(s2.start(), start);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn start(mut self, start: DateTime) -> SeriesWith<R> {
        self.bounds.0 = Bound::Included(start);
        self
    }

    /// Sets the end of the series.
    ///
    /// The end is exclusive and will never be returned as an event from `Series`' iterator and its
    /// various lookup methods. If you need to set the `end` inclusively, consider using a
    /// [`.range(start..=end)`][SeriesWith::range] instead.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::civil::date;
    /// use recurring::Series;
    /// use recurring::repeat::daily;
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let end = date(2025, 2, 1).at(0, 0, 0, 0);
    ///
    /// let s2 = s1.with().end(end).build()?;
    ///
    /// assert_eq!(s2.end(), end);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn end(mut self, end: DateTime) -> SeriesWith<R> {
        self.bounds.1 = Bound::Excluded(end);
        self
    }

    /// Sets the duration of individual events in the series.
    ///
    /// If `.event_duration()` is not called with a custom value, events will not have an end
    /// datetime.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::ToSpan;
    /// use jiff::civil::date;
    /// use recurring::Series;
    /// use recurring::repeat::daily;
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let s2 = s1.with().event_duration(1.hour()).build()?;
    ///
    /// assert_eq!(s2.event_duration().fieldwise(), 1.hour());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn event_duration(mut self, event_duration: Span) -> SeriesWith<R> {
        self.event_duration = event_duration;
        self
    }

    /// Sets the repeat interval for the series.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::ToSpan;
    /// use jiff::civil::date;
    /// use recurring::Series;
    /// use recurring::repeat::daily;
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let s2 = s1.with().repeat(daily(2)).build()?;
    ///
    /// assert_eq!(s2.repeat(), &daily(2));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn repeat<S: Repeat>(self, repeat: S) -> SeriesWith<S> {
        SeriesWith {
            repeat,
            bounds: self.bounds,
            event_duration: self.event_duration,
        }
    }

    /// Builds a [`Series`] that yields events according to the provided [`Repeat`] implementation.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the configured `end` is less than or equal to `start`, or if the
    /// configured `event_duration` is negative.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # use recurring::Series;
    /// use jiff::civil::date;
    /// use recurring::repeat::daily;
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let s2 = s1.with()
    ///     .end(date(2025, 1, 3).at(0, 0, 0, 0))
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Result<Series<R>, Error> {
        let range = try_simplify_range(self.bounds)?;
        if range.start >= range.end {
            return Err(Error::from(ErrorKind::InvalidBounds));
        }

        if self.event_duration.is_negative() {
            return Err(Error::from(ErrorKind::InvalidEventDuration));
        }

        Ok(Series {
            repeat: self.repeat,
            range,
            event_duration: self.event_duration,
        })
    }
}

/// An iterator over the events of a [`Series`].
///
/// This struct is created by the [`.iter()`][Series::iter] method of a `Series`. See its
/// documentation for more.
#[derive(Debug, Clone)]
pub struct Iter<'a, R> {
    series: &'a Series<R>,
    next_start: Option<DateTime>,
}

impl<'a, R> Iter<'a, R> {
    fn new(series: &'a Series<R>) -> Iter<'a, R> {
        Iter {
            series,
            next_start: Some(series.range.start),
        }
    }
}

impl<R> Iterator for Iter<'_, R>
where
    R: Repeat,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let series = &self.series;
        let start = self.next_start?;

        if !series.range.contains(&start) {
            return None;
        }

        self.next_start = series.repeat.next_event(start);

        // Handle the case where the series start does not fall into the desired frequency and
        // skip over to the next event right away.
        if start == series.range.start && !series.contains_event(start) {
            return self.next();
        }

        series.get_event_unchecked(start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repeat::{daily, hourly};
    use alloc::vec;
    use alloc::vec::Vec;
    use jiff::{
        ToSpan,
        civil::{date, datetime, time},
    };

    #[test]
    fn series_range() {
        let series = Series::new(.., daily(1));
        assert_eq!(series.start(), DateTime::MIN);
        assert_eq!(series.end(), DateTime::MAX);

        let start = date(2025, 1, 1).at(0, 0, 0, 0);
        let end = date(2025, 2, 1).at(0, 0, 0, 0);

        let series = Series::new(start..end, daily(1));
        assert_eq!(series.start(), start);
        assert_eq!(series.end(), end);

        let series = Series::new(..=end, daily(1));
        assert_eq!(series.start(), DateTime::MIN);
        assert_eq!(series.end(), end + 1.nanosecond());
    }

    #[test]
    fn daily_series() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let series = Series::new(start.., daily(2));
        let events: Vec<_> = series.iter().take(5).collect();
        let expected = vec![
            Event::at(datetime(2025, 1, 1, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 3, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 5, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 7, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 9, 1, 1, 1, 0)),
        ];
        assert_eq!(events, expected);
    }

    #[test]
    fn daily_series_at() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let series = Series::new(start.., daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3)));
        let events: Vec<_> = series.iter().take(5).collect();
        let expected = vec![
            Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)),
            Event::at(datetime(2025, 1, 1, 3, 3, 3, 3)),
            Event::at(datetime(2025, 1, 3, 2, 2, 2, 2)),
            Event::at(datetime(2025, 1, 3, 3, 3, 3, 3)),
            Event::at(datetime(2025, 1, 5, 2, 2, 2, 2)),
        ];
        assert_eq!(events, expected);
    }

    #[test]
    fn daily_series_with_end() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 5, 1, 1, 1, 0);
        let series = Series::new(start..end, daily(2)).with().build().unwrap();
        let events: Vec<_> = series.iter().collect();
        let expected = vec![
            Event::at(datetime(2025, 1, 1, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 3, 1, 1, 1, 0)),
        ];
        assert_eq!(events, expected);
    }

    #[test]
    fn daily_series_with_end_and_duration() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 5, 1, 1, 1, 0);
        let series = Series::new(start..end, daily(2))
            .with()
            .event_duration(1.hour())
            .build()
            .unwrap();

        let events: Vec<_> = series.iter().collect();
        let expected = vec![
            Event::new(
                datetime(2025, 1, 1, 1, 1, 1, 0),
                datetime(2025, 1, 1, 2, 1, 1, 0),
            )
            .unwrap(),
            Event::new(
                datetime(2025, 1, 3, 1, 1, 1, 0),
                datetime(2025, 1, 3, 2, 1, 1, 0),
            )
            .unwrap(),
        ];
        assert_eq!(events, expected);
    }

    #[test]
    fn series_contains_event() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let series = Series::new(start.., daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3)));
        assert!(!series.contains_event(datetime(2025, 1, 1, 1, 1, 1, 0)));
        assert!(series.contains_event(datetime(2025, 1, 1, 2, 2, 2, 2)));
        assert!(series.contains_event(datetime(2025, 1, 1, 3, 3, 3, 3)));
        assert!(!series.contains_event(datetime(2025, 1, 1, 2, 2, 2, 3)));
        assert!(!series.contains_event(datetime(2025, 1, 1, 3, 3, 3, 2)));
    }

    #[test]
    fn series_relative_events() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 3, 1, 1, 1, 0);
        let series = Series::new(
            start..end,
            daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3)),
        );
        assert_eq!(series.get_event(datetime(2025, 1, 1, 1, 1, 1, 0)), None);
        assert_eq!(
            series.get_event(datetime(2025, 1, 1, 2, 2, 2, 2)),
            Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)))
        );

        assert_eq!(
            series.get_event_after(datetime(2025, 1, 1, 2, 2, 2, 2)),
            Some(Event::at(datetime(2025, 1, 1, 3, 3, 3, 3)))
        );

        assert_eq!(
            series.get_event_before(datetime(2025, 1, 1, 2, 2, 2, 2)),
            None
        );

        assert_eq!(
            series.get_event_before(datetime(2025, 1, 1, 3, 3, 3, 3)),
            Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2))),
        );
    }

    #[test]
    fn series_get_event_containing() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 3, 1, 1, 1, 0);
        let series = Series::new(start..end, daily(2).at(time(2, 2, 2, 2)));
        assert_eq!(
            series.get_event_containing(datetime(2025, 1, 1, 1, 1, 1, 0)),
            None
        );
        assert_eq!(
            series.get_event_containing(datetime(2025, 1, 1, 2, 2, 2, 2)),
            Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)))
        );
        assert_eq!(
            series.get_event_containing(datetime(2025, 1, 1, 2, 2, 2, 3)),
            None
        );

        let series = Series::new(start..end, daily(2).at(time(2, 2, 2, 2)))
            .with()
            .event_duration(1.hour())
            .build()
            .unwrap();

        assert_eq!(
            series.get_event_containing(datetime(2025, 1, 1, 2, 2, 2, 3)),
            Some(
                Event::new(
                    datetime(2025, 1, 1, 2, 2, 2, 2),
                    datetime(2025, 1, 1, 3, 2, 2, 2)
                )
                .unwrap()
            )
        );

        assert_eq!(
            series.get_event_containing(datetime(2025, 1, 1, 3, 2, 2, 1)),
            Some(
                Event::new(
                    datetime(2025, 1, 1, 2, 2, 2, 2),
                    datetime(2025, 1, 1, 3, 2, 2, 2)
                )
                .unwrap()
            )
        );

        assert_eq!(
            series.get_event_containing(datetime(2025, 1, 1, 3, 2, 2, 2)),
            None
        );
    }

    #[test]
    fn series_first_event() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 3, 1, 1, 1, 0);
        let series = Series::new(start..end, daily(2).at(time(2, 2, 2, 2)));
        assert_eq!(
            series.first_event(),
            Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)))
        );
    }

    #[test]
    fn series_last_event() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 10, 1, 1, 1, 0);
        let series = Series::new(start..end, daily(2).at(time(2, 2, 2, 2)));
        assert_eq!(
            series.last_event(),
            Some(Event::at(datetime(2025, 1, 9, 2, 2, 2, 2)))
        );

        let series = Series::new(start.., daily(2).at(time(2, 2, 2, 2)));
        assert_eq!(
            series.last_event(),
            Some(Event::at(datetime(9999, 12, 30, 2, 2, 2, 2)))
        );
    }

    #[test]
    fn series_last_event_unbounded() {
        let start = date(2025, 1, 1).at(1, 1, 1, 0);
        let series = Series::new(start.., hourly(2));
        assert_eq!(
            series.last_event(),
            Some(Event::at(date(9999, 12, 31).at(23, 1, 1, 0)))
        );
    }

    #[test]
    fn series_get_closest_event() {
        let start = datetime(2025, 1, 1, 0, 0, 0, 0);
        let series = Series::new(start.., hourly(1));

        assert_eq!(
            series.get_closest_event(datetime(2024, 12, 31, 0, 0, 0, 0)),
            Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
        );
        assert_eq!(
            series.get_closest_event(datetime(2025, 1, 1, 0, 0, 0, 0)),
            Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
        );
        assert_eq!(
            series.get_closest_event(datetime(2025, 1, 1, 0, 29, 0, 999)),
            Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
        );
        assert_eq!(
            series.get_closest_event(datetime(2025, 1, 1, 0, 30, 0, 0)),
            Some(Event::at(datetime(2025, 1, 1, 1, 0, 0, 0)))
        );
        assert_eq!(
            series.get_closest_event(DateTime::MIN),
            Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
        );
        assert_eq!(
            series.get_closest_event(DateTime::MAX),
            Some(Event::at(datetime(9999, 12, 31, 23, 0, 0, 0)))
        );
    }
}
