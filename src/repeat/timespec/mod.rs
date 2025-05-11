mod timeunit;

use self::timeunit::{Days, Hours, Minutes, Months, Seconds, Weekdays, Years};
use crate::Repeat;
use core::ops::Range;
use jiff::ToSpan;
use jiff::civil::{DateTime, Weekday};

/// A time spec implementation similar to the cron pattern.
///
/// You can imagine this as some more granular form of the the cron pattern `* * * * *` (at
/// every minute). `TimeSpec` also includes a 6th component to facilitate second precision.
///
/// The default of a time spec is to tick at every second, which would be equivalent to the
/// second-enhanced cron pattern of `* * * * * *`.
///
/// After constructed, this type has various builder methods like [`.second()`][TimeSpec::second]
/// and [`.hours()`][TimeSpec::hours] to configure the details of the timespec.
///
/// # Example: once per day at a certain time
///
/// ```
/// # use recurring::repeat::TimeSpec;
/// // Every day at 12:30.
/// let spec = TimeSpec::new().hour(12).minute(30);
/// ```
///
/// # Example: at multiple occasion thoughout the day
///
/// ```
/// # use recurring::repeat::TimeSpec;
/// // Every day at 12:30, 13:30 and 14:30.
/// let spec = TimeSpec::new().hours(12..15).minute(30);
/// ```
///
/// # Example: multiple non-consecutive time units
///
/// ```
/// # use recurring::repeat::TimeSpec;
/// // Every day at 10:30 and 20:30.
/// let spec = TimeSpec::new().hour(10).hour(20).minute(30);
/// // Or
/// let spec = TimeSpec::new().hours([10, 20]).minute(30);
/// ```
///
/// # Example: pitfalls
///
/// Because `TimeSpec` behaves pretty much like a cron pattern, the following is expected behaviour
/// but might not be what you want.
///
/// ```
/// # use recurring::repeat::TimeSpec;
/// // This ticks at 10:15 and 20:30. But it also ticks at 10:30 and 20:15.
/// let spec = TimeSpec::new()
///     .hour(10).minute(15)
///     .hour(20).minute(30);
///
/// // And so does this.
/// let spec = TimeSpec::new().hours([10, 20]).minutes([15, 30]);
///
/// // Let's break it down:
/// let spec = TimeSpec::new()
///     .hour(10)    // Tick at hour 10.
///     .minute(15)  // Tick at minute 15.
///     .hour(20)    // Tick at hour 20 (we already tick at 10 too).
///     .minute(30); // Tick at minute 30 (we already tick at 15 too).
///
/// // This ticks every day at:
/// //
/// // - On hour 10 at minute 15 and 30
/// // - On hour 20 at minute 15 and 30
/// ```
///
/// # Example: time unit ranges
///
/// ```
/// // This ticks at every hour from 10 to 19 at every minute from 15 to 29 in european summer
/// // months:
/// # use recurring::repeat::TimeSpec;
/// let spec = TimeSpec::new().months(6..=9).hours(10..=20).minutes(15..30);
/// ```
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

// Builder methods
impl TimeSpec {
    /// Create a new `TimeSpec` that ticks every second.
    pub fn new() -> TimeSpec {
        TimeSpec::default()
    }

    /// Limit the time spec to a specific year.
    ///
    /// This method can be called multiple times to limit the time spec to multiple different
    /// years. Alternatively, you can use [`.years()`][TimeSpec::years] to feed years from an
    /// iterator.
    ///
    /// # Panics
    ///
    /// This panics when the year is too small or too big. The minimum value is `-9999`. The
    /// maximum value is `9999`.
    #[must_use]
    pub fn year(mut self, year: i16) -> TimeSpec {
        self.years.insert(year);
        self
    }

    /// Limit the years in the time spec to specific values from an iterator.
    ///
    /// # Panics
    ///
    /// This panics when any of the year values produced by the iterator is year is too small
    /// or too big. The minimum value is `-9999`. The maximum value is `9999`.
    #[must_use]
    pub fn years<I: IntoIterator<Item = i16>>(self, years: I) -> TimeSpec {
        years.into_iter().fold(self, TimeSpec::year)
    }

