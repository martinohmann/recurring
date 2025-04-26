use crate::Error;
use jiff::{SignedDuration, civil::DateTime};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    start: DateTime,
    end: Option<DateTime>,
}

impl Event {
    /// Creates a new `Event` which starts and ends at `instant`.
    ///
    /// The event duration is effectively zero.
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
    pub fn new(start: DateTime, end: DateTime) -> Result<Event, Error> {
        if start >= end {
            return Err(Error::InvalidEventEnd);
        }

        Ok(Event {
            start,
            end: Some(end),
        })
    }

    pub fn start(&self) -> DateTime {
        self.start
    }

    pub fn end(&self) -> Option<DateTime> {
        self.end
    }

    pub fn duration(&self) -> Option<SignedDuration> {
        self.end.map(|end| self.start.duration_until(end))
    }

    pub fn contains(&self, instant: DateTime) -> bool {
        if let Some(end) = self.end {
            instant >= self.start && instant < end
        } else {
            instant == self.start
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::{ToSpan, civil::datetime};

    #[test]
    fn event() {
        let start = datetime(2025, 1, 1, 0, 0, 0, 0);
        let event = Event::at(start);
        assert_eq!(event.start(), start);
        assert!(event.end().is_none());
        assert!(event.duration().is_none());

        let end = datetime(2025, 1, 2, 0, 0, 0, 0);
        let event = Event::new(start, end).unwrap();
        assert_eq!(event.start(), start);
        assert_eq!(event.end(), Some(end));
        assert_eq!(event.duration(), Some(SignedDuration::from_hours(24)));
    }

    #[test]
    fn event_end_before_start() {
        let start = datetime(2025, 1, 2, 0, 0, 0, 0);
        let end = datetime(2025, 1, 1, 0, 0, 0, 0);
        assert!(Event::new(start, end).is_err());
    }

    #[test]
    fn event_contains() {
        let start = datetime(2025, 1, 1, 0, 0, 0, 0);
        let event = Event::at(start);
        assert!(event.contains(start));
        assert!(!event.contains(start - 1.nanosecond()));
        assert!(!event.contains(start + 1.nanosecond()));

        let end = datetime(2025, 1, 2, 0, 0, 0, 0);
        let event = Event::new(start, end).unwrap();
        assert!(event.contains(start));
        assert!(event.contains(start + 1.nanosecond()));
        assert!(event.contains(end - 1.nanosecond()));
        assert!(!event.contains(start - 1.nanosecond()));
        assert!(!event.contains(end));
        assert!(!event.contains(end + 1.nanosecond()));
    }
}
