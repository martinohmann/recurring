//! Types for specifying repeat intervals.
use crate::{Error, Repeat};
use alloc::vec::Vec;
use core::ops::Range;
use jiff::{
    Span, SpanTotal, ToSpan, Unit,
    civil::{DateTime, Time},
};

/// A precise interval for repeating events.
///
/// # Example
///
/// ```
/// use jiff::ToSpan;
/// use recurring::repeat::Interval;
///
/// let every_two_hours = Interval::new(2.hours());
/// ```
#[derive(Debug, Clone)]
pub struct Interval(Span);

impl Interval {
    /// Creates a new `Interval` from a `Span`.
    ///
    /// For a fallible alternative see [`Interval::try_new`].
    ///
    /// # Panics
    ///
    /// Panics if `span` is negative or zero.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::ToSpan;
    /// use recurring::repeat::Interval;
    ///
    /// let every_two_hours = Interval::new(2.hours());
    /// ```
    pub fn new(span: Span) -> Interval {
        assert!(span.is_positive(), "interval must be positive and non-zero");
        Interval(span)
    }

    /// Creates a new `Interval` from a `Span`.
    ///
    /// For an infallible alternative that panics on invalid spans instead see [`Interval::new`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if `span` is negative or zero.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::ToSpan;
    /// use recurring::repeat::Interval;
    ///
    /// assert!(Interval::try_new(1.day()).is_ok());
    /// assert!(Interval::try_new(0.seconds()).is_err());
    /// assert!(Interval::try_new(-1.day()).is_err());
    /// ```
    pub fn try_new(span: Span) -> Result<Interval, Error> {
        if !span.is_positive() {
            return Err(Error::InvalidInterval);
        }

        Ok(Interval(span))
    }
}

impl Repeat for Interval {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_add(self.0).ok()
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_sub(self.0).ok()
    }

    fn is_aligned_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> bool {
        is_aligned_to_series(instant, bounds, self.0)
    }

    fn align_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime> {
        align_to_series(instant, bounds, self.0)
    }
}

/// An interval for daily events which may also include fixed times of the day.
///
/// # Example
///
/// ```
/// use recurring::repeat::Daily;
/// use jiff::civil::time;
///
/// let every_two_days_at_twelve = Daily::new(1).at(time(12, 0, 0, 0));
/// ```
#[derive(Debug, Clone)]
pub struct Daily {
    interval: Interval,
    at: Vec<Time>,
}

impl Daily {
    /// Creates a new `Daily` from an interval of days.
    ///
    /// For a fallible alternative see [`Daily::try_new`].
    ///
    /// # Panics
    ///
    /// Panics if `span` is negative or zero.
    ///
    /// # Example
    ///
    /// ```
    /// use recurring::repeat::Daily;
    ///
    /// let every_two_days = Daily::new(2);
    /// ```
    pub fn new<I: ToSpan>(interval: I) -> Daily {
        Daily {
            interval: Interval::new(interval.days()),
            at: Vec::new(),
        }
    }

    /// Creates a new `Daily` from an interval of days.
    ///
    /// For an infallible alternative that panics on invalid spans instead see [`Daily::new`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if `span` is negative or zero.
    ///
    /// # Example
    ///
    /// ```
    /// use recurring::repeat::Daily;
    ///
    /// assert!(Daily::try_new(1).is_ok());
    /// assert!(Daily::try_new(0).is_err());
    /// assert!(Daily::try_new(-1).is_err());
    /// ```
    pub fn try_new<I: ToSpan>(interval: I) -> Result<Daily, Error> {
        Ok(Daily {
            interval: Interval::try_new(interval.days())?,
            at: Vec::new(),
        })
    }

    /// Adds a time to the daily repeat interval.
    ///
    /// # Example
    ///
    /// ```
    /// use recurring::repeat::daily;
    /// use jiff::civil::time;
    ///
    /// let every_day_at_twelve = daily(1).at(time(12, 0, 0, 0));
    ///
    /// let every_day_at_midnight_and_twelve = every_day_at_twelve.at(time(0, 0, 0, 0));
    /// ```
    #[must_use]
    pub fn at<T: Into<Time>>(self, time: T) -> Daily {
        self.at_times([time])
    }

