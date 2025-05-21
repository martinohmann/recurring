use super::utils::{FloatExt, spans_until};
use crate::{
    Pattern,
    error::{Error, err},
    private,
};
use core::ops::Range;
use jiff::{Span, civil::DateTime};

/// A fixed interval recurrence pattern.
///
/// # Example
///
/// ```
/// use jiff::ToSpan;
/// use recurring::pattern::Interval;
///
/// let every_two_hours = Interval::new(2.hours());
/// ```
#[derive(Debug, Clone)]
pub struct Interval {
    span: Span,
    offset: Option<Span>,
}

impl Interval {
    /// Creates a new `Interval` from a `Span`.
    ///
    /// The fallible version of this method is [`Interval::try_new`].
    ///
    /// # Panics
    ///
    /// Panics if `span` is negative or zero, or contains non-zero units smaller than seconds.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::ToSpan;
    /// use recurring::pattern::Interval;
    ///
    /// let every_two_hours = Interval::new(2.hours());
    /// ```
    #[inline]
    pub fn new(span: Span) -> Interval {
        Interval::try_new(span).expect("invalid interval span")
    }

    /// Creates a new `Interval` from a `Span`.
    ///
    /// The packicking version of this method is [`Interval::new`].
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
    /// use recurring::pattern::Interval;
    ///
    /// assert!(Interval::try_new(1.day()).is_ok());
    /// assert!(Interval::try_new(0.seconds()).is_err());
    /// assert!(Interval::try_new(-1.day()).is_err());
    /// assert!(Interval::try_new(1.nanosecond()).is_err());
    /// ```
    pub fn try_new(span: Span) -> Result<Interval, Error> {
        if !Interval::validate(span) {
            return Err(err!(
                "interval must be positive, non-zero and must not include sub-second units but got {span}"
            ));
        }

        Ok(Interval { span, offset: None })
    }

    /// Set the offset from the series start at which the first recurrence happens.
    ///
    /// The fallible version of this method is [`Interval::try_offset`].
    ///
    /// # Panics
    ///
    /// Panics if `offset` is negative or zero, or contains non-zero units smaller than seconds.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::ToSpan;
    /// use recurring::pattern::Interval;
    ///
    /// // The first event of this pattern will be 2h 30m after the series start, the second event
    /// // will be 4h 30m after the start.
    /// let interval = Interval::new(2.hours()).offset(30.minutes());
    /// ```
    #[inline]
    #[must_use]
    pub fn offset(self, offset: Span) -> Interval {
        self.try_offset(offset).expect("invalid offset span")
    }

    /// Set the offset from the series start at which the first recurrence happens.
    ///
    /// The panicking version of this method is [`Interval::offset`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if `offset` is negative or zero, or contains non-zero units smaller than
    /// seconds.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::ToSpan;
    /// use recurring::pattern::Interval;
    ///
    /// // The first event of this pattern will be 2h 30m after the series start, the second event
    /// // will be 4h 30m after the start.
    /// let interval = Interval::new(2.hours()).try_offset(30.minutes())?;
    ///
    /// // Negative or too small offsets are not allowed.
    /// assert!(Interval::new(2.hours()).try_offset(0.seconds()).is_err());
    /// assert!(Interval::new(2.hours()).try_offset(10.nanoseconds()).is_err());
    /// assert!(Interval::new(2.hours()).try_offset(-1.hour()).is_err());
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    pub fn try_offset(mut self, offset: Span) -> Result<Interval, Error> {
        if !Interval::validate(offset) {
            return Err(err!(
                "offset must be positive, non-zero and must not include sub-second units but got {offset}"
            ));
        }

        self.offset = Some(offset);
        Ok(self)
    }

    fn validate(span: Span) -> bool {
        span.is_positive()
            && span.get_milliseconds() == 0
            && span.get_microseconds() == 0
            && span.get_nanoseconds() == 0
    }

    fn add_offset(&self, instant: DateTime) -> Option<DateTime> {
        let Some(offset) = self.offset else {
            return Some(instant);
        };

        instant.checked_add(offset).ok()
    }
}

#[allow(clippy::cast_possible_truncation)] // We only cast floats with zero fractional part to i64.
impl Pattern for Interval {
    fn next_after(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let start = self.add_offset(range.start)?;
        if start >= range.end {
            return None;
        }

        if instant < start {
            // We want the start if instant happens before that.
            return Some(start);
        }

        let spans = spans_until(self.span, start, instant.min(range.end))?;
        let count = spans.ceil_strict();

        start
            .checked_add(count as i64 * self.span)
            .ok()
            .filter(|&next| next < range.end)
    }

    fn previous_before(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let start = self.add_offset(range.start)?;
        if instant <= start || start >= range.end {
            return None;
        }

        let spans = spans_until(self.span, start, instant.min(range.end))?;
        let count = spans.floor_strict();

        start.checked_add(count as i64 * self.span).ok()
    }

    fn closest_to(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let start = self.add_offset(range.start)?;
        if start >= range.end {
            return None;
        }

        let end = instant.max(start).min(range.end);
        let spans = spans_until(self.span, start, end)?;

        let count = if end == range.end && spans.round() >= spans {
            // The series would hit the end bound exactly or due to rounding up. We want the last
            // event in the series in this case because the range end is excluded from the series.
            spans.floor_strict()
        } else {
            spans.round()
        };

        start.checked_add(count as i64 * self.span).ok()
    }
}

impl private::Sealed for Interval {}

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
