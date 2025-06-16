use crate::error::{Error, err};
use crate::series::{Series, SeriesCore};
use crate::{IntoBounds, Pattern, try_simplify_range};
use core::ops::{Bound, RangeBounds};
use jiff::{Span, civil::DateTime};

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
/// - `fixpoint`: A custom fixpoint different from the series `start` for relative recurrence
///   patterns.
#[derive(Debug, Clone)]
pub struct SeriesWith<P> {
    pattern: P,
    bounds: (Bound<DateTime>, Bound<DateTime>),
    fixpoint: Option<DateTime>,
    event_duration: Span,
}

impl<P> SeriesWith<P>
where
    P: Pattern,
{
    /// Creates a new `SeriesWith` from a `Series`.
    pub(crate) fn new(series: Series<P>) -> SeriesWith<P> {
        SeriesWith {
            pattern: series.core.pattern,
            bounds: series.range.into_bounds(),
            fixpoint: series.range.fixpoint,
            event_duration: series.core.event_duration,
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

    /// Sets a fixpoint for relative recurrence patterns.
    ///
    /// This is used as a starting point for `Pattern` implementations that are relative to some
    /// point in time to calculate the closest recurrence adjacent to a certain instant.
    ///
    /// The default behaviour is to use the series start as a fixpoint.
    ///
    /// The fixpoint must be less or equal to the series start.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::{ToSpan, civil::date};
    /// use recurring::{Event, Series, pattern::daily};
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let fixpoint = date(2024, 12, 31).at(12, 34, 56, 0);
    /// let series = Series::new(start.., daily(1))
    ///     .with()
    ///     .fixpoint(fixpoint)
    ///     .build()?;
    ///
    /// assert_eq!(series.first(), Some(Event::at(date(2025, 1, 1).at(12, 34, 56, 0))));
    ///
    /// // Fixpoint must be less or equal to `start`.
    /// assert!(series.with().fixpoint(start).build().is_ok());
    /// assert!(series.with().fixpoint(start + 1.nanosecond()).build().is_err());
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn fixpoint(mut self, fixpoint: DateTime) -> SeriesWith<P> {
        self.fixpoint = Some(fixpoint);
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
            fixpoint: self.fixpoint,
            event_duration: self.event_duration,
        }
    }

    /// Builds a [`Series`] that yields events according to the provided recurrence [`Pattern`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the configured `end` is less than or equal to `start`, if the
    /// configured `event_duration` is negative, or if the `event_duration` greater or equal to the
    /// range (`start..end`) of the series, or if the `fixpoint` is greater than the series `start`.
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
            return Err(Error::datetime_range("series", range.into()));
        }

        if let Some(fixpoint) = self.fixpoint {
            range = range.with_fixpoint(fixpoint)?;
        }

        Ok(Series {
            core: SeriesCore::new(self.pattern, self.event_duration),
            range,
        })
    }
}
