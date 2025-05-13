use crate::{
    Error, Repeat,
    error::ErrorKind,
    private,
    repeat::utils::{is_period_boundary, periods_in_range_until},
};
use core::ops::Range;
use jiff::{Span, civil::DateTime};

/// A precise period for repeating events.
///
/// # Example
///
/// ```
/// use jiff::ToSpan;
/// use recurring::repeat::Period;
///
/// let every_two_hours = Period::new(2.hours());
/// ```
#[derive(Debug, Clone)]
pub struct Period {
    span: Span,
}

impl Period {
    /// Creates a new `Period` from a `Span`.
    ///
    /// For a fallible alternative see [`Period::try_new`].
    ///
    /// # Panics
    ///
    /// Panics if `span` is negative or zero, or contains non-zero units smaller than seconds.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::ToSpan;
    /// use recurring::repeat::Period;
    ///
    /// let every_two_hours = Period::new(2.hours());
    /// ```
    pub fn new(span: Span) -> Period {
        assert!(
            Period::validate(span),
            "period must be positive, non-zero and not include sub-second units"
        );
        Period { span }
    }

    /// Creates a new `Period` from a `Span`.
    ///
    /// For an infallible alternative that panics on invalid spans instead see [`Period::new`].
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
    /// use recurring::repeat::Period;
    ///
    /// assert!(Period::try_new(1.day()).is_ok());
    /// assert!(Period::try_new(0.seconds()).is_err());
    /// assert!(Period::try_new(-1.day()).is_err());
    /// assert!(Period::try_new(1.nanosecond()).is_err());
    /// ```
    pub fn try_new(span: Span) -> Result<Period, Error> {
        if !Period::validate(span) {
            return Err(Error::from(ErrorKind::InvalidPeriod));
        }

        Ok(Period { span })
    }

    fn validate(span: Span) -> bool {
        span.is_positive()
            && span.get_milliseconds() == 0
            && span.get_microseconds() == 0
            && span.get_nanoseconds() == 0
    }
}

impl Repeat for Period {
    fn next_after(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        if instant < range.start {
            // We want the range start if instant happens before that.
            return Some(range.start);
        }

        let mut periods = periods_in_range_until(self.span, range, instant)?;

        if is_period_boundary(periods) {
            // We want the next event.
            periods += 1.0;
        }

        #[allow(clippy::cast_possible_truncation)] // Already rounded.
        let n = periods.ceil() as i64;
        range
            .start
            .checked_add(n * self.span)
            .ok()
            .filter(|&event| event < range.end)
    }

    fn previous_before(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        if instant <= range.start {
            return None;
        }

        let mut periods = periods_in_range_until(self.span, range, instant)?;

        if is_period_boundary(periods) {
            // We want the previous event.
            periods -= 1.0;
        }

        #[allow(clippy::cast_possible_truncation)] // Already rounded.
        let n = periods.floor() as i64;
        range
            .start
            .checked_add(n * self.span)
            .ok()
            .filter(|&event| event >= range.start)
    }

    fn closest_to(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let periods = periods_in_range_until(self.span, range, instant)?;
        let mut periods_rounded = periods.round();

        if instant >= range.end && periods_rounded >= periods {
            // The series would hit the end bound exactly or due to rounding up. We want the last
            // event in the series in this case because the range end is excluded from the series.
            periods_rounded -= 1.0;
        }

        #[allow(clippy::cast_possible_truncation)] // Already rounded.
        let n = periods_rounded as i64;
        range.start.checked_add(n * self.span).ok()
    }
}

impl private::Sealed for Period {}

impl PartialEq for Period {
    fn eq(&self, other: &Self) -> bool {
        self.span.fieldwise() == other.span
    }
}

impl PartialEq<&Period> for Period {
    fn eq(&self, other: &&Self) -> bool {
        self.span.fieldwise() == other.span
    }
}

impl Eq for Period {}
