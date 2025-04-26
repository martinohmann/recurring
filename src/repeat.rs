use crate::Repeat;
use jiff::{
    ToSpan,
    civil::{DateTime, Time},
};

#[derive(Debug, Clone)]
pub struct Secondly {
    interval: i32,
}

impl Secondly {
    pub fn new(interval: i32) -> Secondly {
        Secondly { interval }
    }
}

impl Repeat for Secondly {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_add(self.interval.seconds()).ok()
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_sub(self.interval.seconds()).ok()
    }

    fn aligns_with_series(&self, instant: DateTime, series_start: DateTime) -> bool {
        instant.subsec_nanosecond() == series_start.subsec_nanosecond()
    }

    fn align_to_series(&self, instant: DateTime, series_start: DateTime) -> Option<DateTime> {
        instant
            .with()
            .subsec_nanosecond(series_start.subsec_nanosecond())
            .build()
            .ok()
    }
}

#[derive(Debug, Clone)]
pub struct Minutely {
    interval: i32,
}

impl Minutely {
    pub fn new(interval: i32) -> Minutely {
        Minutely { interval }
    }
}

impl Repeat for Minutely {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_add(self.interval.minutes()).ok()
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_sub(self.interval.minutes()).ok()
    }

    fn aligns_with_series(&self, instant: DateTime, series_start: DateTime) -> bool {
        instant.second() == series_start.second()
            && instant.subsec_nanosecond() == series_start.subsec_nanosecond()
    }

    fn align_to_series(&self, instant: DateTime, series_start: DateTime) -> Option<DateTime> {
        instant
            .with()
            .second(series_start.second())
            .subsec_nanosecond(series_start.subsec_nanosecond())
            .build()
            .ok()
    }
}

#[derive(Debug, Clone)]
pub struct Hourly {
    interval: i32,
}

impl Hourly {
    pub fn new(interval: i32) -> Hourly {
        Hourly { interval }
    }
}

impl Repeat for Hourly {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_add(self.interval.hours()).ok()
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_sub(self.interval.hours()).ok()
    }

    fn aligns_with_series(&self, instant: DateTime, series_start: DateTime) -> bool {
        instant.minute() == series_start.minute()
            && instant.second() == series_start.second()
            && instant.subsec_nanosecond() == series_start.subsec_nanosecond()
    }

    fn align_to_series(&self, instant: DateTime, series_start: DateTime) -> Option<DateTime> {
        instant
            .with()
            .minute(series_start.minute())
            .second(series_start.second())
            .subsec_nanosecond(series_start.subsec_nanosecond())
            .build()
            .ok()
    }
}

#[derive(Debug, Clone)]
pub struct Daily {
    interval: i32,
    at: Vec<Time>,
}

impl Daily {
    pub fn new(interval: i32) -> Daily {
        Daily {
            interval,
            at: Vec::new(),
        }
    }

    #[must_use]
    pub fn at(mut self, time: Time) -> Daily {
        self.at.push(time);
        self.at.sort();
        self
    }
}

impl Repeat for Daily {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        if self.at.is_empty() {
            instant.checked_add(self.interval.days()).ok()
        } else {
            for time in &self.at {
                let date = instant.with().time(*time).build().ok()?;

                if date > instant {
                    return Some(date);
                }
            }

            let next_date = instant.checked_add(self.interval.days()).ok()?;

            next_date.with().time(self.at[0]).build().ok()
        }
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        if self.at.is_empty() {
            instant.checked_sub(self.interval.days()).ok()
        } else {
            for time in self.at.iter().rev() {
                let date = instant.with().time(*time).build().ok()?;

                if date < instant {
                    return Some(date);
                }
            }

            let next_date = instant.checked_sub(self.interval.days()).ok()?;

            next_date.with().time(*self.at.last().unwrap()).build().ok()
        }
    }

    fn aligns_with_series(&self, instant: DateTime, series_start: DateTime) -> bool {
        if self.at.is_empty() {
            return instant.time() == series_start.time();
        }

        instant
            .checked_sub(1.minute())
            .ok()
            .and_then(|start| self.next_event(start))
            == Some(instant)
    }

    fn align_to_series(&self, instant: DateTime, series_start: DateTime) -> Option<DateTime> {
        if self.at.is_empty() {
            return instant.with().time(series_start.time()).build().ok();
        }

        instant.with().time(*self.at.first().unwrap()).build().ok()
    }
}

pub fn secondly(interval: i32) -> Secondly {
    Secondly::new(interval)
}

pub fn minutely(interval: i32) -> Minutely {
    Minutely::new(interval)
}

pub fn hourly(interval: i32) -> Hourly {
    Hourly::new(interval)
}

pub fn daily(interval: i32) -> Daily {
    Daily::new(interval)
}
