use jiff::civil::{DateTime, Weekday};
use jiff::{SignedDuration, Span, ToSpan};

pub mod event;
pub mod frequency;
pub mod repeat;
pub mod series;

#[derive(Debug, Clone)]
pub struct Event {
    start: DateTime,
    end: Option<DateTime>,
    repeat: Option<Frequency>,
}

impl Event {
    pub fn new(start: DateTime, end: DateTime) -> Event {
        Event::at(start).until(end)
    }

    pub fn at(instant: DateTime) -> Event {
        Event {
            start: instant,
            end: None,
            repeat: None,
        }
    }

    pub fn until(&self, end: DateTime) -> Event {
        assert!(self.start <= end, "event end > start");
        Event {
            start: self.start,
            end: Some(end),
            repeat: None,
        }
    }

    pub fn repeat(&self, frequency: Frequency) -> Event {
        Event {
            start: self.start,
            end: self.end,
            repeat: Some(frequency),
        }
    }

    pub fn series(&self) -> Series {
        Series::new(self)
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

    pub fn frequency(&self) -> Option<Frequency> {
        self.repeat
    }

    pub fn contains(&self, instant: DateTime) -> bool {
        match self.end {
            Some(end) => self.start >= instant && instant < end,
            None => self.start == instant,
        }
    }

    pub fn is_recurring(&self) -> bool {
        self.repeat.is_some()
    }

    pub fn next(&self) -> Option<Event> {
        self.shift(|frequency| frequency.next_date_time(self.start))
    }

    pub fn previous(&self) -> Option<Event> {
        self.shift(|frequency| frequency.previous_date_time(self.start))
    }

    fn shift<F>(&self, f: F) -> Option<Event>
    where
        F: FnOnce(&Frequency) -> Option<DateTime>,
    {
        self.repeat.and_then(|frequency| {
            let start = f(&frequency)?;
            let end = self
                .duration()
                .and_then(|duration| start.checked_add(duration).ok());

            Some(Event {
                start,
                end,
                repeat: self.repeat,
            })
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Frequency {
    DayOfMonth(i8),
    DayOfYear(i16),
    LastOfMonth,
    LastOfYear,
    Span(Span),
    Weekday(Weekday),
}

impl Frequency {
    fn next_date_time(&self, instant: DateTime) -> Option<DateTime> {
        match *self {
            Frequency::DayOfYear(day_of_year) => {
                let span = if instant.day_of_year() >= day_of_year {
                    (instant.days_in_year() - instant.day_of_year() + day_of_year).days()
                } else {
                    (day_of_year - instant.day_of_year()).days()
                };

                instant.checked_add(span).ok()
            }
            Frequency::DayOfMonth(day_of_month) => {
                let span = if instant.day() >= day_of_month {
                    (instant.days_in_month() - instant.day() + day_of_month).days()
                } else {
                    (day_of_month - instant.day()).days()
                };

                instant.checked_add(span).ok()
            }
            Frequency::LastOfMonth => {
                let last_of_month = instant.last_of_month();

                if instant.day() == last_of_month.day() {
                    instant
                        .checked_add(1.month())
                        .map(|instant| instant.last_of_month())
                        .ok()
                } else {
                    Some(last_of_month)
                }
            }
            Frequency::LastOfYear => {
                let last_of_year = instant.last_of_year();

                if instant.day() == last_of_year.day() {
                    instant
                        .checked_add(1.year())
                        .map(|instant| instant.last_of_year())
                        .ok()
                } else {
                    Some(last_of_year)
                }
            }
            Frequency::Span(span) => instant.checked_add(span).ok(),
            Frequency::Weekday(weekday) => {
                if instant.weekday() == weekday {
                    instant.checked_add(1.week()).ok()
                } else {
                    let offset_days = instant.weekday().until(weekday);
                    instant.checked_add(offset_days.days()).ok()
                }
            }
        }
    }

    fn previous_date_time(&self, _instant: DateTime) -> Option<DateTime> {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct Series<'a> {
    first: &'a Event,
    end: Option<DateTime>,
}

impl<'a> Series<'a> {
    fn new(first: &Event) -> Series {
        Series { first, end: None }
    }

    pub fn until(&self, end: DateTime) -> Series<'a> {
        Series {
            first: self.first,
            end: Some(end),
        }
    }

    pub fn first(&self) -> &Event {
        self.first
    }

    pub fn start(&self) -> DateTime {
        self.first.start()
    }

    pub fn end(&self) -> Option<DateTime> {
        self.end
    }

    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    pub fn last(&self) -> Option<Event> {
        self.end.and_then(|end| self.event_before(end))
    }

    pub fn event_before(&self, instant: DateTime) -> Option<Event> {
        let mut previous: Option<Event> = None;

        for event in self {
            if let Some(previous) = previous {
                if instant > previous.start() && instant <= event.start() {
                    return Some(previous);
                }
            }

            previous = Some(event);
        }

        None
    }

    pub fn event_containing(&self, instant: DateTime) -> Option<Event> {
        for event in self {
            if event.start() > instant {
                return None;
            }

            if event.contains(instant) {
                return Some(event);
            }
        }

        None
    }

    pub fn event_after(&self, instant: DateTime) -> Option<Event> {
        self.iter().find(|event| event.start() > instant)
    }
}

impl<'a> IntoIterator for &'a Series<'a> {
    type Item = Event;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a> {
    series: &'a Series<'a>,
    first: Option<Event>,
    current: Option<Event>,
}

impl<'a> Iter<'a> {
    fn new(series: &'a Series) -> Iter<'a> {
        Iter {
            series,
            first: Some(series.first().clone()),
            current: None,
        }
    }
}

impl Iterator for Iter<'_> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let event = if let Some(first) = self.first.take() {
            first
        } else {
            let current = self.current.take()?;
            current.next()?
        };

        if let Some(end) = self.series.end() {
            if event.start() >= end {
                return None;
            }
        }

        self.current = Some(event.clone());

        Some(event)
    }
}
