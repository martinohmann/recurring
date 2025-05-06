use crate::{Error, Repeat, error::ErrorKind, repeat::utils::closest_event};
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
            return Err(Error::from(ErrorKind::InvalidInterval));
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

    fn closest_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        closest_event(self.0, instant, range)
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        self.0.fieldwise() == other.0
    }
}

impl PartialEq<&Interval> for Interval {
    fn eq(&self, other: &&Self) -> bool {
        self.0.fieldwise() == other.0
    }
}

impl Eq for Interval {}
