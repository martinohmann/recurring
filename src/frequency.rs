use crate::repeat::{Daily, Hourly, Minutely, Repeat, Secondly};
use jiff::ToSpan;
use jiff::civil::{DateTime, Weekday};

#[derive(Debug, Clone)]
pub enum Frequency {
    Secondly(Secondly),
    Minutely(Minutely),
    Hourly(Hourly),
    Daily(Daily),
    DayOfMonth(i8),
    DayOfYear(i16),
    LastOfMonth,
    LastOfYear,
    Weekday(Weekday),
}

impl Repeat for Frequency {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        match self {
            Frequency::Secondly(secondly) => secondly.next_event(instant),
            Frequency::Minutely(minutely) => minutely.next_event(instant),
            Frequency::Hourly(hourly) => hourly.next_event(instant),
            Frequency::Daily(daily) => daily.next_event(instant),
            Frequency::DayOfYear(day_of_year) => {
                let span = if instant.day_of_year() >= *day_of_year {
                    (instant.days_in_year() - instant.day_of_year() + day_of_year).days()
                } else {
                    (day_of_year - instant.day_of_year()).days()
                };

                instant.checked_add(span).ok()
            }
            Frequency::DayOfMonth(day_of_month) => {
                let span = if instant.day() >= *day_of_month {
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
            Frequency::Weekday(weekday) => {
                if instant.weekday() == *weekday {
                    instant.checked_add(1.week()).ok()
                } else {
                    let offset_days = instant.weekday().until(*weekday);
                    instant.checked_add(offset_days.days()).ok()
                }
            }
        }
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        match self {
            Frequency::Secondly(secondly) => secondly.previous_event(instant),
            Frequency::Minutely(minutely) => minutely.previous_event(instant),
            Frequency::Hourly(hourly) => hourly.previous_event(instant),
            Frequency::Daily(daily) => daily.previous_event(instant),
            _ => unimplemented!(),
        }
    }

    fn contains_event(&self, instant: DateTime) -> bool {
        match self {
            Frequency::Secondly(secondly) => secondly.contains_event(instant),
            Frequency::Minutely(minutely) => minutely.contains_event(instant),
            Frequency::Hourly(hourly) => hourly.contains_event(instant),
            Frequency::Daily(daily) => daily.contains_event(instant),
            Frequency::DayOfYear(day_of_year) => instant.day_of_year() == *day_of_year,
            Frequency::DayOfMonth(day_of_month) => instant.day() == *day_of_month,
            Frequency::LastOfMonth => instant.day() == instant.last_of_month().day(),
            Frequency::LastOfYear => instant.day() == instant.last_of_year().day(),
            Frequency::Weekday(weekday) => instant.weekday() == *weekday,
        }
    }
}