    /// Limit the time spec to a specific month.
    ///
    /// This method can be called multiple times to limit the time spec to multiple different
    /// months. Alternatively, you can use [`.months()`][TimeSpec::months] to feed months from an
    /// iterator.
    ///
    /// # Panics
    ///
    /// This panics when the month is too small or too big. The minimum value is `1`. The maximum
    /// value is `12`.
    #[must_use]
    pub fn month(mut self, month: i8) -> TimeSpec {
        self.months.insert(month);
        self
    }

    /// Limit the months in the time spec to specific values from an iterator.
    ///
    /// # Panics
    ///
    /// This panics when any of the month values produced by the iterator is month is too small
    /// or too big. The minimum value is `1`. The maximum value is `12`.
    #[must_use]
    pub fn months<I: IntoIterator<Item = i8>>(self, months: I) -> TimeSpec {
        months.into_iter().fold(self, TimeSpec::month)
    }

    /// Limit the time spec to a specific weekday.
    ///
    /// This method can be called multiple times to limit the time spec to multiple different
    /// weekdays. Alternatively, you can use [`.weekdays()`][TimeSpec::weekdays] to feed weekdays
    /// from an iterator.
    #[must_use]
    pub fn weekday(mut self, weekday: Weekday) -> TimeSpec {
        self.weekdays.insert(weekday.to_monday_one_offset());
        self
    }

    /// Limit the weekdays in the time spec to specific values from an iterator.
    #[must_use]
    pub fn weekdays<I: IntoIterator<Item = Weekday>>(self, weekdays: I) -> TimeSpec {
        weekdays.into_iter().fold(self, TimeSpec::weekday)
    }

    /// Limit the time spec to a specific day.
    ///
    /// This method can be called multiple times to limit the time spec to multiple different days.
    /// Alternatively, you can use [`.days()`][TimeSpec::days] to feed days from an iterator.
    ///
    /// # Panics
    ///
    /// This panics when the day is too small or too big. The minimum value is `1`. The maximum
    /// value is `31`.
    #[must_use]
    pub fn day(mut self, day: i8) -> TimeSpec {
        self.days.insert(day);
        self
    }

    /// Limit the days in the time spec to specific values from an iterator.
    ///
    /// # Panics
    ///
    /// This panics when any of the day values produced by the iterator is day is too small
    /// or too big. The minimum value is `1`. The maximum value is `31`.
    #[must_use]
    pub fn days<I: IntoIterator<Item = i8>>(self, days: I) -> TimeSpec {
        days.into_iter().fold(self, TimeSpec::day)
    }

    /// Limit the time spec to a specific hour.
    ///
    /// This method can be called multiple times to limit the time spec to multiple different hours.
    /// Alternatively, you can use [`.hours()`][TimeSpec::hours] to feed hours from an iterator.
    ///
    /// # Panics
    ///
    /// This panics when the hour is too small or too big. The minimum value is `0`. The maximum
    /// value is `23`.
    #[must_use]
    pub fn hour(mut self, hour: i8) -> TimeSpec {
        self.hours.insert(hour);
        self
    }

    /// Limit the hours in the time spec to specific values from an iterator.
    ///
    /// # Panics
    ///
    /// This panics when any of the hour values produced by the iterator is hour is too small
    /// or too big. The minimum value is `0`. The maximum value is `23`.
    #[must_use]
    pub fn hours<I: IntoIterator<Item = i8>>(self, hours: I) -> TimeSpec {
        hours.into_iter().fold(self, TimeSpec::hour)
    }

    /// Limit the time spec to a specific minute.
    ///
    /// This method can be called multiple times to limit the time spec to multiple different minutes.
    /// Alternatively, you can use [`.minutes()`][TimeSpec::minutes] to feed minutes from an iterator.
    ///
    /// # Panics
    ///
    /// This panics when the minute is too small or too big. The minimum value is `0`. The maximum
    /// value is `59`.
    #[must_use]
    pub fn minute(mut self, minute: i8) -> TimeSpec {
        self.minutes.insert(minute);
        self
    }

    /// Limit the minutes in the time spec to specific values from an iterator.
    ///
    /// # Panics
    ///
    /// This panics when any of the minute values produced by the iterator is minute is too small
    /// or too big. The minimum value is `0`. The maximum value is `59`.
    #[must_use]
    pub fn minutes<I: IntoIterator<Item = i8>>(self, minutes: I) -> TimeSpec {
        minutes.into_iter().fold(self, TimeSpec::minute)
    }

