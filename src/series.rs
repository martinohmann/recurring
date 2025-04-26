use crate::{Error, Event, Repeat};
use jiff::{Span, Zoned, civil::DateTime};

#[derive(Debug, Clone)]
pub struct Series<R> {
    repeat: R,
    start: DateTime,
    end: DateTime,
    event_duration: Span,
}

impl Series<()> {
    pub fn builder() -> SeriesBuilder {
        SeriesBuilder::new()
    }
}

impl<R> Series<R>
where
    R: Repeat,
{
    /// # Errors
    ///
    /// Returns an `Error` if `start` is `DateTime::MAX`.
    pub fn new(start: DateTime, repeat: R) -> Result<Series<R>, Error> {
        Series::builder().start(start).build(repeat)
    }

    pub fn iter(&self) -> Iter<'_, R> {
        Iter::new(self)
    }

    pub fn first_event(&self) -> Option<Event> {
        if let Some(event) = self.event_at(self.start) {
            return Some(event);
        }

        let start = self.repeat.next_event(self.start)?;

        self.event_at_unchecked(start)
    }

    pub fn last_event(&self) -> Option<Event> {
        let start = self.repeat.previous_event(self.end)?;

        self.event_at_unchecked(start)
    }

    pub fn has_event_at(&self, instant: DateTime) -> bool {
        instant >= self.start
            && instant < self.end
            && self.repeat.is_event_start(instant, self.start)
    }

    pub fn event_at(&self, instant: DateTime) -> Option<Event> {
        if !self.has_event_at(instant) {
            return None;
        }

        self.event_at_unchecked(instant)
    }

    fn event_at_unchecked(&self, start: DateTime) -> Option<Event> {
        if self.event_duration.is_positive() {
            let end = start.checked_add(self.event_duration).ok()?;
            return Event::new(start, end).ok();
        }

        Some(Event::at(start))
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
        let mut start = self.align_to_event_start(instant)?;

        if start <= instant {
            start = self.repeat.next_event(start)?;
        }

        self.event_at(start)
    }

    pub fn event_before(&self, instant: DateTime) -> Option<Event> {
        let mut start = self.align_to_event_start(instant)?;

        if start >= instant {
            start = self.repeat.previous_event(start)?;
        }

        self.event_at(start)
    }

    pub fn closest_event(&self, instant: DateTime) -> Option<Event> {
        if let Some(event) = self.event_at(instant) {
            return Some(event);
        }

        match (self.event_before(instant), self.event_after(instant)) {
            (Some(before), Some(after)) => {
                if before.start().duration_until(instant) < after.start().duration_since(instant) {
                    Some(before)
                } else {
                    Some(after)
                }
            }
            (Some(before), None) => Some(before),
            (None, Some(after)) => Some(after),
            (None, None) => None,
        }
    }

    fn align_to_event_start(&self, instant: DateTime) -> Option<DateTime> {
        let aligned = self.repeat.align_to_event_start(instant, self.start)?;

        if aligned < self.start {
            return self.repeat.align_to_event_start(self.start, self.start);
        }

        if aligned > self.end {
            return self.repeat.align_to_event_start(self.end, self.start);
        }

        Some(aligned)
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

#[derive(Debug, Clone, Default)]
pub struct SeriesBuilder {
    start: Option<DateTime>,
    end: Option<DateTime>,
    event_duration: Option<Span>,
}

impl SeriesBuilder {
    /// Creates a new `SeriesBuilder` with default options.
    pub fn new() -> SeriesBuilder {
        SeriesBuilder::default()
    }

    #[must_use]
    pub fn start(mut self, start: DateTime) -> SeriesBuilder {
        self.start = Some(start);
        self
    }

    #[must_use]
    pub fn end(mut self, end: DateTime) -> SeriesBuilder {
        self.end = Some(end);
        self
    }

    #[must_use]
    pub fn event_duration(mut self, event_duration: Span) -> SeriesBuilder {
        self.event_duration = Some(event_duration);
        self
    }

    /// # Errors
    ///
    /// Returns an `Error` if the configured `end` is less than or equal to `start`, or if the
    /// configured `event_duration` is negative.
    pub fn build<R>(self, repeat: R) -> Result<Series<R>, Error>
    where
        R: Repeat,
    {
        let start = self.start.unwrap_or_else(|| Zoned::now().datetime());
        let end = self.end.unwrap_or(DateTime::MAX);
        let event_duration = self.event_duration.unwrap_or_default();

        if end <= start {
            return Err(Error::InvalidSeriesEnd);
        }

        if event_duration.is_negative() {
            return Err(Error::InvalidEventDuration);
        }

        Ok(Series {
            repeat,
            start,
            end,
            event_duration,
        })
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
    use crate::repeat::{daily, hourly};
    use alloc::vec;
    use alloc::vec::Vec;
    use jiff::{
        ToSpan,
        civil::{datetime, time},
    };

    #[test]
    fn daily_series() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let series = Series::new(start, daily(2)).unwrap();
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
        let series =
            Series::new(start, daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3))).unwrap();
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
        let series = Series::builder()
            .start(start)
            .end(end)
            .build(daily(2))
            .unwrap();
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
        let series = Series::builder()
            .start(start)
            .end(end)
            .event_duration(1.hour())
            .build(daily(2))
            .unwrap();

        let events: Vec<_> = series.iter().collect();
        let expected = vec![
            Event::new(
                datetime(2025, 1, 1, 1, 1, 1, 0),
                datetime(2025, 1, 1, 2, 1, 1, 0),
            )
            .unwrap(),
            Event::new(
                datetime(2025, 1, 3, 1, 1, 1, 0),
                datetime(2025, 1, 3, 2, 1, 1, 0),
            )
            .unwrap(),
            Event::new(
                datetime(2025, 1, 5, 1, 1, 1, 0),
                datetime(2025, 1, 5, 2, 1, 1, 0),
            )
            .unwrap(),
        ];
        assert_eq!(events, expected);
    }

    #[test]
    fn series_has_event_at() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let series =
            Series::new(start, daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3))).unwrap();
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
        let series = Series::builder()
            .start(start)
            .end(end)
            .build(daily(2).at(time(2, 2, 2, 2)).at(time(3, 3, 3, 3)))
            .unwrap();
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
        let series = Series::builder()
            .start(start)
            .end(end)
            .build(daily(2).at(time(2, 2, 2, 2)))
            .unwrap();
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

        let series = Series::builder()
            .start(start)
            .end(end)
            .event_duration(1.hour())
            .build(daily(2).at(time(2, 2, 2, 2)))
            .unwrap();

        assert_eq!(
            series.event_containing(datetime(2025, 1, 1, 2, 2, 2, 3)),
            Some(
                Event::new(
                    datetime(2025, 1, 1, 2, 2, 2, 2),
                    datetime(2025, 1, 1, 3, 2, 2, 2)
                )
                .unwrap()
            )
        );

        assert_eq!(
            series.event_containing(datetime(2025, 1, 1, 3, 2, 2, 1)),
            Some(
                Event::new(
                    datetime(2025, 1, 1, 2, 2, 2, 2),
                    datetime(2025, 1, 1, 3, 2, 2, 2)
                )
                .unwrap()
            )
        );

        assert_eq!(
            series.event_containing(datetime(2025, 1, 1, 3, 2, 2, 2)),
            None
        );
    }

    #[test]
    fn series_first_event() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 3, 1, 1, 1, 0);
        let series = Series::builder()
            .start(start)
            .end(end)
            .build(daily(2).at(time(2, 2, 2, 2)))
            .unwrap();
        assert_eq!(
            series.first_event(),
            Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)))
        );
    }

    #[test]
    fn series_last_event() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 10, 1, 1, 1, 0);
        let series = Series::builder()
            .start(start)
            .end(end)
            .build(daily(2).at(time(2, 2, 2, 2)))
            .unwrap();
        assert_eq!(
            series.last_event(),
            Some(Event::at(datetime(2025, 1, 8, 2, 2, 2, 2)))
        );

        let series = Series::new(start, daily(2).at(time(2, 2, 2, 2))).unwrap();
        assert_eq!(
            series.last_event(),
            Some(Event::at(datetime(9999, 12, 31, 2, 2, 2, 2)))
        );
    }

    #[test]
    fn series_closest_event() {
        let start = datetime(2025, 1, 1, 0, 0, 0, 0);
        let series = Series::new(start, hourly(1)).unwrap();

        assert_eq!(
            series.closest_event(datetime(2024, 12, 31, 0, 0, 0, 0)),
            Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
        );
        assert_eq!(
            series.closest_event(datetime(2025, 1, 1, 0, 0, 0, 0)),
            Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
        );
        assert_eq!(
            series.closest_event(datetime(2025, 1, 1, 0, 29, 0, 999)),
            Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
        );
        assert_eq!(
            series.closest_event(datetime(2025, 1, 1, 0, 30, 0, 0)),
            Some(Event::at(datetime(2025, 1, 1, 1, 0, 0, 0)))
        );
        assert_eq!(
            series.closest_event(DateTime::MIN),
            Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
        );
        assert_eq!(
            series.closest_event(DateTime::MAX),
            Some(Event::at(datetime(9999, 12, 31, 23, 0, 0, 0)))
        );
    }
}
