use crate::{event::Event, repeat::Repeat};
use jiff::{Span, civil::DateTime};

#[derive(Debug, Clone)]
pub struct Series<R> {
    repeat: R,
    start: DateTime,
    end: DateTime,
    duration: Option<Span>,
}

impl<R> Series<R>
where
    R: Repeat,
{
    pub fn new(
        repeat: R,
        start: DateTime,
        end: Option<DateTime>,
        duration: Option<Span>,
    ) -> Series<R> {
        Series {
            repeat,
            start,
            end: end.unwrap_or(DateTime::MAX),
            duration,
        }
    }

    pub fn iter(&self) -> Iter<'_, R> {
        Iter::new(self)
    }

    pub fn first(&self) -> Option<Event> {
        if let Some(event) = self.event_at(self.start) {
            return Some(event);
        }

        let start = self.repeat.next_event(self.start)?;

        self.event_at_unchecked(start)
    }

    pub fn last(&self) -> Option<Event> {
        let start = self.repeat.previous_event(self.end)?;

        self.event_at_unchecked(start)
    }

    pub fn has_event_at(&self, instant: DateTime) -> bool {
        instant >= self.start
            && instant < self.end
            && self.repeat.aligns_with_series(instant, self.start)
    }

    pub fn event_at(&self, start: DateTime) -> Option<Event> {
        if !self.has_event_at(start) {
            return None;
        }

        self.event_at_unchecked(start)
    }

    fn event_at_unchecked(&self, start: DateTime) -> Option<Event> {
        let mut event = Event::at(start);

        if let Some(duration) = self.duration {
            let end = start.checked_add(duration).ok()?;
            event = event.ends_at(end);
        }

        Some(event)
    }

    pub fn event_containing(&self, instant: DateTime) -> Option<Event> {
        if let Some(event) = self.event_at(instant) {
            return Some(event);
        }

        let previous = self.event_before(instant)?;

        if previous.contains(instant) {
            Some(previous)
        } else {
            None
        }
    }

    pub fn event_after(&self, instant: DateTime) -> Option<Event> {
        let mut start = self.align_to_series(instant)?;

        if start <= instant {
            start = self.repeat.next_event(start)?;
        }

        self.event_at(start)
    }

    pub fn event_before(&self, instant: DateTime) -> Option<Event> {
        let mut start = self.align_to_series(instant)?;

        if start >= instant {
            start = self.repeat.previous_event(start)?;
        }

        self.event_at(start)
    }

    fn align_to_series(&self, instant: DateTime) -> Option<DateTime> {
        self.repeat.align_to_series(instant, self.start)
    }
}