    /// Limit the time spec to a specific second.
    ///
    /// This method can be called multiple times to limit the time spec to multiple different seconds.
    /// Alternatively, you can use [`.seconds()`][TimeSpec::seconds] to feed seconds from an iterator.
    ///
    /// # Panics
    ///
    /// This panics when the second is too small or too big. The minimum value is `0`. The maximum
    /// value is `59`.
    #[must_use]
    pub fn second(mut self, second: i8) -> TimeSpec {
        self.seconds.insert(second);
        self
    }

    /// Limit the seconds in the time spec to specific values from an iterator.
    ///
    /// # Panics
    ///
    /// This panics when any of the second values produced by the iterator is second is too small
    /// or too big. The minimum value is `0`. The maximum value is `59`.
    #[must_use]
    pub fn seconds<I: IntoIterator<Item = i8>>(self, seconds: I) -> TimeSpec {
        seconds.into_iter().fold(self, TimeSpec::second)
    }
}

// Implementation of finding timespec events.
impl TimeSpec {
    fn next_or_current_event(
        &self,
        instant: DateTime,
        range: &Range<DateTime>,
    ) -> Option<DateTime> {
        let mut clamp = DateTimeClamp::new(instant);

        for year in self.years.range(clamp.year..=Years::MAX) {
            if year > instant.year() {
                clamp.months_to_min();
            }

            let month_start = clamp.month;
            if !self.months.contains(month_start) {
                clamp.months_to_min();
            }

            for month in self.months.range(month_start..=Months::MAX) {
                let day_start = clamp.day;
                if !self.days.contains(day_start) {
                    clamp.days_to_min();
                }

                let day_end = days_in_month(month, year);
                let day_start = day_start.min(day_end);

                'day_loop: for day in self.days.range(day_start..=day_end) {
                    let hour_start = clamp.hour;
                    if !self.hours.contains(clamp.hour) {
                        clamp.hours_to_min();
                    }

                    for hour in self.hours.range(hour_start..=Hours::MAX) {
                        let minute_start = clamp.minute;
                        if !self.minutes.contains(minute_start) {
                            clamp.minutes_to_min();
                        }

                        for minute in self.minutes.range(minute_start..=Minutes::MAX) {
                            let second_start = clamp.second;
                            if !self.seconds.contains(second_start) {
                                clamp.seconds_to_min();
                            }

                            for second in self.seconds.range(second_start..=Seconds::MAX) {
                                let Ok(date) =
                                    DateTime::new(year, month, day, hour, minute, second, 0)
                                else {
                                    continue;
                                };

                                if self
                                    .weekdays
                                    .contains(date.weekday().to_monday_one_offset())
                                {
                                    if range.contains(&date) {
                                        return Some(date);
                                    }
                                    return None;
                                }

                                continue 'day_loop;
                            }

                            clamp.minutes_to_min();
                        }

                        clamp.hours_to_min();
                    }

                    clamp.days_to_min();
                }

                clamp.months_to_min();
            }
        }

        None
    }

    fn previous_or_current_event(
        &self,
        instant: DateTime,
        range: &Range<DateTime>,
    ) -> Option<DateTime> {
        let mut clamp = DateTimeClamp::new(instant);

        for year in self.years.range(Years::MIN..=clamp.year).rev() {
            let month_end = clamp.month;
            if !self.months.contains(month_end) {
                clamp.months_to_max();
            }

            for month in self.months.range(Months::MIN..=month_end).rev() {
                let day_end = clamp.day;
                if !self.days.contains(day_end) {
                    clamp.days_to_max();
                }

                let day_end = days_in_month(month, year).min(day_end);

                'day_loop: for day in self.days.range(Days::MIN..=day_end).rev() {
                    let hour_end = clamp.hour;
                    if !self.hours.contains(clamp.hour) {
                        clamp.hours_to_max();
                    }

                    for hour in self.hours.range(Hours::MIN..=hour_end).rev() {
                        let minute_end = clamp.minute;
                        if !self.minutes.contains(minute_end) {
                            clamp.minutes_to_max();
                        }

                        for minute in self.minutes.range(Minutes::MIN..=minute_end).rev() {
                            let second_end = clamp.second;
                            if !self.seconds.contains(second_end) {
                                clamp.seconds_to_max();
                            }

                            for second in self.seconds.range(Seconds::MIN..=second_end).rev() {
                                let Ok(date) =
                                    DateTime::new(year, month, day, hour, minute, second, 0)
                                else {
                                    continue;
                                };

                                if self
                                    .weekdays
                                    .contains(date.weekday().to_monday_one_offset())
                                {
                                    if range.contains(&date) {
                                        return Some(date);
                                    }
                                    return None;
                                }

                                continue 'day_loop;
                            }

                            clamp.minutes_to_max();
                        }

                        clamp.hours_to_max();
                    }

                    clamp.days_to_max();
                }

                clamp.months_to_max();
            }
        }

        None
    }
}

