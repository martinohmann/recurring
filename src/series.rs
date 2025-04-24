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
        Iter {
            series: self,
            next_start: Some(self.start),
        }
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

impl<R> Iterator for Iter<'_, R>
where
    R: Repeat,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(start) = self.next_start {
            if start > self.series.end {
                return None;
            }

            self.next_start = self.series.repeat.next_event(start);

            // Handle the case where the series start does not fall into the desired frequency and
            // skip over to the next event right away.
            if start == self.series.start && !self.series.repeat.contains_event(start) {
                return self.next();
            }

            let mut event = Event::at(start);

            if let Some(duration) = self.series.duration {
                let end = start.checked_add(duration).ok()?;
                event = event.ends_at(end);
            }

            return Some(event);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repeat::Daily;
    use jiff::{ToSpan, civil::datetime};

    #[test]
    fn daily_series() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let series = Series::new(Daily::new(2), start, None, None);
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
    fn daily_series_with_end() {
        let start = datetime(2025, 1, 1, 1, 1, 1, 0);
        let end = datetime(2025, 1, 5, 1, 1, 1, 0);
        let series = Series::new(Daily::new(2), start, Some(end), None);
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
        let series = Series::new(Daily::new(2), start, Some(end), Some(1.hour()));
        let events: Vec<_> = series.iter().collect();
        let expected = vec![
            Event::at(datetime(2025, 1, 1, 1, 1, 1, 0)).ends_at(datetime(2025, 1, 1, 2, 1, 1, 0)),
            Event::at(datetime(2025, 1, 3, 1, 1, 1, 0)).ends_at(datetime(2025, 1, 3, 2, 1, 1, 0)),
            Event::at(datetime(2025, 1, 5, 1, 1, 1, 0)).ends_at(datetime(2025, 1, 5, 2, 1, 1, 0)),
        ];
        assert_eq!(events, expected);
    }
}
