use jiff::{SignedDuration, civil::DateTime};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    start: DateTime,
    end: Option<DateTime>,
}

impl Event {
    pub fn at(instant: DateTime) -> Event {
        Event {
            start: instant,
            end: None,
        }
    }

    /// # Panics
    ///
    /// If `end < start`, this function panics.
    pub fn new(start: DateTime, end: DateTime) -> Event {
        Event::at(start).ends_at(end)
    }

    /// # Panics
    ///
    /// If `end` is less than the `Event`'s `start`, this method panics.
    #[must_use]
    pub fn ends_at(self, end: DateTime) -> Event {
        assert!(end >= self.start, "event end < start");
        Event {
            start: self.start,
            end: Some(end),
        }
    }

    pub fn start(&self) -> DateTime {
        self.start
    }

    pub fn end(&self) -> Option<DateTime> {
        self.end
    }

    pub fn duration(&self) -> SignedDuration {
        self.end
            .map_or(SignedDuration::ZERO, |end| self.start.duration_until(end))
    }

    pub fn contains(&self, instant: DateTime) -> bool {
        match self.end {
            Some(end) => instant >= self.start && instant < end,
            None => instant == self.start,
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
        assert_eq!(event.duration(), SignedDuration::ZERO);

        let end = datetime(2025, 1, 2, 0, 0, 0, 0);
        let event = event.ends_at(end);
        assert_eq!(event.start(), start);
        assert_eq!(event.end(), Some(end));
        assert_eq!(event.duration(), SignedDuration::from_hours(24));
    }

    #[test]
    #[should_panic]
    fn event_end_before_start() {
        let start = datetime(2025, 1, 2, 0, 0, 0, 0);
        let end = datetime(2025, 1, 1, 0, 0, 0, 0);
        Event::new(start, end);
    }

    #[test]
    fn event_contains() {
        let start = datetime(2025, 1, 1, 0, 0, 0, 0);
        let event = Event::at(start);
        assert!(event.contains(start));
        assert!(!event.contains(start - 1.nanosecond()));
        assert!(!event.contains(start + 1.nanosecond()));

        let end = datetime(2025, 1, 2, 0, 0, 0, 0);
        let event = event.ends_at(end);
        assert!(event.contains(start));
        assert!(event.contains(start + 1.nanosecond()));
        assert!(event.contains(end - 1.nanosecond()));
        assert!(!event.contains(start - 1.nanosecond()));
        assert!(!event.contains(end));
        assert!(!event.contains(end + 1.nanosecond()));
    }
}
