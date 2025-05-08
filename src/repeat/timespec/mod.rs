mod timeunit;

use self::timeunit::{Days, Hours, Minutes, Months, Seconds, Weekdays, Years};
use crate::Repeat;
use core::ops::Range;
use jiff::ToSpan;
use jiff::civil::{DateTime, Weekday};

#[derive(Debug, Clone, Default)]
pub struct TimeSpec {
    years: Years,
    months: Months,
    weekdays: Weekdays,
    days: Days,
    hours: Hours,
    minutes: Minutes,
    seconds: Seconds,
}

impl TimeSpec {
    pub fn new() -> TimeSpec {
        TimeSpec::default()
    }

    #[must_use]
    pub fn year(mut self, year: i16) -> TimeSpec {
        self.years.insert(year);
        self
    }

    #[must_use]
    pub fn years<I: IntoIterator<Item = i16>>(self, years: I) -> TimeSpec {
        years.into_iter().fold(self, TimeSpec::year)
    }

    #[must_use]
    pub fn month(mut self, month: i8) -> TimeSpec {
        self.months.insert(month);
        self
    }

    #[must_use]
    pub fn months<I: IntoIterator<Item = i8>>(self, months: I) -> TimeSpec {
        months.into_iter().fold(self, TimeSpec::month)
    }

    #[must_use]
    pub fn weekday(mut self, weekday: Weekday) -> TimeSpec {
        self.weekdays.insert(weekday as i8);
        self
    }

    #[must_use]
    pub fn weekdays<I: IntoIterator<Item = Weekday>>(self, weekdays: I) -> TimeSpec {
        weekdays.into_iter().fold(self, TimeSpec::weekday)
    }

    #[must_use]
    pub fn day(mut self, day: i8) -> TimeSpec {
        self.days.insert(day);
        self
    }

    #[must_use]
    pub fn days<I: IntoIterator<Item = i8>>(self, days: I) -> TimeSpec {
        days.into_iter().fold(self, TimeSpec::day)
    }

    #[must_use]
    pub fn hour(mut self, hour: i8) -> TimeSpec {
        self.hours.insert(hour);
        self
    }

    #[must_use]
    pub fn hours<I: IntoIterator<Item = i8>>(self, hours: I) -> TimeSpec {
        hours.into_iter().fold(self, TimeSpec::hour)
    }

    #[must_use]
    pub fn minute(mut self, minute: i8) -> TimeSpec {
        self.minutes.insert(minute);
        self
    }

    #[must_use]
    pub fn minutes<I: IntoIterator<Item = i8>>(self, minutes: I) -> TimeSpec {
        minutes.into_iter().fold(self, TimeSpec::minute)
    }

    #[must_use]
    pub fn second(mut self, second: i8) -> TimeSpec {
        self.seconds.insert(second);
        self
    }

    #[must_use]
    pub fn seconds<I: IntoIterator<Item = i8>>(self, seconds: I) -> TimeSpec {
        seconds.into_iter().fold(self, TimeSpec::second)
    }
}

impl Repeat for TimeSpec {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        let mut state = State::new(instant.checked_add(1.second()).ok()?);

        for year in self.years.range(state.year..=Years::MAX) {
            if year > instant.year() {
                state.months_to_min();
            }

            let month_start = state.month;
            if !self.months.contains(month_start) {
                state.months_to_min();
            }

            for month in self.months.range(month_start..=Months::MAX) {
                let day_start = state.day;
                if !self.days.contains(day_start) {
                    state.days_to_min();
                }

                let day_end = days_in_month(month, year);
                let day_start = day_start.min(day_end);

                'day_loop: for day in self.days.range(day_start..=day_end) {
                    let hour_start = state.hour;
                    if !self.hours.contains(state.hour) {
                        state.hours_to_min();
                    }

                    for hour in self.hours.range(hour_start..=Hours::MAX) {
                        let minute_start = state.minute;
                        if !self.minutes.contains(minute_start) {
                            state.minutes_to_min();
                        }

                        for minute in self.minutes.range(minute_start..=Minutes::MAX) {
                            let second_start = state.second;
                            if !self.seconds.contains(second_start) {
                                state.seconds_to_min();
                            }

                            for second in self.seconds.range(second_start..=Seconds::MAX) {
                                let Ok(date) =
                                    DateTime::new(year, month, day, hour, minute, second, 0)
                                else {
                                    continue;
                                };

                                if self.weekdays.contains(date.weekday() as i8) {
                                    return Some(date);
                                }

                                continue 'day_loop;
                            }

                            state.minutes_to_min();
                        }

                        state.hours_to_min();
                    }

                    state.days_to_min();
                }

                state.months_to_min();
            }
        }

        None
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        let initial = if instant.subsec_nanosecond() > 0 {
            instant
        } else {
            instant.checked_sub(1.second()).ok()?
        };

        let mut state = State::new(initial);

        for year in self.years.range(Years::MIN..=state.year).rev() {
            let month_end = state.month;
            if !self.months.contains(month_end) {
                state.months_to_max();
            }

            for month in self.months.range(Months::MIN..=month_end).rev() {
                let day_end = state.day;
                if !self.days.contains(day_end) {
                    state.days_to_max();
                }

                let day_end = days_in_month(month, year).min(day_end);

                'day_loop: for day in self.days.range(Days::MIN..=day_end).rev() {
                    let hour_end = state.hour;
                    if !self.hours.contains(state.hour) {
                        state.hours_to_max();
                    }

                    for hour in self.hours.range(Hours::MIN..=hour_end).rev() {
                        let minute_end = state.minute;
                        if !self.minutes.contains(minute_end) {
                            state.minutes_to_max();
                        }

                        for minute in self.minutes.range(Minutes::MIN..=minute_end).rev() {
                            let second_end = state.second;
                            if !self.seconds.contains(second_end) {
                                state.seconds_to_max();
                            }

                            for second in self.seconds.range(Seconds::MIN..=second_end).rev() {
                                let Ok(date) =
                                    DateTime::new(year, month, day, hour, minute, second, 0)
                                else {
                                    continue;
                                };

                                if self.weekdays.contains(date.weekday() as i8) {
                                    return Some(date);
                                }

                                continue 'day_loop;
                            }

                            state.minutes_to_max();
                        }

                        state.hours_to_max();
                    }

                    state.days_to_max();
                }

                state.months_to_max();
            }
        }

        None
    }

    fn closest_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let previous = self
            .previous_event(instant)
            .filter(|previous| range.contains(previous));
        let next = self.next_event(instant).filter(|next| range.contains(next));

        match (previous, next) {
            (Some(previous), Some(next)) => {
                if instant.duration_since(previous) < instant.duration_until(next) {
                    Some(previous)
                } else {
                    Some(next)
                }
            }
            (Some(previous), None) => Some(previous),
            (None, Some(next)) => Some(next),
            (None, None) => None,
        }
    }
}

