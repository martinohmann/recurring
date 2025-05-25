use crate::error::{Error, err};
use crate::pattern::utils::{advance_by_until, closest_to, pick_best};
use crate::{Pattern, SeriesRange, private};
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
    /// Panics if `span` is negative or zero.
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
    /// The panicking version of this method is [`Interval::new`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if `span` is negative or zero.
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
    /// ```
    pub fn try_new(span: Span) -> Result<Interval, Error> {
        if !span.is_positive() {
            return Err(err!("interval must be positive but got {span}"));
        }

        Ok(Interval { span, offset: None })
    }

    /// Set the offset from the series start at which the first recurrence happens.
    ///
    /// The fallible version of this method is [`Interval::try_offset`].
    ///
    /// # Panics
    ///
    /// Panics if `offset` is negative.
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
    /// Returns an `Error` if `offset` is negative.
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
    /// // Negative offsets are not allowed.
    /// assert!(Interval::new(2.hours()).try_offset(-1.hour()).is_err());
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    pub fn try_offset(mut self, offset: Span) -> Result<Interval, Error> {
        if offset.is_negative() {
            return Err(err!("offset must be zero or positive but got {offset}"));
        }

        self.offset = Some(offset);
        Ok(self)
    }

    fn add_offset(&self, instant: DateTime) -> Option<DateTime> {
        let Some(offset) = self.offset else {
            return Some(instant);
        };

        instant.checked_add(offset).ok()
    }
}

impl Pattern for Interval {
    fn next_after(&self, instant: DateTime, range: SeriesRange) -> Option<DateTime> {
        let start = self.add_offset(range.start)?;
        if start >= range.end {
            return None;
        }

        let fixpoint = self.add_offset(range.fixpoint())?;

        if instant < start {
            // We want the first event in the series, which may be the series start or a later
            // point within the series.

            if start == fixpoint {
                // Fast path: we can just return `start` here.
                return Some(start);
            }

            // Slow path: first event is relative to `fixpoint`, we need to compute it.
            let date = advance_by_until(fixpoint, self.span, start);
            if date >= start {
                return Some(date);
            }

            return date
                .checked_add(self.span)
                .ok()
                .filter(|&next| next < range.end);
        }

        // Normal path: advance to the next event.
        let date = advance_by_until(fixpoint, self.span, instant.min(range.end));
        if date == range.end {
            return None;
        }

        date.checked_add(self.span)
            .ok()
            .filter(|&next| next < range.end)
    }

    fn previous_before(&self, instant: DateTime, range: SeriesRange) -> Option<DateTime> {
        let start = self.add_offset(range.start)?;
        if instant <= start || start >= range.end {
            return None;
        }

        let fixpoint = self.add_offset(range.fixpoint())?;
        let date = advance_by_until(fixpoint, self.span, instant.min(range.end));
        if date < instant {
            return Some(date);
        }

        date.checked_sub(self.span)
            .ok()
            .filter(|&prev| prev >= start)
    }

    fn closest_to(&self, instant: DateTime, range: SeriesRange) -> Option<DateTime> {
        let start = self.add_offset(range.start)?;
        if start >= range.end {
            return None;
        }

        let fixpoint = self.add_offset(range.fixpoint())?;
        let date = advance_by_until(fixpoint, self.span, instant.max(start).min(range.end));

        let prev = if date == range.end {
            date.checked_sub(self.span).ok()
        } else {
            Some(date)
        }
        .filter(|&prev| prev >= start);

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