    /// Adds times to the daily repeat interval.
    ///
    /// # Example
    ///
    /// ```
    /// use recurring::repeat::daily;
    /// use jiff::civil::time;
    ///
    /// let every_day_at_midnight_and_twelve = daily(1)
    ///     .at_times([time(0, 0, 0, 0), time(12, 0, 0, 0)]);
    /// ```
    #[must_use]
    pub fn at_times<T: IntoIterator<Item = impl Into<Time>>>(mut self, times: T) -> Daily {
        self.at.extend(times.into_iter().map(Into::into));
        self.at.sort();
        self.at.dedup();
        self
    }
}

impl Repeat for Daily {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        if self.at.is_empty() {
            self.interval.next_event(instant)
        } else {
            for time in &self.at {
                let date = instant.with().time(*time).build().ok()?;

                if date > instant {
                    return Some(date);
                }
            }

            let next = self.interval.next_event(instant)?;
            next.with().time(self.at[0]).build().ok()
        }
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        if self.at.is_empty() {
            self.interval.previous_event(instant)
        } else {
            for time in self.at.iter().rev() {
                let date = instant.with().time(*time).build().ok()?;

                if date < instant {
                    return Some(date);
                }
            }

            let previous = self.interval.previous_event(instant)?;
            previous.with().time(*self.at.last().unwrap()).build().ok()
        }
    }

    fn is_aligned_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> bool {
        if self.at.is_empty() {
            self.interval.is_aligned_to_series(instant, bounds)
        } else {
            instant
                .checked_sub(1.minute())
                .ok()
                .and_then(|start| self.next_event(start))
                .is_some_and(|date| date == instant)
        }
    }

    fn align_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime> {
        if self.at.is_empty() {
            self.interval.align_to_series(instant, bounds)
        } else {
            let aligned = self.interval.align_to_series(instant, bounds)?;

            self.at
                .iter()
                .filter_map(|time| aligned.with().time(*time).build().ok())
                .filter(|date| bounds.contains(date))
                .min_by_key(|date| date.duration_since(instant).abs())
        }
    }
}

/// Creates an interval for repeating events.
///
/// # Panics
///
/// Panics if `span` is negative or zero.
///
/// # Example
///
/// ```
/// use jiff::ToSpan;
/// use recurring::repeat::interval;
///
/// let every_day_and_a_half = interval(1.day().hours(12));
/// ```
#[inline]
pub fn interval(span: Span) -> Interval {
    Interval::new(span)
}

/// Creates an interval for repeating events on a per-second basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::secondly;
///
/// let every_ten_seconds = secondly(10);
/// ```
#[inline]
pub fn secondly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.seconds())
}

/// Creates an interval for repeating events on a per-minute basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::minutely;
///
/// let every_thirty_minutes = minutely(30);
/// ```
#[inline]
pub fn minutely<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.minutes())
}

/// Creates an interval for repeating events on a hourly basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::hourly;
///
/// let every_twelve_hours = hourly(12);
/// ```
#[inline]
pub fn hourly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.hours())
}

/// Creates an interval for repeating events on a daily basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::daily;
///
/// let every_two_days = daily(2);
/// ```
#[inline]
pub fn daily<I: ToSpan>(interval: I) -> Daily {
    Daily::new(interval)
}

/// Creates an interval for repeating events on a monthly basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::monthly;
///
/// let every_three_months = monthly(3);
/// ```
#[inline]
pub fn monthly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.months())
}

/// Creates an interval for repeating events on a yearly basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::yearly;
///
/// let every_five_years = yearly(5);
/// ```
#[inline]
pub fn yearly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.years())
}