impl<'a, R> IntoIterator for &'a Series<R>
where
    R: Repeat,
{
    type Item = Event;
    type IntoIter = Iter<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a, R> {
    series: &'a Series<R>,
    next_start: Option<DateTime>,
}

impl<'a, R> Iter<'a, R> {
    fn new(series: &'a Series<R>) -> Iter<'a, R> {
        Iter {
            series,
            next_start: Some(series.start),
        }
    }
}

impl<R> Iterator for Iter<'_, R>
where
    R: Repeat,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let series = &self.series;
        let start = self.next_start?;

        if start > series.end {
            return None;
        }

        self.next_start = series.repeat.next_event(start);

        // Handle the case where the series start does not fall into the desired frequency and
        // skip over to the next event right away.
        if start == series.start && !series.has_event_at(start) {
            return self.next();
        }

        series.event_at_unchecked(start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repeat::daily;
    use jiff::{
        ToSpan,
        civil::{datetime, time},
    };

    #[test]
    fn daily_series() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let series = Series::new(daily(2), start, None, None);
        let events: Vec<_> = series.iter().take(5).collect();
        let expected = vec![
            Event::at(datetime(2025, 1, 1, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 3, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 5, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 7, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 9, 1, 1, 1, 0)),
        ];
        assert_eq!(events, expected);
    }

    #[test]
    fn daily_series_at() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let series = Series::new(
            daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3)),
            start,
            None,
            None,
        );
        let events: Vec<_> = series.iter().take(5).collect();
        let expected = vec![
            Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)),
            Event::at(datetime(2025, 1, 1, 3, 3, 3, 3)),
            Event::at(datetime(2025, 1, 3, 2, 2, 2, 2)),
            Event::at(datetime(2025, 1, 3, 3, 3, 3, 3)),
            Event::at(datetime(2025, 1, 5, 2, 2, 2, 2)),
        ];
        assert_eq!(events, expected);
    }

    #[test]
    fn daily_series_with_end() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 5, 1, 1, 1, 0);
        let series = Series::new(daily(2), start, Some(end), None);
        let events: Vec<_> = series.iter().collect();
        let expected = vec![
            Event::at(datetime(2025, 1, 1, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 3, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 5, 1, 1, 1, 0)),
        ];
        assert_eq!(events, expected);
    }

    #[test]
    fn daily_series_with_end_and_duration() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 5, 1, 1, 1, 0);
        let series = Series::new(daily(2), start, Some(end), Some(1.hour()));
        let events: Vec<_> = series.iter().collect();
        let expected = vec![
            Event::at(datetime(2025, 1, 1, 1, 1, 1, 0)).ends_at(datetime(2025, 1, 1, 2, 1, 1, 0)),
            Event::at(datetime(2025, 1, 3, 1, 1, 1, 0)).ends_at(datetime(2025, 1, 3, 2, 1, 1, 0)),
            Event::at(datetime(2025, 1, 5, 1, 1, 1, 0)).ends_at(datetime(2025, 1, 5, 2, 1, 1, 0)),
        ];
        assert_eq!(events, expected);
    }

    #[test]
    fn series_has_event_at() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let series = Series::new(
            daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3)),
            start,
            None,
            None,
        );
        assert!(!series.has_event_at(datetime(2025, 1, 1, 1, 1, 1, 0)));
        assert!(series.has_event_at(datetime(2025, 1, 1, 2, 2, 2, 2)));
        assert!(series.has_event_at(datetime(2025, 1, 1, 3, 3, 3, 3)));
        assert!(!series.has_event_at(datetime(2025, 1, 1, 2, 2, 2, 3)));
        assert!(!series.has_event_at(datetime(2025, 1, 1, 3, 3, 3, 2)));
    }

    #[test]
    fn series_relative_events() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 3, 1, 1, 1, 0);
        let series = Series::new(
            daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3)),
            start,
            Some(end),
            None,
        );
        assert_eq!(series.event_at(datetime(2025, 1, 1, 1, 1, 1, 0)), None);
        assert_eq!(
            series.event_at(datetime(2025, 1, 1, 2, 2, 2, 2)),
            Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)))
        );

        assert_eq!(
            series.event_after(datetime(2025, 1, 1, 2, 2, 2, 2)),
            Some(Event::at(datetime(2025, 1, 1, 3, 3, 3, 3)))
        );

        assert_eq!(series.event_before(datetime(2025, 1, 1, 2, 2, 2, 2)), None);

        assert_eq!(
            series.event_before(datetime(2025, 1, 1, 3, 3, 3, 3)),
            Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2))),
        );
    }

    #[test]
    fn series_event_containing() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 3, 1, 1, 1, 0);
        let series = Series::new(daily(2).at(time(2, 2, 2, 2)), start, Some(end), None);
        assert_eq!(
            series.event_containing(datetime(2025, 1, 1, 1, 1, 1, 0)),
            None
        );
        assert_eq!(
            series.event_containing(datetime(2025, 1, 1, 2, 2, 2, 2)),
            Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)))
        );
        assert_eq!(
            series.event_containing(datetime(2025, 1, 1, 2, 2, 2, 3)),
            None
        );

        let series = Series::new(
            daily(2).at(time(2, 2, 2, 2)),
            start,
            Some(end),
            Some(1.hour()),
        );

        assert_eq!(
            series.event_containing(datetime(2025, 1, 1, 2, 2, 2, 3)),
            Some(Event::new(
                datetime(2025, 1, 1, 2, 2, 2, 2),
                datetime(2025, 1, 1, 3, 2, 2, 2)
            ))
        );

        assert_eq!(
            series.event_containing(datetime(2025, 1, 1, 3, 2, 2, 1)),
            Some(Event::new(
                datetime(2025, 1, 1, 2, 2, 2, 2),
                datetime(2025, 1, 1, 3, 2, 2, 2)
            ))
        );

        assert_eq!(
            series.event_containing(datetime(2025, 1, 1, 3, 2, 2, 2)),
            None
        );
    }

    #[test]
    fn series_first() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 3, 1, 1, 1, 0);
        let series = Series::new(daily(2).at(time(2, 2, 2, 2)), start, Some(end), None);
        assert_eq!(
            series.first(),
            Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)))
        );
    }

    #[test]
    fn series_last() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 10, 1, 1, 1, 0);
        let series = Series::new(daily(2).at(time(2, 2, 2, 2)), start, Some(end), None);
        assert_eq!(
            series.last(),
            Some(Event::at(datetime(2025, 1, 8, 2, 2, 2, 2)))
        );

        let series = Series::new(daily(2).at(time(2, 2, 2, 2)), start, None, None);
        assert_eq!(
            series.last(),
            Some(Event::at(datetime(9999, 12, 31, 2, 2, 2, 2)))
        );
    }
}
