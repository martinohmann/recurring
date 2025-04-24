use jiff::{
    ToSpan,
    civil::{DateTime, Time},
};

pub trait Repeat {
    fn next_event(&self, instant: DateTime) -> Option<DateTime>;

    fn previous_event(&self, instant: DateTime) -> Option<DateTime>;

    fn contains_event(&self, instant: DateTime) -> bool;
}

#[derive(Debug, Clone)]
pub struct Secondly {
    pub interval: i32,
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

    fn contains_event(&self, _instant: DateTime) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct Minutely {
    pub interval: i32,
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

    fn contains_event(&self, _instant: DateTime) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct Hourly {
    pub interval: i32,
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

    fn contains_event(&self, _instant: DateTime) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct Daily {
    pub interval: i32,
    pub at: Vec<Time>,
}

impl Daily {
    pub fn new(interval: i32) -> Daily {
        Daily {
            interval,
            at: Vec::new(),
        }
    }

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

    fn contains_event(&self, instant: DateTime) -> bool {
        if self.at.is_empty() {
            return true;
        }

        instant.checked_sub(1.minute()).map_or(false, |start| {
            self.next_event(start)
                .map_or(false, |next_date| next_date == instant)
        })
    }
}
