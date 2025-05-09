use crate::{
    Error, Repeat,
    error::ErrorKind,
    repeat::utils::{intervals_in_range_until, is_interval_boundary},
};
use core::ops::Range;
use jiff::{Span, civil::DateTime};

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
pub struct Interval {
    span: Span,
}

impl Interval {
    /// Creates a new `Interval` from a `Span`.
    ///
    /// For a fallible alternative see [`Interval::try_new`].
    ///
    /// # Panics
    ///
    /// Panics if `span` is negative or zero, or contains non-zero units smaller than seconds.
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
        assert!(
            Interval::validate(span),
            "interval must be positive, non-zero and not include sub-second units"
        );
        Interval { span }
    }

    /// Creates a new `Interval` from a `Span`.
    ///
    /// For an infallible alternative that panics on invalid spans instead see [`Interval::new`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if `span` is negative or zero, or contains non-zero units smaller than
    /// seconds.
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
    /// assert!(Interval::try_new(1.nanosecond()).is_err());
    /// ```
    pub fn try_new(span: Span) -> Result<Interval, Error> {
        if !Interval::validate(span) {
            return Err(Error::from(ErrorKind::InvalidInterval));
        }

        Ok(Interval { span })
    }

    fn validate(span: Span) -> bool {
        span.is_positive()
            && span.get_milliseconds() == 0
            && span.get_microseconds() == 0
            && span.get_nanoseconds() == 0
    }
}

impl Repeat for Interval {
    fn next_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        if instant < range.start {
            // We want the range start if instant happens before that.
            return Some(range.start);
        }

        let mut intervals = intervals_in_range_until(self.span, range, instant)?;

        if is_interval_boundary(intervals) {
            // We want the next event.
            intervals += 1.0;
        }

        #[allow(clippy::cast_possible_truncation)] // Already rounded.
        let n = intervals.ceil() as i64;
        range
            .start
            .checked_add(n * self.span)
            .ok()
            .filter(|&event| event < range.end)
    }

    fn previous_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        if instant <= range.start {
            return None;
        }

        let mut intervals = intervals_in_range_until(self.span, range, instant)?;

        if is_interval_boundary(intervals) {
            // We want the previous event.
            intervals -= 1.0;
        }

        #[allow(clippy::cast_possible_truncation)] // Already rounded.
        let n = intervals.floor() as i64;
        range
            .start
            .checked_add(n * self.span)
            .ok()
            .filter(|&event| event >= range.start)
    }

    fn closest_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let intervals = intervals_in_range_until(self.span, range, instant)?;
        let mut intervals_rounded = intervals.round();

        if instant >= range.end && intervals_rounded >= intervals {
            // The series would hit the end bound exactly or due to rounding up. We want the last
            // event in the series in this case because the range end is excluded from the series.
            intervals_rounded -= 1.0;
        }

        #[allow(clippy::cast_possible_truncation)] // Already rounded.
        let n = intervals_rounded as i64;
        range.start.checked_add(n * self.span).ok()
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        self.span.fieldwise() == other.span
    }
}

impl PartialEq<&Interval> for Interval {
    fn eq(&self, other: &&Self) -> bool {
        self.span.fieldwise() == other.span
    }
}

impl Eq for Interval {}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::ToSpan;
    use jiff::civil::date;

    #[test]
    fn closest_event() {
        let start = date(2025, 1, 1).at(0, 0, 0, 0);
        let end = date(2025, 1, 3).at(0, 0, 0, 0);
        let range = start..end;
        let interval = Interval::new(1.hour());

        assert_eq!(
            interval.closest_event(date(2025, 1, 1).at(0, 0, 0, 0), &range),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            interval.closest_event(date(2025, 1, 1).at(0, 30, 0, 0), &range),
            Some(date(2025, 1, 1).at(1, 0, 0, 0))
        );

        assert_eq!(
            interval.closest_event(
                date(2025, 1, 1)
                    .at(0, 30, 0, 0)
                    .checked_sub(1.nanosecond())
                    .unwrap(),
                &range,
            ),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            interval.closest_event(date(2024, 12, 31).at(0, 30, 0, 0), &range),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            interval.closest_event(date(2025, 1, 3).at(0, 0, 0, 0), &range),
            Some(date(2025, 1, 2).at(23, 0, 0, 0))
        );

        assert_eq!(
            interval.closest_event(date(2025, 2, 10).at(0, 30, 0, 0), &range),
            Some(date(2025, 1, 2).at(23, 0, 0, 0))
        );
    }

    #[test]
    fn closest_event_datetime_max() {
        let start = date(2025, 1, 1).at(0, 0, 0, 0);
        let range = start..DateTime::MAX;
        let interval = Interval::new(2.hours());

        assert_eq!(
            interval.closest_event(DateTime::MAX, &range),
            Some(date(9999, 12, 31).at(22, 0, 0, 0))
        );
    }

    #[test]
    fn next_event() {
        let start = date(2025, 1, 1).at(0, 0, 0, 0);
        let end = date(2025, 1, 3).at(0, 0, 0, 0);
        let range = start..end;
        let interval = Interval::new(1.hour());
        assert_eq!(
            interval.next_event(DateTime::MIN, &range),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );
        assert_eq!(
            interval.next_event(range.start.checked_sub(1.nanosecond()).unwrap(), &range,),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            interval.next_event(range.start, &range),
            Some(date(2025, 1, 1).at(1, 0, 0, 0))
        );

        assert_eq!(
            interval.next_event(date(2025, 1, 1).at(0, 30, 0, 0), &range),
            Some(date(2025, 1, 1).at(1, 0, 0, 0))
        );

        assert_eq!(
            interval.next_event(range.end - 1.hour().nanoseconds(1), &range),
            Some(date(2025, 1, 2).at(23, 0, 0, 0))
        );
        assert_eq!(interval.next_event(range.end - 1.hour(), &range), None,);
        assert_eq!(interval.next_event(range.end, &range), None);
        assert_eq!(interval.next_event(DateTime::MAX, &range), None);
    }

    #[test]
    fn previous_event() {
        let start = date(2025, 1, 1).at(0, 0, 0, 0);
        let end = date(2025, 1, 3).at(0, 0, 0, 0);
        let range = start..end;
        let interval = Interval::new(1.hour());
        assert_eq!(interval.previous_event(DateTime::MIN, &range), None);
        assert_eq!(interval.previous_event(range.start, &range), None);
        assert_eq!(
            interval.previous_event(range.start + 1.second(), &range),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            interval.previous_event(date(2025, 1, 1).at(0, 30, 0, 0), &range),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            interval.previous_event(date(2025, 1, 3).at(0, 0, 0, 0), &range),
            Some(date(2025, 1, 2).at(23, 0, 0, 0))
        );

        assert_eq!(
            interval.previous_event(range.end, &range),
            Some(date(2025, 1, 2).at(23, 0, 0, 0))
        );
    }
}