struct State {
    year: i16,
    month: i8,
    day: i8,
    hour: i8,
    minute: i8,
    second: i8,
}

impl State {
    fn new(date: DateTime) -> State {
        State {
            year: date.year(),
            month: date.month(),
            day: date.day(),
            hour: date.hour(),
            minute: date.minute(),
            second: date.second(),
        }
    }

    fn months_to_max(&mut self) {
        self.month = Months::MAX;
        self.days_to_max();
    }

    fn months_to_min(&mut self) {
        self.month = Months::MIN;
        self.days_to_min();
    }

    fn days_to_max(&mut self) {
        self.day = Days::MAX;
        self.hours_to_max();
    }

    fn days_to_min(&mut self) {
        self.day = Days::MIN;
        self.hours_to_min();
    }

    fn hours_to_max(&mut self) {
        self.hour = Hours::MAX;
        self.minutes_to_max();
    }

    fn hours_to_min(&mut self) {
        self.hour = Hours::MIN;
        self.minutes_to_min();
    }

    fn minutes_to_max(&mut self) {
        self.minute = Minutes::MAX;
        self.seconds_to_max();
    }

    fn minutes_to_min(&mut self) {
        self.minute = Minutes::MIN;
        self.seconds_to_min();
    }

    fn seconds_to_max(&mut self) {
        self.second = Seconds::MAX;
    }

    fn seconds_to_min(&mut self) {
        self.second = Seconds::MIN;
    }
}

fn is_leap_year(year: i16) -> bool {
    let by_four = year % 4 == 0;
    let by_hundred = year % 100 == 0;
    let by_four_hundred = year % 400 == 0;
    by_four && ((!by_hundred) || by_four_hundred)
}

fn days_in_month(month: i8, year: i16) -> i8 {
    match month {
        9 | 4 | 6 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    #[test]
    fn next_event() {
        let ts = TimeSpec::new().minute(3).second(5).second(10);

        assert_eq!(
            ts.next_event(date(2025, 1, 1).at(0, 0, 0, 0)),
            Some(date(2025, 1, 1).at(0, 3, 5, 0))
        );

        assert_eq!(
            ts.next_event(date(2025, 1, 1).at(0, 3, 5, 0)),
            Some(date(2025, 1, 1).at(0, 3, 10, 0))
        );

        assert_eq!(
            ts.next_event(date(2025, 1, 1).at(0, 3, 10, 0)),
            Some(date(2025, 1, 1).at(1, 3, 5, 0))
        );
    }

    #[test]
    fn previous_event() {
        let ts = TimeSpec::new().minute(3).seconds([5, 10]);

        assert_eq!(
            ts.previous_event(date(2025, 1, 1).at(0, 3, 5, 0)),
            Some(date(2024, 12, 31).at(23, 3, 10, 0))
        );

        assert_eq!(
            ts.previous_event(date(2024, 12, 31).at(23, 3, 10, 0)),
            Some(date(2024, 12, 31).at(23, 3, 5, 0))
        );

        assert_eq!(
            ts.previous_event(date(2024, 12, 31).at(23, 3, 5, 0)),
            Some(date(2024, 12, 31).at(22, 3, 10, 0))
        );
    }
}
