use crate::{
    Event, IntoBounds, Pattern,
    error::{Error, err},
    try_simplify_range,
};
use core::ops::{Bound, Range, RangeBounds};
use jiff::{Span, civil::DateTime};

/// A series of recurring events.
///
/// # Example
///
/// ```
/// use jiff::civil::date;
/// use recurring::{Event, Series, pattern::hourly};
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
/// # Ok::<(), Box<dyn core::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct Series<P> {
    pattern: P,
    range: Range<DateTime>,
    event_duration: Span,
}

impl<P> Series<P>
where
    P: Pattern,
{
    /// Creates a new `Series` that produces events within the provided `range` according to the
    /// given recurrence [`Pattern`].
    ///
    /// To configure more aspects of the series call `.with()` on the constructed
    /// `Series` value. See the documentation of [`Series::with`] for more details.
    ///
    /// The fallible version of this method is [`Series::try_new`].
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
    /// use recurring::{Series, pattern::hourly};
    ///
    /// let series = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2));
    /// ```
    #[inline]
    pub fn new<B: RangeBounds<DateTime>>(range: B, pattern: P) -> Series<P> {
        Series::try_new(range, pattern).expect("invalid series range bounds")
    }

    /// Creates a new `Series` that produces events within the provided `range` according to the
    /// given recurrence [`Pattern`].
    ///
    /// To configure more aspects of the series call `.with()` on the constructed
    /// `Series` value. See the documentation of [`Series::with`] for more details.
    ///
    /// The panicking version of this method is [`Series::new`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the start or end of the range bounds would overflow `DateTime::MAX`
    /// after normalization or if `start` >= `end`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::{DateTime, date};
    /// use recurring::{Series, pattern::hourly};
    ///
    /// assert!(Series::try_new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2)).is_ok());
    /// assert!(Series::try_new(DateTime::MAX.., hourly(2)).is_err());
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    pub fn try_new<B: RangeBounds<DateTime>>(range: B, pattern: P) -> Result<Series<P>, Error> {
        let range = try_simplify_range(range)?;
        if range.start >= range.end {
            return Err(Error::datetime_range("series", range));
        }

        Ok(Series {
            pattern,
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
    /// use jiff::{ToSpan, civil::date};
    /// use recurring::{Series, pattern::daily};
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let s2 = s1.with()
    ///     .end(date(2025, 2, 1).at(0, 0, 0, 0))
    ///     .event_duration(1.hour())
    ///     .build()?;
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    pub fn with(&self) -> SeriesWith<P> {
        SeriesWith::new(self.clone())
    }

    /// Returns the range from series start (inclusive) to series end (exclusive).
    ///
    /// If the series has a non-zero event duration configured, the returned range will end at
    /// `initial_end - event_duration`.
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
    /// bound until after which the series will stop yielding events.
    ///
    /// If the series has a non-zero event duration configured, this will return `initial_end -
    /// event_duration`.
    pub fn end(&self) -> DateTime {
        self.range.end
    }

    /// Returns the duration of individual events in the series.
    ///
    /// If this is zero, events will not have an end date.
    pub fn event_duration(&self) -> Span {
        self.event_duration
    }

    /// Returns a reference to the recurrence pattern used by the series.
    pub fn pattern(&self) -> &P {
        &self.pattern
    }

    /// Creates an iterator over the events in a the series.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::{Event, Series, pattern::hourly};
    ///
    /// let series = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2));
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// assert_eq!(events.next(), Some(Event::at(date(2025, 1, 1).at(2, 0, 0, 0))));
    ///
    /// // Get events from the end of the series.
    /// assert_eq!(events.next_back(), Some(Event::at(date(9999, 12, 31).at(22, 0, 0, 0))));
    /// assert_eq!(events.next_back(), Some(Event::at(date(9999, 12, 31).at(20, 0, 0, 0))));
    /// ```
    pub fn iter(&self) -> Iter<'_, P> {
        Iter::new(self)
    }

    /// Gets the first event in the series.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::{Event, Series, pattern::hourly};
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
    /// use jiff::civil::date;
    /// use recurring::{Event, Series, pattern::hourly};
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let end = date(2026, 1, 1).at(0, 0, 0, 0);
    ///
    /// let series = Series::new(start..end, hourly(2));
    ///
    /// assert_eq!(series.last_event(), Some(Event::at(date(2025, 12, 31).at(22, 0, 0, 0))));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
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
    /// use recurring::{Event, Series, pattern::hourly};
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
    /// use recurring::{Event, Series, pattern::hourly};
    ///
    /// let series = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2));
    ///
    /// assert!(series.get_event(date(2025, 1, 1).at(1, 0, 0, 0)).is_none());
    /// assert!(series.get_event(date(2026, 12, 31).at(14, 0, 0, 0)).is_some());
    /// ```
    pub fn get_event(&self, instant: DateTime) -> Option<Event> {
        let closest = self.pattern.closest_to(instant, &self.range)?;
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
    /// use jiff::{ToSpan, civil::date};
    /// use recurring::{Event, Series, pattern::hourly};
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
    ///     Some(Event::new(series_start, series_start + 30.minutes())),
    /// );
    /// assert_eq!(
    ///     series.get_event_containing(series_start + 31.minutes()),
    ///     None,
    /// );
    /// assert_eq!(
    ///     series.get_event_containing(series_start + 1.hour().minutes(20)),
    ///     Some(Event::new(series_start + 1.hour(), series_start + 1.hour().minutes(30))),
    /// );
    /// assert_eq!(
    ///     series.get_event_containing(series_end),
    ///     None,
    /// );
    /// # Ok::<(), Box<dyn core::error::Error>>(())
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
    /// use jiff::{ToSpan, civil::{date, DateTime}};
    /// use recurring::{Event, Series, pattern::hourly};
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
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    pub fn get_event_after(&self, instant: DateTime) -> Option<Event> {
        self.pattern
            .next_after(instant, &self.range)
            .and_then(|next| self.get_event_unchecked(next))
    }

    /// Gets the previous event before `instant`.
    ///
    /// Returns `None` if `instant` is less than or equal to the series start.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::{ToSpan, civil::{DateTime, date}};
    /// use recurring::{Event, Series, pattern::hourly};
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
        self.pattern
            .previous_before(instant, &self.range)
            .and_then(|previous| self.get_event_unchecked(previous))
    }

    /// Gets the series event with the start time closest to `instant`.
    ///
    /// The returned event may start before, at or after `instant`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::{ToSpan, civil::date};
    /// use recurring::{Event, Series, pattern::hourly};
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
        self.pattern
            .closest_to(instant, &self.range)
            .and_then(|closest| self.get_event_unchecked(closest))
    }

    /// Get an event without any bound checks. The datetime at `start` is assumed to be aligned to
    /// the series and within the series start and end bounds.
    #[inline]
    fn get_event_unchecked(&self, start: DateTime) -> Option<Event> {
        if self.event_duration.is_positive() {
            let end = start.checked_add(self.event_duration).ok()?;
            Some(Event::new_unchecked(start, end))
        } else {
            Some(Event::at(start))
        }
    }
}

impl<'a, P> IntoIterator for &'a Series<P>
where
    P: Pattern,
{
    type Item = Event;
    type IntoIter = Iter<'a, P>;

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
/// - `pattern`: The recurrence pattern for the series.
#[derive(Debug, Clone)]
pub struct SeriesWith<P> {
    pattern: P,
    bounds: (Bound<DateTime>, Bound<DateTime>),
    event_duration: Span,
}

impl<P> SeriesWith<P>
where
    P: Pattern,
{
    /// Creates an new `SeriesWith` from a `Series`.
    fn new(series: Series<P>) -> SeriesWith<P> {
        SeriesWith {
            pattern: series.pattern,
            bounds: series.range.into_bounds(),
            event_duration: series.event_duration,
        }
    }

    /// Sets the range of the series.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::{ToSpan, civil::date};
    /// use recurring::{Series, pattern::daily};
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
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn range<B: RangeBounds<DateTime>>(mut self, range: B) -> SeriesWith<P> {
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
    /// use jiff::civil::date;
    /// use recurring::{Series, pattern::daily};
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let start = date(2025, 2, 1).at(0, 0, 0, 0);
    ///
    /// let s2 = s1.with().start(start).build()?;
    ///
    /// assert_eq!(s2.start(), start);
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn start(mut self, start: DateTime) -> SeriesWith<P> {
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
    /// use jiff::civil::date;
    /// use recurring::{Series, pattern::daily};
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let end = date(2025, 2, 1).at(0, 0, 0, 0);
    ///
    /// let s2 = s1.with().end(end).build()?;
    ///
    /// assert_eq!(s2.end(), end);
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn end(mut self, end: DateTime) -> SeriesWith<P> {
        self.bounds.1 = Bound::Excluded(end);
        self
    }

    /// Sets the duration of individual events in the series.
    ///
    /// If `.event_duration()` is not called with a custom value, events will not have an end
    /// datetime.
    ///
    /// The event duration may be longer than the time between the dates produces by the recurrence
    /// pattern, in which case events will overlap.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::{ToSpan, civil::date};
    /// use recurring::{Series, pattern::daily};
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let s2 = s1.with().event_duration(1.hour()).build()?;
    ///
    /// assert_eq!(s2.event_duration().fieldwise(), 1.hour());
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn event_duration(mut self, event_duration: Span) -> SeriesWith<P> {
        self.event_duration = event_duration;
        self
    }

    /// Sets the recurrence pattern for the series.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::{ToSpan, civil::date};
    /// use recurring::{Series, pattern::daily};
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let s2 = s1.with().pattern(daily(2)).build()?;
    ///
    /// assert_eq!(s2.pattern(), &daily(2));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn pattern<Q: Pattern>(self, pattern: Q) -> SeriesWith<Q> {
        SeriesWith {
            pattern,
            bounds: self.bounds,
            event_duration: self.event_duration,
        }
    }

    /// Builds a [`Series`] that yields events according to the provided recurrence [`Pattern`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the configured `end` is less than or equal to `start`, if the
    /// configured `event_duration` is negative, or if the `event_duration` greater or equal to the
    /// range (`start..end`) of the series.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::{ToSpan, civil::date};
    /// use recurring::{Series, pattern::daily};
    ///
    /// let s1 = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., daily(1));
    ///
    /// let s2 = s1.with()
    ///     .end(date(2025, 1, 3).at(0, 0, 0, 0))
    ///     .build()?;
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    pub fn build(self) -> Result<Series<P>, Error> {
        let mut range = try_simplify_range(self.bounds)?;

        if self.event_duration.is_negative() {
            return Err(err!(
                "event duration must be positive or zero but got {}",
                self.event_duration
            ));
        }

        if self.event_duration.is_positive() {
            range.end = range.end.checked_sub(self.event_duration)?;
        }

        if range.start >= range.end {
            return Err(Error::datetime_range("series", range));
        }

        Ok(Series {
            pattern: self.pattern,
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
pub struct Iter<'a, P> {
    series: &'a Series<P>,
    first: bool,
    cursor_front: Option<DateTime>,
    cursor_back: Option<DateTime>,
}

impl<'a, P: Pattern> Iter<'a, P> {
    fn new(series: &'a Series<P>) -> Iter<'a, P> {
        Iter {
            series,
            first: true,
            cursor_front: Some(series.start()),
            cursor_back: Some(series.end()),
        }
    }
}

impl<P> Iterator for Iter<'_, P>
where
    P: Pattern,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let cursor = self.cursor_front.take()?;
        let event = if self.first {
            self.first = false;
            self.series.first_event()?
        } else {
            self.series.get_event_after(cursor)?
        };

        self.cursor_front = Some(event.start());
        Some(event)
    }
}

impl<P> DoubleEndedIterator for Iter<'_, P>
where
    P: Pattern,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let cursor = self.cursor_back.take()?;
        let event = self.series.get_event_before(cursor)?;
        self.cursor_back = Some(event.start());
        Some(event)
    }
}
