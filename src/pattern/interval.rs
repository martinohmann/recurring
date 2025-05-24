use crate::error::{Error, err};
use crate::pattern::utils::{advance_by_until, closest_to, pick_best};
use crate::{Pattern, private};
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

        let date = advance_by_until(start, self.span, instant.min(range.end));
        if date == range.end {
            return None;
        }

        date.checked_add(self.span)
            .ok()
            .filter(|&next| next < range.end)
    }

    fn previous_before(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let start = self.add_offset(range.start)?;
        if instant <= start || start >= range.end {
            return None;
        }

        let date = advance_by_until(start, self.span, instant.min(range.end));
        if date < instant {
            return Some(date);
        }

        date.checked_sub(self.span)
            .ok()
            .filter(|&prev| prev >= start)
    }

    fn closest_to(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let start = self.add_offset(range.start)?;
        if start >= range.end {
            return None;
        }

        let date = advance_by_until(start, self.span, instant.max(start).min(range.end));

        let prev = if date == range.end {
            date.checked_sub(self.span)
                .ok()
                .filter(|&prev| prev >= start)
        } else {
            Some(date)
        };

        let next = date
            .checked_add(self.span)
            .ok()
            .filter(|&next| next < range.end);

        pick_best(prev, next, |prev, next| closest_to(instant, prev, next))
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
