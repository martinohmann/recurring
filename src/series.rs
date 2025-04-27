use crate::{Error, Event, Repeat};
use core::ops::Range;
use jiff::{Span, Zoned, civil::DateTime};

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
/// let series = Series::builder()
///     .start(date(2025, 1, 1).at(0, 0, 0, 0))
///     .end(date(2025, 1, 1).at(4, 0, 0, 0))
///     .build(hourly(2))?;
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
    bounds: Range<DateTime>,
    event_duration: Span,
}

impl Series<()> {
    /// Creates a new `SeriesBuilder` with default options.
    ///
    /// See the type documentation of [`SeriesBuilder`] for more details.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # use recurring::Series;
    /// use jiff::civil::date;
    /// use recurring::repeat::daily;
    ///
    /// let series = Series::builder()
    ///     .start(date(2025, 1, 1).at(0, 0, 0, 0))
    ///     .end(date(2026, 1, 1).at(0, 0, 0, 0))
    ///     .build(daily(1))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> SeriesBuilder {
        SeriesBuilder::default()
    }
}

impl<R> Series<R>
where
    R: Repeat,
{
    /// Convenience method to create a new `Series` starting at `start` that produces events in the
    /// given `repeat` interval.
    ///
    /// This is equivalent to calling `Series::builder().start(start).build(repeat)`.
    ///
    /// For more series configuration options consider using the builder provided by
    /// [`Series::builder`]. See the documentation of [`SeriesBuilder`] for more details.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if `start` is `DateTime::MAX`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::civil::date;
    /// # use recurring::Series;
    /// use recurring::repeat::hourly;
    ///
    /// let series = Series::new(date(2025, 1, 1).at(0, 0, 0, 0), hourly(2))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(start: DateTime, repeat: R) -> Result<Series<R>, Error> {
        Series::builder().start(start).build(repeat)
    }

    /// Creates an iterator over the events in a the series.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::civil::date;
    /// use recurring::{Event, Series};
    /// use recurring::repeat::hourly;
    ///
    /// let series = Series::builder()
    ///     .start(date(2025, 1, 1).at(0, 0, 0, 0))
    ///     .build(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(2, 0, 0, 0))));
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter(&self) -> Iter<'_, R> {
        Iter::new(self)
    }

    /// Gets the first event in the series.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::civil::date;
    /// use recurring::{Event, Series};
    /// use recurring::repeat::hourly;
    ///
    /// let series = Series::builder()
    ///     .start(date(2025, 1, 1).at(0, 0, 0, 0))
    ///     .build(hourly(2))?;
    ///
    /// assert_eq!(series.first_event(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// # Ok(())
    /// # }
    /// ```
    pub fn first_event(&self) -> Option<Event> {
        self.get_event(self.bounds.start)
            .or_else(|| self.get_event_after(self.bounds.start))
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
    /// let series = Series::builder()
    ///     .start(date(2025, 1, 1).at(0, 0, 0, 0))
    ///     .end(date(2026, 1, 1).at(0, 0, 0, 0))
    ///     .build(hourly(2))?;
    ///
    /// assert_eq!(series.last_event(), Some(Event::at(date(2025, 12, 31).at(22, 0, 0, 0))));
    /// # Ok(())
    /// # }
    /// ```
    pub fn last_event(&self) -> Option<Event> {
        self.get_event_before(self.bounds.end)
    }

    /// Returns `true` when the series contains an event starting at `instant`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::civil::date;
    /// # use recurring::Series;
    /// use recurring::repeat::hourly;
    ///
    /// let series = Series::builder()
    ///     .start(date(2025, 1, 1).at(0, 0, 0, 0))
    ///     .build(hourly(2))?;
    ///
    /// assert!(!series.contains_event(date(2025, 1, 1).at(0, 35, 0, 0)));
    /// assert!(series.contains_event(date(2025, 2, 10).at(12, 0, 0, 0)));
    /// # Ok(())
    /// # }
    /// ```
    pub fn contains_event(&self, instant: DateTime) -> bool {
        self.bounds.contains(&instant) && self.repeat.is_event_start(instant, self.bounds.start)
    }

    /// Gets an event in the series.
    ///
    /// Returns `Some(_)` if there's an event starting at `instant`, otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// use jiff::civil::date;
    /// use recurring::{Event, Series};
    /// use recurring::repeat::hourly;
    ///
    /// let series = Series::builder()
    ///     .start(date(2025, 1, 1).at(0, 0, 0, 0))
    ///     .build(hourly(2))?;
    ///
    /// assert!(series.get_event(date(2025, 1, 1).at(1, 0, 0, 0)).is_none());
    /// assert!(series.get_event(date(2026, 12, 31).at(14, 0, 0, 0)).is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_event(&self, instant: DateTime) -> Option<Event> {
        if self.contains_event(instant) {
            self.get_event_unchecked(instant)
        } else {
            None
        }
    }

    fn get_event_unchecked(&self, start: DateTime) -> Option<Event> {
        if self.event_duration.is_positive() {
            let end = start.checked_add(self.event_duration).ok()?;
            Event::new(start, end).ok()
        } else {
            Some(Event::at(start))
        }
    }

    pub fn get_event_containing(&self, instant: DateTime) -> Option<Event> {
        if let Some(event) = self.get_event(instant) {
            return Some(event);
        }

        let previous = self.get_event_before(instant)?;

        if previous.contains(instant) {
            Some(previous)
        } else {
            None
        }
    }

    pub fn get_event_after(&self, instant: DateTime) -> Option<Event> {
        let mut start = self.align_to_event_start(instant)?;

        if start <= instant {
            start = self.repeat.next_event(start)?;
        }

        self.get_event(start)
    }

    pub fn get_event_before(&self, instant: DateTime) -> Option<Event> {
        let mut start = self.align_to_event_start(instant)?;

        if start >= instant {
            start = self.repeat.previous_event(start)?;
        }

        self.get_event(start)
    }

    pub fn get_closest_event(&self, instant: DateTime) -> Option<Event> {
        if let Some(event) = self.get_event(instant) {
            return Some(event);
        }

        match (
            self.get_event_before(instant),
            self.get_event_after(instant),
        ) {
            (Some(before), Some(after)) => {
                if before.start().duration_until(instant) < after.start().duration_since(instant) {
                    Some(before)
                } else {
                    Some(after)
                }
            }
            (Some(before), None) => Some(before),
            (None, Some(after)) => Some(after),
            (None, None) => None,
        }
    }

    fn align_to_event_start(&self, instant: DateTime) -> Option<DateTime> {
        let aligned = self
            .repeat
            .align_to_event_start(instant, self.bounds.start)?;

        if aligned < self.bounds.start {
            self.repeat
                .align_to_event_start(self.bounds.start, self.bounds.start)
        } else if aligned >= self.bounds.end {
            self.repeat
                .align_to_event_start(self.bounds.end, self.bounds.start)
        } else {
            Some(aligned)
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
/// Values of this type are produced by [`Series::builder`]. The `SeriesBuilder` can be
/// materialized into a `Series` by calling its [`.build()`](SeriesBuilder::build) method with the
/// desired repeat interval for the event series.
///
/// The builder allows to configure the following optional parameters for a `Series`:
///
/// - `start`: The datetime at which the series starts. This is not necessarily identical to the
///   start of the first event in the series. The default value is the current datetime upon
///   calling the [`SeriesBuilder::build`].
/// - `end`: The datetime at which the series ends. The default is [`DateTime::MAX`], which means
///   that it only ends when events become unrepresentable as `DateTime`.
/// - `event_duration`: The [`Span`] of an individual event in the series. This could be minutes,
///   hours, days or any other duration the `Span` type supports. If `event_duration` is not set,
///   individual events will not have an end datetime and have an effective duration of zero.
#[derive(Debug, Clone, Default)]
pub struct SeriesBuilder {
    start: Option<DateTime>,
    end: Option<DateTime>,
    event_duration: Option<Span>,
}

impl SeriesBuilder {
    /// Sets the start of the series.
    ///
    /// If `.start()` is not called with a custom value, the `.build()` method will set the start
    /// of the series to the current datetime.
    ///
    /// # Example
    ///
    /// ```
    /// # use recurring::Series;
    /// use jiff::civil::date;
    ///
    /// let builder = Series::builder().start(date(2025, 1, 1).at(0, 0, 0, 0));
    /// ```
    #[must_use]
    pub fn start(mut self, start: DateTime) -> SeriesBuilder {
        self.start = Some(start);
        self
    }

    /// Sets the end of the series.
    ///
    /// The end of the series defaults to [`DateTime::MAX`] if `.end()` is not called with a custom
    /// value.
    ///
    /// # Example
    ///
    /// ```
    /// # use recurring::Series;
    /// use jiff::civil::date;
    ///
    /// let builder = Series::builder().end(date(2025, 1, 1).at(1, 30, 0, 0));
    /// ```
    #[must_use]
    pub fn end(mut self, end: DateTime) -> SeriesBuilder {
        self.end = Some(end);
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
    /// # use recurring::Series;
    /// use jiff::ToSpan;
    /// use jiff::civil::date;
    ///
    /// let builder = Series::builder().event_duration(1.hour());
    /// ```
    #[must_use]
    pub fn event_duration(mut self, event_duration: Span) -> SeriesBuilder {
        self.event_duration = Some(event_duration);
        self
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
    /// let series = Series::builder()
    ///     .start(date(2025, 1, 1).at(0, 0, 0, 0))
    ///     .build(daily(1))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build<R>(self, repeat: R) -> Result<Series<R>, Error>
    where
        R: Repeat,
    {
        let start = self.start.unwrap_or_else(|| Zoned::now().datetime());
        let end = self.end.unwrap_or(DateTime::MAX);
        let event_duration = self.event_duration.unwrap_or_default();

        if end <= start {
            return Err(Error::InvalidSeriesEnd);
        }

        if event_duration.is_negative() {
            return Err(Error::InvalidEventDuration);
        }

        Ok(Series {
            repeat,
            bounds: (start..end),
            event_duration,
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
            next_start: Some(series.bounds.start),
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

        if !series.bounds.contains(&start) {
            return None;
        }

        self.next_start = series.repeat.next_event(start);

        // Handle the case where the series start does not fall into the desired frequency and
        // skip over to the next event right away.
        if start == series.bounds.start && !series.contains_event(start) {
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
        civil::{datetime, time},
    };

    #[test]
    fn daily_series() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let series = Series::new(start, daily(2)).unwrap();
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
        let series =
            Series::new(start, daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3))).unwrap();
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
        let series = Series::builder()
            .start(start)
            .end(end)
            .build(daily(2))
            .unwrap();
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
        let series = Series::builder()
            .start(start)
            .end(end)
            .event_duration(1.hour())
            .build(daily(2))
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
        let series =
            Series::new(start, daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3))).unwrap();
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
        let series = Series::builder()
            .start(start)
            .end(end)
            .build(daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3)))
            .unwrap();
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
        let series = Series::builder()
            .start(start)
            .end(end)
            .build(daily(2).at(time(2, 2, 2, 2)))
            .unwrap();
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

        let series = Series::builder()
            .start(start)
            .end(end)
            .event_duration(1.hour())
            .build(daily(2).at(time(2, 2, 2, 2)))
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
        let series = Series::builder()
            .start(start)
            .end(end)
            .build(daily(2).at(time(2, 2, 2, 2)))
            .unwrap();
        assert_eq!(
            series.first_event(),
            Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)))
        );
    }

    #[test]
    fn series_last_event() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 10, 1, 1, 1, 0);
        let series = Series::builder()
            .start(start)
            .end(end)
            .build(daily(2).at(time(2, 2, 2, 2)))
            .unwrap();
        assert_eq!(
            series.last_event(),
            Some(Event::at(datetime(2025, 1, 8, 2, 2, 2, 2)))
        );

        let series = Series::new(start, daily(2).at(time(2, 2, 2, 2))).unwrap();
        assert_eq!(
            series.last_event(),
            Some(Event::at(datetime(9999, 12, 31, 2, 2, 2, 2)))
        );
    }

    #[test]
    fn series_get_closest_event() {
        let start = datetime(2025, 1, 1, 0, 0, 0, 0);
        let series = Series::new(start, hourly(1)).unwrap();

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
