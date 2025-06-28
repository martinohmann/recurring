//! A series of recurring events.
mod core;
mod iter;
mod range;
mod split;
mod with;

use core::SeriesCore;
pub use iter::Iter;
pub use range::Range;
pub use split::{SeriesSplit, SplitMode};
pub use with::SeriesWith;

use crate::error::Error;
use crate::{DateTimeRange, Event, Pattern, try_simplify_range};
use ::core::ops::RangeBounds;
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
    core: SeriesCore<P>,
    range: DateTimeRange,
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
    #[inline]
    pub fn try_new<B: RangeBounds<DateTime>>(range: B, pattern: P) -> Result<Series<P>, Error> {
        let range = try_simplify_range(range)?;
        if range.start >= range.end {
            return Err(Error::datetime_range("series", range.into()));
        }

        Ok(Series {
            core: SeriesCore::new(pattern, Span::new()),
            range,
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
    #[inline]
    pub fn with(&self) -> SeriesWith<P> {
        SeriesWith::new(self.clone())
    }

    /// Returns the `DateTime` at which the series starts (inclusive).
    ///
    /// This is not necessarily the time of the first event in the series.
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn fixpoint(&self) -> DateTime {
        self.range.fixpoint()
    }

    /// Returns the duration of individual events in the series.
    ///
    /// If this is zero, events will not have an end date.
    #[inline]
    pub fn event_duration(&self) -> Span {
        self.core.event_duration()
    }

    /// Returns a reference to the recurrence pattern used by the series.
    #[inline]
    pub fn pattern(&self) -> &P {
        self.core.pattern()
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
    /// let mut iter = series.iter();
    ///
    /// assert_eq!(iter.next(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// assert_eq!(iter.next(), Some(Event::at(date(2025, 1, 1).at(2, 0, 0, 0))));
    ///
    /// // Get events from the end of the series.
    /// assert_eq!(iter.next_back(), Some(Event::at(date(9999, 12, 31).at(22, 0, 0, 0))));
    /// assert_eq!(iter.next_back(), Some(Event::at(date(9999, 12, 31).at(20, 0, 0, 0))));
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, P> {
        Iter::new(self)
    }

    /// Creates an iterator over a sub-range of the events in a the series.
    ///
    /// The returned iterator will iterate over the intersection of the provided range and the
    /// series' original range.
    ///
    /// The fallible version of this method is [`Series::try_range`].
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
    /// use recurring::{Event, Series, pattern::hourly};
    ///
    /// let series = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2));
    ///
    /// let range = date(2026, 1, 1).at(12, 34, 56, 0)..date(2027, 1, 1).at(12, 34, 56, 0);
    /// let mut iter = series.range(range);
    ///
    /// assert_eq!(iter.next(), Some(Event::at(date(2026, 1, 1).at(14, 0, 0, 0))));
    /// assert_eq!(iter.next(), Some(Event::at(date(2026, 1, 1).at(16, 0, 0, 0))));
    ///
    /// // Get events from the end of the series sub-range.
    /// assert_eq!(iter.next_back(), Some(Event::at(date(2027, 1, 1).at(12, 0, 0, 0))));
    /// assert_eq!(iter.next_back(), Some(Event::at(date(2027, 1, 1).at(10, 0, 0, 0))));
    /// ```
    #[inline]
    pub fn range<B: RangeBounds<DateTime>>(&self, range: B) -> Range<'_, P> {
        self.try_range(range).expect("invalid range bounds")
    }

    /// Creates an iterator over a sub-range of the events in a the series.
    ///
    /// The returned iterator will iterate over the intersection of the provided range and the
    /// series' original range.
    ///
    /// The panicking version of this method is [`Series::range`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the start or end of the range bounds would overflow `DateTime::MAX`
    /// after normalization or if `start` >= `end`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::{Event, Series, pattern::hourly};
    ///
    /// let series = Series::new(date(2025, 1, 1).at(0, 0, 0, 0).., hourly(2));
    ///
    /// let range = date(2026, 1, 1).at(12, 34, 56, 0)..date(2027, 1, 1).at(12, 34, 56, 0);
    /// let mut iter = series.try_range(range)?;
    ///
    /// assert_eq!(iter.next(), Some(Event::at(date(2026, 1, 1).at(14, 0, 0, 0))));
    /// assert_eq!(iter.next(), Some(Event::at(date(2026, 1, 1).at(16, 0, 0, 0))));
    ///
    /// // Get events from the end of the series sub-range.
    /// assert_eq!(iter.next_back(), Some(Event::at(date(2027, 1, 1).at(12, 0, 0, 0))));
    /// assert_eq!(iter.next_back(), Some(Event::at(date(2027, 1, 1).at(10, 0, 0, 0))));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub fn try_range<B: RangeBounds<DateTime>>(&self, range: B) -> Result<Range<'_, P>, Error> {
        let mut range = try_simplify_range(range)?;
        if self.event_duration().is_positive() {
            range.end = range.end.checked_sub(self.event_duration())?;
        }

        self.range
            .intersect(range)
            .map(|range| Range::new(&self.core, range))
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
    /// assert_eq!(series.first(), Some(Event::at(date(2025, 1, 1).at(0, 0, 0, 0))));
    /// ```
    #[inline]
    pub fn first(&self) -> Option<Event> {
        self.get_closest_to(self.range.start)
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
    /// assert_eq!(series.last(), Some(Event::at(date(2025, 12, 31).at(22, 0, 0, 0))));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub fn last(&self) -> Option<Event> {
        self.get_previous_before(self.range.end)
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
    /// assert!(!series.contains(date(2025, 1, 1).at(0, 35, 0, 0)));
    /// assert!(series.contains(date(2025, 2, 10).at(12, 0, 0, 0)));
    /// ```
    #[inline]
    pub fn contains(&self, instant: DateTime) -> bool {
        self.get(instant).is_some()
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
    /// assert!(series.get(date(2025, 1, 1).at(1, 0, 0, 0)).is_none());
    /// assert!(series.get(date(2026, 12, 31).at(14, 0, 0, 0)).is_some());
    /// ```
    #[inline]
    pub fn get(&self, instant: DateTime) -> Option<Event> {
        self.core.get(instant, self.range)
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
    ///     series.get_containing(series_start - 1.minute()),
    ///     None,
    /// );
    /// assert_eq!(
    ///     series.get_containing(series_start),
    ///     Some(Event::new(series_start, series_start + 30.minutes())),
    /// );
    /// assert_eq!(
    ///     series.get_containing(series_start + 31.minutes()),
    ///     None,
    /// );
    /// assert_eq!(
    ///     series.get_containing(series_start + 1.hour().minutes(20)),
    ///     Some(Event::new(series_start + 1.hour(), series_start + 1.hour().minutes(30))),
    /// );
    /// assert_eq!(
    ///     series.get_containing(series_end),
    ///     None,
    /// );
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub fn get_containing(&self, instant: DateTime) -> Option<Event> {
        self.core.get_containing(instant, self.range)
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
    ///     series.get_next_after(series_start - 1.minute()),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_next_after(series_start),
    ///     Some(Event::at(series_start + 1.hour())),
    /// );
    /// assert_eq!(
    ///     series.get_next_after(series_start + 1.minute()),
    ///     Some(Event::at(series_start + 1.hour())),
    /// );
    /// assert_eq!(
    ///     series.get_next_after(series_end - 1.hour().minutes(1)),
    ///     Some(Event::at(series_end - 1.hour())),
    /// );
    /// assert_eq!(
    ///     series.get_next_after(series_end - 1.hour()),
    ///     None,
    /// );
    /// assert_eq!(
    ///     series.get_next_after(series_end),
    ///     None,
    /// );
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub fn get_next_after(&self, instant: DateTime) -> Option<Event> {
        self.core.get_next_after(instant, self.range)
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
    /// assert_eq!(series.get_previous_before(series_start), None);
    /// assert_eq!(series.get_previous_before(series_start - 1.minute()), None);
    /// assert_eq!(
    ///     series.get_previous_before(series_start + 29.minute()),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_previous_before(series_start + 1.hour()),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_previous_before(series_start + 1.hour().seconds(1)),
    ///     Some(Event::at(series_start + 1.hour())),
    /// );
    /// assert_eq!(
    ///     series.get_previous_before(DateTime::MAX),
    ///     Some(Event::at(date(9999, 12, 31).at(23, 0, 0, 0))),
    /// );
    /// ```
    #[inline]
    pub fn get_previous_before(&self, instant: DateTime) -> Option<Event> {
        self.core.get_previous_before(instant, self.range)
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
    ///     series.get_closest_to(series_start),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_closest_to(series_start - 1.minute()),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_closest_to(series_start + 29.minutes()),
    ///     Some(Event::at(series_start)),
    /// );
    /// assert_eq!(
    ///     series.get_closest_to(series_start + 30.minutes()),
    ///     Some(Event::at(series_start + 1.hour())),
    /// );
    /// ```
    #[inline]
    pub fn get_closest_to(&self, instant: DateTime) -> Option<Event> {
        self.core.get_closest_to(instant, self.range)
    }

    /// Splits off a part of the series.
    ///
    /// If splitting succeeds, the original series' end is adjusted towards the cutoff point.
    ///
    /// Note that this routine is generic and accepts anything that implements `Into<SeriesSplit>`.
    /// Some notable implementations are:
    ///
    /// * `From<DateTime> for SeriesSplit` will construct a series split configuration which splits
    ///   the series at a given datetime. The `Date`, `Zoned` and `&Zoned` types can be used
    ///   instead of `DateTime` as well.
    /// * `From<(SplitMode, T)> for SeriesSplit where T: Into<SeriesSplit>` will construct a series
    ///   split configuration using a certain [`SplitMode`]. This enables splitting at the next,
    ///   previous or closest event.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`SeriesSplit`] fails to find a cutoff point according to its
    /// [`SplitMode`] or if splitting the series would result in either of the series to have
    /// `start >= end`. It is guaranteed that the original series was not altered if this method
    /// returns an error.
    ///
    /// # Example: split at an exact datetime
    ///
    /// ```
    /// use jiff::{ToSpan, civil::{date, DateTime}};
    /// use recurring::{Event, Series, pattern::hourly};
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let mut s1 = Series::new(start.., hourly(1));
    ///
    /// let cutoff_point = date(2025, 4, 1).at(12, 34, 56, 0);
    ///
    /// let s2 = s1.split_off(cutoff_point)?;
    ///
    /// assert_eq!(s2.first(), Some(Event::at(date(2025, 4, 1).at(13, 0, 0, 0))));
    /// assert_eq!(s1.end(), cutoff_point);
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    ///
    /// # Example: split at the next event after a datetime
    ///
    /// ```
    /// use jiff::{ToSpan, civil::{date, DateTime}};
    /// use recurring::{Event, Series, pattern::hourly};
    /// use recurring::series::SplitMode;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let mut s1 = Series::new(start.., hourly(1));
    ///
    /// let instant = date(2025, 4, 1).at(12, 34, 56, 0);
    ///
    /// // Use `SplitMode::NextAfter` to split at the next event after `instant`.
    /// let s2 = s1.split_off((SplitMode::NextAfter, instant))?;
    ///
    /// assert_eq!(s2.first(), Some(Event::at(date(2025, 4, 1).at(13, 0, 0, 0))));
    /// assert_eq!(s1.end(), date(2025, 4, 1).at(13, 0, 0, 0));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    pub fn split_off<S: Into<SeriesSplit>>(&mut self, options: S) -> Result<Series<P>, Error> {
        let options: SeriesSplit = options.into();
        options.split_off(self)
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
