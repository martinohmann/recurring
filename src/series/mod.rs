mod iter;
mod range;
mod with;

pub use iter::Iter;
pub use range::SeriesRange;
pub use with::SeriesWith;

use crate::error::Error;
use crate::{Event, Pattern, try_simplify_range};
use core::ops::RangeBounds;
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
    range: SeriesRange,
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
            range: range.into(),
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

    /// Returns the fixpoint for relative recurrence patterns.
    ///
    /// This is used as a starting point for `Pattern` implementations that are relative to some
    /// point in time.
    ///
    /// Unless [`SeriesWith::fixpoint`] was called with a specific value, this returns the same
    /// value as [`Series::start`].
    pub fn fixpoint(&self) -> DateTime {
        self.range.fixpoint()
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
        let closest = self.closest_to(instant)?;
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
        self.next_after(instant)
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
        self.previous_before(instant)
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
        self.closest_to(instant)
            .and_then(|closest| self.get_event_unchecked(closest))
    }
}

// Internal APIs.
impl<P> Series<P>
where
    P: Pattern,
{
    /// Find the next `DateTime` after `instant` within the series.
    #[inline]
    fn next_after(&self, instant: DateTime) -> Option<DateTime> {
        self.pattern.next_after(instant, self.range)
    }

    /// Find the previous `DateTime` before `instant` within the series.
    #[inline]
    fn previous_before(&self, instant: DateTime) -> Option<DateTime> {
        self.pattern.previous_before(instant, self.range)
    }

    /// Find a `DateTime` closest to `instant` within the series.
    #[inline]
    fn closest_to(&self, instant: DateTime) -> Option<DateTime> {
        self.pattern.closest_to(instant, self.range)
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
