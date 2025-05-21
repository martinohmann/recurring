use crate::error::Error;
use core::fmt;
use jiff::{Span, civil::DateTime};

/// Represents an event that happens at a given point in time and may span until an optional end
/// datetime.
///
/// Single instant events can be created via [`Event::at`], while [`Event::new`] and
/// [`Event::try_new`] can be used to construct events with an explict end.
///
/// # Example
///
/// ```
/// use jiff::{ToSpan, civil::date};
/// use recurring::Event;
///
/// let start = date(2025, 1, 1).at(0, 0, 0, 0);
/// let end = date(2025, 1, 2).at(0, 0, 0, 0);
/// let event = Event::new(start, end);
/// assert_eq!(event.start(), start);
/// assert_eq!(event.end(), Some(end));
/// assert_eq!(event.duration().fieldwise(), 1.day());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    start: DateTime,
    end: Option<DateTime>,
}

impl Event {
    /// Creates a new `Event` which starts and ends at `instant`.
    ///
    /// The event duration is effectively zero.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::Event;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let event = Event::at(start);
    /// ```
    #[inline]
    pub fn at(instant: DateTime) -> Event {
        Event {
            start: instant,
            end: None,
        }
    }

    /// Creates a new `Event` which spans from a `start` (inclusive) to an `end` (exclusive).
    ///
    /// The fallible version of this method is [`Event::try_new`].
    ///
    /// # Panics
    ///
    /// Panics if `start >= end`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::Event;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let end = date(2025, 1, 2).at(0, 0, 0, 0);
    /// let event = Event::new(start, end);
    /// ```
    pub fn new(start: DateTime, end: DateTime) -> Event {
        Event::try_new(start, end).expect("invalid event end")
    }

    /// Creates a new `Event` which spans from a `start` (inclusive) to an `end` (exclusive).
    ///
    /// The packicking version of this method is [`Event::new`].
    ///
    /// # Errors
    ///
    /// Returns and `Error` if `start >= end`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::Event;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let end = date(2025, 1, 2).at(0, 0, 0, 0);
    /// let event = Event::try_new(start, end)?;
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    pub fn try_new(start: DateTime, end: DateTime) -> Result<Event, Error> {
        if start >= end {
            return Err(Error::datetime_range("event", start..end));
        }

        Ok(Event::new_unchecked(start, end))
    }

    /// Creates a new `Event` which spans from a `start` (inclusive) to an `end` (exclusive)
    /// without checking that `end` is strictly greater than `start`.
    #[inline]
    pub(crate) fn new_unchecked(start: DateTime, end: DateTime) -> Event {
        Event {
            start,
            end: Some(end),
        }
    }

    /// Returns the `DateTime` at which the event starts.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::Event;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let event = Event::at(start);
    /// assert_eq!(event.start(), start);
    /// ```
    pub fn start(&self) -> DateTime {
        self.start
    }

    /// Returns the `DateTime` at which the event ends if it has an end, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::date;
    /// use recurring::Event;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let event = Event::at(start);
    /// assert!(event.end().is_none());
    ///
    /// let end = date(2025, 1, 2).at(0, 0, 0, 0);
    /// let event = Event::new(start, end);
    /// assert_eq!(event.end(), Some(end));
    /// ```
    pub fn end(&self) -> Option<DateTime> {
        self.end
    }

    /// Returns the duration between the events' start and end.
    ///
    /// For events that don't have an end, this always returns a zero `Span`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::{ToSpan, civil::date};
    /// use recurring::Event;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let event = Event::at(start);
    /// assert!(event.duration().is_zero());
    ///
    /// let end = date(2025, 1, 2).at(0, 0, 0, 0);
    /// let event = Event::new(start, end);
    /// assert_eq!(event.duration().fieldwise(), 1.day());
    /// ```
    pub fn duration(&self) -> Span {
        self.end
            .and_then(|end| self.start.until(end).ok())
            .unwrap_or_default()
    }

    /// Returns `true` if `instant` falls within the events' duration, `false` otherwise.
    ///
    /// For events that don't have an end, this is equivalent to `event.start() == instant`.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::{ToSpan, civil::date};
    /// use recurring::Event;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let event = Event::at(start);
    /// assert!(!event.contains(start - 1.nanosecond()));
    /// assert!(event.contains(start));
    /// assert!(!event.contains(start + 1.nanosecond()));
    ///
    /// let end = date(2025, 1, 2).at(0, 0, 0, 0);
    /// let event = Event::new(start, end);
    /// assert!(!event.contains(start - 1.nanosecond()));
    /// assert!(event.contains(start));
    /// assert!(event.contains(start + 1.nanosecond()));
    /// assert!(event.contains(end - 1.nanosecond()));
    /// assert!(!event.contains(end));
    /// assert!(!event.contains(end + 1.nanosecond()));
    /// ```
    pub fn contains(&self, instant: DateTime) -> bool {
        if let Some(end) = self.end {
            instant >= self.start && instant < end
        } else {
            instant == self.start
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.start.fmt(f)?;
        if let Some(end) = &self.end {
            f.write_str(" - ")?;
            end.fmt(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use jiff::civil::date;

    #[test]
    fn event_end_before_start() {
        let start = date(2025, 1, 2).at(0, 0, 0, 0);
        let end = date(2025, 1, 1).at(0, 0, 0, 0);
        assert!(Event::try_new(start, end).is_err());
    }

    #[test]
    fn event_display() {
        assert_eq!(
            Event::at(date(2025, 1, 1).at(0, 0, 0, 0)).to_string(),
            "2025-01-01T00:00:00"
        );
        assert_eq!(
            Event::new(
                date(2025, 1, 1).at(0, 0, 0, 0),
                date(2025, 1, 1).at(12, 0, 0, 0)
            )
            .to_string(),
            "2025-01-01T00:00:00 - 2025-01-01T12:00:00"
        );
    }
}