impl Repeat for TimeSpec {
    fn next_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let instant = instant.checked_add(1.second()).ok()?;
        self.next_or_current_event(instant, range)
    }

    fn previous_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let instant = if instant.subsec_nanosecond() > 0 {
            instant
        } else {
            instant.checked_sub(1.second()).ok()?
        };
        self.previous_or_current_event(instant, range)
    }

    fn closest_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let Some(next) = self.next_or_current_event(instant, range) else {
            return self.previous_event(instant, range);
        };

        if next == instant {
            return Some(next);
        }

        let Some(previous) = self.previous_event(instant, range) else {
            return Some(next);
        };

        if instant.duration_since(previous) >= instant.duration_until(next) {
            Some(next)
        } else {
            Some(previous)
        }
    }
}

struct DateTimeClamp {
    year: i16,
    month: i8,
    day: i8,
    hour: i8,
    minute: i8,
    second: i8,
}

impl DateTimeClamp {
    fn new(date: DateTime) -> DateTimeClamp {
        DateTimeClamp {
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
        let range = DateTime::MIN..DateTime::MAX;
        let ts = TimeSpec::new().minute(3).second(5).second(10);

        assert_eq!(
            ts.next_event(date(2025, 1, 1).at(0, 0, 0, 0), &range),
            Some(date(2025, 1, 1).at(0, 3, 5, 0))
        );

        assert_eq!(
            ts.next_event(date(2025, 1, 1).at(0, 3, 5, 0), &range),
            Some(date(2025, 1, 1).at(0, 3, 10, 0))
        );

        assert_eq!(
            ts.next_event(date(2025, 1, 1).at(0, 3, 10, 0), &range),
            Some(date(2025, 1, 1).at(1, 3, 5, 0))
        );
    }

    #[test]
    fn previous_event() {
        let range = DateTime::MIN..DateTime::MAX;
        let ts = TimeSpec::new().minute(3).seconds([5, 10]);

        assert_eq!(
            ts.previous_event(date(2025, 1, 1).at(0, 3, 5, 0), &range),
            Some(date(2024, 12, 31).at(23, 3, 10, 0))
        );

        assert_eq!(
            ts.previous_event(date(2024, 12, 31).at(23, 3, 10, 0), &range),
            Some(date(2024, 12, 31).at(23, 3, 5, 0))
        );

        assert_eq!(
            ts.previous_event(date(2024, 12, 31).at(23, 3, 5, 0), &range),
            Some(date(2024, 12, 31).at(22, 3, 10, 0))
        );
    }

    #[test]
    fn closest_event() {
        let range = DateTime::MIN..DateTime::MAX;
        let ts = TimeSpec::new().hour(1).minute(30).second(0);

        assert_eq!(
            ts.closest_event(date(2025, 1, 1).at(1, 29, 59, 0), &range),
            Some(date(2025, 1, 1).at(1, 30, 0, 0))
        );

        assert_eq!(
            ts.closest_event(date(2025, 1, 1).at(1, 30, 1, 0), &range),
            Some(date(2025, 1, 1).at(1, 30, 0, 0))
        );

        assert_eq!(
            ts.closest_event(date(2025, 1, 1).at(14, 0, 0, 0), &range),
            Some(date(2025, 1, 2).at(1, 30, 0, 0))
        );

        assert_eq!(
            ts.closest_event(date(2025, 1, 1).at(1, 30, 0, 0), &range),
            Some(date(2025, 1, 1).at(1, 30, 0, 0))
        );
    }
}