fn is_aligned_to_series(instant: DateTime, bounds: &Range<DateTime>, interval: Span) -> bool {
    const ALLOWED_INTERVAL_ERROR: f64 = 0.000_001;

    intervals_until(bounds.start, instant, interval)
        .is_some_and(|intervals| intervals.fract() < ALLOWED_INTERVAL_ERROR)
}

fn align_to_series(
    instant: DateTime,
    bounds: &Range<DateTime>,
    interval: Span,
) -> Option<DateTime> {
    let intervals = get_alignment_intervals(instant, bounds, interval)?;
    bounds.start.checked_add(intervals * interval).ok()
}

fn get_alignment_intervals(
    instant: DateTime,
    bounds: &Range<DateTime>,
    interval: Span,
) -> Option<i64> {
    if instant <= bounds.start {
        return Some(0);
    }

    let end = instant.min(bounds.end);

    let mut intervals = intervals_until(bounds.start, end, interval)?;

    if end == bounds.end && intervals.round() >= intervals {
        // The series would hit the end bound exactly or due to rounding up. We need to substract
        // an interval because the series end bound is exclusive.
        intervals -= 1.0;
    }

    #[allow(clippy::cast_possible_truncation)]
    Some(intervals.round() as i64)
}

fn intervals_until(start: DateTime, end: DateTime, interval: Span) -> Option<f64> {
    let interval_seconds = span_seconds(interval)?;
    seconds_until(start, end).map(|seconds| seconds / interval_seconds)
}

fn seconds_until(start: DateTime, end: DateTime) -> Option<f64> {
    start.until(end).ok().and_then(span_seconds)
}

fn span_seconds(span: Span) -> Option<f64> {
    span.total(SpanTotal::from(Unit::Second).days_are_24_hours())
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    #[test]
    fn test_is_aligned_to_series() {
        let start = date(2025, 1, 1).at(0, 0, 0, 0);
        let bounds = start..DateTime::MAX;

        assert!(!is_aligned_to_series(
            date(2025, 1, 1).at(0, 30, 0, 0),
            &bounds,
            1.hour()
        ));
        assert!(is_aligned_to_series(
            date(2025, 1, 1).at(0, 0, 0, 0),
            &bounds,
            1.hour()
        ));
        assert!(is_aligned_to_series(
            date(2025, 1, 1).at(1, 0, 0, 0),
            &bounds,
            1.hour()
        ));

        let bounds = start..DateTime::MAX;

        assert!(is_aligned_to_series(
            date(9999, 12, 31).at(22, 0, 0, 0),
            &bounds,
            2.hour()
        ),);
    }

    #[test]
    fn test_align_to_series() {
        let start = date(2025, 1, 1).at(0, 0, 0, 0);
        let end = date(2025, 1, 3).at(0, 0, 0, 0);
        let bounds = start..end;

        assert_eq!(
            align_to_series(date(2025, 1, 1).at(0, 0, 0, 0), &bounds, 1.hour()),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            align_to_series(date(2025, 1, 1).at(0, 30, 0, 0), &bounds, 1.hour()),
            Some(date(2025, 1, 1).at(1, 0, 0, 0))
        );

        assert_eq!(
            align_to_series(
                date(2025, 1, 1)
                    .at(0, 30, 0, 0)
                    .checked_sub(1.nanosecond())
                    .unwrap(),
                &bounds,
                1.hour()
            ),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            align_to_series(date(2024, 12, 31).at(0, 30, 0, 0), &bounds, 1.hour()),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            align_to_series(date(2025, 1, 3).at(0, 0, 0, 0), &bounds, 1.hour()),
            Some(date(2025, 1, 2).at(23, 0, 0, 0))
        );

        assert_eq!(
            align_to_series(date(2025, 2, 10).at(0, 30, 0, 0), &bounds, 1.hour()),
            Some(date(2025, 1, 2).at(23, 0, 0, 0))
        );

        let bounds = start..DateTime::MAX;

        assert_eq!(
            align_to_series(DateTime::MAX, &bounds, 2.hours()),
            Some(date(9999, 12, 31).at(22, 0, 0, 0))
        );
    }
}
