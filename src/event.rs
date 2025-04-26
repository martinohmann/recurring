use crate::{Error, Repeat, Series};
use jiff::{SignedDuration, Span, civil::DateTime};

/// Represents an event that happens at a given point in time and may span until an optional end
/// datetime.
///
/// Single instant events can be created via [`Event::at`], while [`Event::new`] can be used to
/// construct events with an explict end.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// # use recurring::Event;
/// use jiff::SignedDuration;
/// use jiff::civil::date;
///
/// let start = date(2025, 1, 1).at(0, 0, 0, 0);
/// let end = date(2025, 1, 2).at(0, 0, 0, 0);
/// let event = Event::new(start, end)?;
/// assert_eq!(event.start(), start);
/// assert_eq!(event.end(), Some(end));
/// assert_eq!(event.duration(), Some(SignedDuration::from_hours(24)));
/// # Ok(())
/// # }
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
    /// # use recurring::Event;
    /// use jiff::civil::date;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let event = Event::at(start);
    /// ```
    pub fn at(instant: DateTime) -> Event {
        Event {
            start: instant,
            end: None,
        }
    }

    /// Creates a new `Event` which span from a `start` (inclusive) to an `end` (exclusive).
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidEventEnd` if `start >= end`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # use recurring::Event;
    /// use jiff::civil::date;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let end = date(2025, 1, 2).at(0, 0, 0, 0);
    /// let event = Event::new(start, end)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(start: DateTime, end: DateTime) -> Result<Event, Error> {
        if start >= end {
            return Err(Error::InvalidEventEnd);
        }

        Ok(Event {
            start,
            end: Some(end),
        })
    }

    /// Returns the `DateTime` at which the event starts.
    ///
    /// # Example
    ///
    /// ```
    /// # use recurring::Event;
    /// use jiff::civil::date;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let event = Event::at(start);
    /// assert_eq!(event.start(), start);
    /// ```
    pub fn start(&self) -> DateTime {
        self.start
    }

    /// Returns the `DateTime` at which the event end if it has an end, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # use recurring::Event;
    /// use jiff::civil::date;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let event = Event::at(start);
    /// assert!(event.end().is_none());
    ///
    /// let end = date(2025, 1, 2).at(0, 0, 0, 0);
    /// let event = Event::new(start, end)?;
    /// assert_eq!(event.end(), Some(end));
    /// # Ok(())
    /// # }
    /// ```
    pub fn end(&self) -> Option<DateTime> {
        self.end
    }

    /// Returns the duration between the events' start and end if it has an end, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # use recurring::Event;
    /// use jiff::SignedDuration;
    /// use jiff::civil::date;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let event = Event::at(start);
    /// assert!(event.duration().is_none());
    ///
    /// let end = date(2025, 1, 2).at(0, 0, 0, 0);
    /// let event = Event::new(start, end)?;
    /// assert_eq!(event.duration(), Some(SignedDuration::from_hours(24)));
    /// # Ok(())
    /// # }
    /// ```
    pub fn duration(&self) -> Option<SignedDuration> {
        self.end.map(|end| self.start.duration_until(end))
    }

    /// Returns `true` if `instant` falls within the events' duration, `false` otherwise.
    ///
    /// For events that don't have an end, this is equivalent to `event.start() == instant`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # use recurring::Event;
    /// use jiff::ToSpan;
    /// use jiff::civil::date;
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let event = Event::at(start);
    /// assert!(!event.contains(start - 1.nanosecond()));
    /// assert!(event.contains(start));
    /// assert!(!event.contains(start + 1.nanosecond()));
    ///
    /// let end = date(2025, 1, 2).at(0, 0, 0, 0);
    /// let event = Event::new(start, end)?;
    /// assert!(!event.contains(start - 1.nanosecond()));
    /// assert!(event.contains(start));
    /// assert!(event.contains(start + 1.nanosecond()));
    /// assert!(event.contains(end - 1.nanosecond()));
    /// assert!(!event.contains(end));
    /// assert!(!event.contains(end + 1.nanosecond()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn contains(&self, instant: DateTime) -> bool {
        if let Some(end) = self.end {
            instant >= self.start && instant < end
        } else {
            instant == self.start
        }
    }

    /// Converts an event to a `Series` with the given [`Repeat`] interval.
    ///
    /// # Errors
    ///
    /// Returns an error if the event duration cannot be represented as a [`Span`] or if the
    /// events' `start` is `DateTime::MAX`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # extern crate alloc;
    /// # use alloc::vec::Vec;
    /// # use recurring::Event;
    /// use recurring::repeat::hourly;
    /// use jiff::ToSpan;
    /// use jiff::civil::date;
    ///
    /// let date = date(2025, 1, 1);
    /// let start = date.at(0, 0, 0, 0);
    /// let end = date.at(0, 30, 0, 0);
    ///
    /// let event = Event::new(start, end)?;
    /// let series = event.to_series(hourly(2))?;
    ///
    /// let mut events = series.iter();
    ///
    /// assert_eq!(events.next(), Some(Event::new(date.at(0, 0, 0, 0), date.at(0, 30, 0, 0))?));
    /// assert_eq!(events.next(), Some(Event::new(date.at(2, 0, 0, 0), date.at(2, 30, 0, 0))?));
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_series<R>(&self, repeat: R) -> Result<Series<R>, Error>
    where
        R: Repeat,
    {
        let mut builder = Series::builder().start(self.start);

        if let Some(duration) = self.duration() {
            let event_duration =
                Span::try_from(duration).map_err(|_| Error::InvalidEventDuration)?;
            builder = builder.event_duration(event_duration);
        }

        builder.build(repeat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    #[test]
    fn event_end_before_start() {
        let start = date(2025, 1, 2).at(0, 0, 0, 0);
        let end = date(2025, 1, 1).at(0, 0, 0, 0);
        assert!(Event::new(start, end).is_err());
    }
}
