use super::ranged::{Days, Hours, Minutes, Months, Seconds, Weekdays, Years};
use crate::{Error, Pattern, private};
use core::ops::Range;
use jiff::ToSpan;
use jiff::civil::{DateTime, Weekday};

/// A cron-like recurrence pattern.
///
/// You can imagine this as some more granular form of the the cron pattern `* * * * *` (at
/// every minute). `Cron` also includes a 6th component to facilitate second precision.
///
/// The default value of a `Cron` produces an event at every second, which would be equivalent to
/// the second-enhanced cron pattern of `* * * * * *`.
///
/// After constructed, this type has various builder methods like [`.second()`][Cron::second]
/// and [`.hours()`][Cron::hours] to configure the details of the cron pattern.
///
/// # Example: once per day at a certain time
///
/// ```
/// # use recurring::pattern::Cron;
/// // Every day at 12:30.
/// let pattern = Cron::new().hour(12).minute(30);
/// ```
///
/// # Example: at multiple occasion thoughout the day
///
/// ```
/// # use recurring::pattern::Cron;
/// // Every day at 12:30, 13:30 and 14:30.
/// let pattern = Cron::new().hours(12..15).minute(30);
/// ```
///
/// # Example: multiple non-consecutive time units
///
/// ```
/// # use recurring::pattern::Cron;
/// // Every day at 10:30 and 20:30.
/// let pattern = Cron::new().hour(10).hour(20).minute(30);
/// // Or
/// let pattern = Cron::new().hours([10, 20]).minute(30);
/// ```
///
/// # Example: pitfalls
///
/// Because `Cron` behaves pretty much like a cron pattern, the following is expected behaviour
/// but might not be what you want.
///
/// ```
/// # use recurring::pattern::Cron;
/// // This ticks at 10:15 and 20:30. But it also ticks at 10:30 and 20:15.
/// let pattern = Cron::new()
///     .hour(10).minute(15)
///     .hour(20).minute(30);
///
/// // And so does this.
/// let pattern = Cron::new().hours([10, 20]).minutes([15, 30]);
///
/// // Let's break it down:
/// let pattern = Cron::new()
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
/// # use recurring::pattern::Cron;
/// let pattern = Cron::new().months(6..=9).hours(10..=20).minutes(15..30);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Cron {
    years: Years,
    months: Months,
    weekdays: Weekdays,
    days: Days,
    hours: Hours,
    minutes: Minutes,
    seconds: Seconds,
}

impl Cron {
    /// Create a new `Cron` that ticks every second.
    pub fn new() -> Cron {
        Cron::default()
    }
}

// Panicking builder methods.
impl Cron {
    /// Limit the pattern to a specific year.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different
    /// years. Alternatively, you can use [`.years()`][Cron::years] to feed years from an
    /// iterator.
    ///
    /// The fallible version of this method is [`Cron::try_year`].
    ///
    /// # Panics
    ///
    /// This panics when the year is too small or too big. The minimum value is `-9999`. The
    /// maximum value is `9999`.
    #[must_use]
    pub fn year(self, year: i16) -> Cron {
        self.try_year(year)
            .expect("value for year is out of bounds")
    }

    /// Limit the years in the pattern to every `step`'s year from `start` onwards.
    ///
    /// The fallible version of this method is [`Cron::try_year_step_by`].
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `-9999`. The maximum start value is `9999`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn year_step_by(self, start: i16, step: usize) -> Cron {
        self.try_year_step_by(start, step)
            .expect("value for year step is out of bounds")
    }

    /// Limit the years in the pattern to specific values from an iterator.
    ///
    /// The fallible version of this method is [`Cron::try_years`].
    ///
    /// # Panics
    ///
    /// This panics when any of the year values produced by the iterator is year is too small
    /// or too big. The minimum value is `-9999`. The maximum value is `9999`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn years<I: IntoIterator<Item = i16>>(self, years: I) -> Cron {
        self.try_years(years)
            .expect("value for years is out of bounds")
    }

    /// Limit the pattern to a specific month.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different
    /// months. Alternatively, you can use [`.months()`][Cron::months] to feed months from an
    /// iterator.
    ///
    /// The fallible version of this method is [`Cron::try_month`].
    ///
    /// # Panics
    ///
    /// This panics when the month is too small or too big. The minimum value is `1`. The maximum
    /// value is `12`.
    #[must_use]
    pub fn month(self, month: i8) -> Cron {
        self.try_month(month)
            .expect("value for month is out of bounds")
    }

    /// Limit the months in the pattern to every `step`'s month from `start` onwards.
    ///
    /// The fallible version of this method is [`Cron::try_month_step_by`].
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `1`. The maximum start value is `12`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn month_step_by(self, start: i8, step: usize) -> Cron {
        self.try_month_step_by(start, step)
            .expect("value for month step is out of bounds")
    }

    /// Limit the months in the pattern to specific values from an iterator.
    ///
    /// The fallible version of this method is [`Cron::try_months`].
    ///
    /// # Panics
    ///
    /// This panics when any of the month values produced by the iterator is month is too small
    /// or too big. The minimum value is `1`. The maximum value is `12`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn months<I: IntoIterator<Item = i8>>(self, months: I) -> Cron {
        self.try_months(months)
            .expect("value for months is out of bounds")
    }

    /// Limit the pattern to a specific weekday.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different
    /// weekdays. Alternatively, you can use [`.weekdays()`][Cron::weekdays] to feed weekdays
    /// from an iterator.
    #[must_use]
    pub fn weekday(self, weekday: Weekday) -> Cron {
        self.weekday_i8(weekday.to_monday_one_offset())
    }

    #[inline]
    fn weekday_i8(mut self, weekday: i8) -> Cron {
        self.weekdays
            .try_insert(weekday)
            .expect("weekday is out of bounds, please file a bug");
        self
    }

    /// Limit the weekdays in the pattern to every `step`'s weekday from `start` onwards.
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn weekday_step_by(self, start: Weekday, step: usize) -> Cron {
        (start.to_monday_one_offset()..=Weekdays::MAX)
            .step_by(step)
            .fold(self, Cron::weekday_i8)
    }

    /// Limit the weekdays in the pattern to specific values from an iterator.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn weekdays<I: IntoIterator<Item = Weekday>>(self, weekdays: I) -> Cron {
        weekdays.into_iter().fold(self, Cron::weekday)
    }

    /// Limit the pattern to a specific day.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different days.
    /// Alternatively, you can use [`.days()`][Cron::days] to feed days from an iterator.
    ///
    /// The fallible version of this method is [`Cron::try_day`].
    ///
    /// # Panics
    ///
    /// This panics when the day is too small or too big. The minimum value is `1`. The maximum
    /// value is `31`.
    #[must_use]
    pub fn day(self, day: i8) -> Cron {
        self.try_day(day).expect("value for day is out of bounds")
    }

    /// Limit the days in the pattern to every `step`'s day from `start` onwards.
    ///
    /// The fallible version of this method is [`Cron::try_day_step_by`].
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `1`. The maximum start value is `31`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn day_step_by(self, start: i8, step: usize) -> Cron {
        self.try_day_step_by(start, step)
            .expect("value for day step is out of bounds")
    }

    /// Limit the days in the pattern to specific values from an iterator.
    ///
    /// The fallible version of this method is [`Cron::try_days`].
    ///
    /// # Panics
    ///
    /// This panics when any of the day values produced by the iterator is day is too small
    /// or too big. The minimum value is `1`. The maximum value is `31`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn days<I: IntoIterator<Item = i8>>(self, days: I) -> Cron {
        self.try_days(days)
            .expect("value for days is out of bounds")
    }

    /// Limit the pattern to a specific hour.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different hours.
    /// Alternatively, you can use [`.hours()`][Cron::hours] to feed hours from an iterator.
    ///
    /// The fallible version of this method is [`Cron::try_hour`].
    ///
    /// # Panics
    ///
    /// This panics when the hour is too small or too big. The minimum value is `0`. The maximum
    /// value is `23`.
    #[must_use]
    pub fn hour(self, hour: i8) -> Cron {
        self.try_hour(hour)
            .expect("value for hour is out of bounds")
    }

    /// Limit the hours in the pattern to every `step`'s hour from `start` onwards.
    ///
    /// The fallible version of this method is [`Cron::try_hour_step_by`].
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `0`. The maximum start value is `23`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn hour_step_by(self, start: i8, step: usize) -> Cron {
        self.try_hour_step_by(start, step)
            .expect("value for hour step is out of bounds")
    }

    /// Limit the hours in the pattern to specific values from an iterator.
    ///
    /// The fallible version of this method is [`Cron::try_hours`].
    ///
    /// # Panics
    ///
    /// This panics when any of the hour values produced by the iterator is hour is too small
    /// or too big. The minimum value is `0`. The maximum value is `23`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn hours<I: IntoIterator<Item = i8>>(self, hours: I) -> Cron {
        self.try_hours(hours)
            .expect("value for hours is out of bounds")
    }

    /// Limit the pattern to a specific minute.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different minutes.
    /// Alternatively, you can use [`.minutes()`][Cron::minutes] to feed minutes from an iterator.
    ///
    /// The fallible version of this method is [`Cron::try_minute`].
    ///
    /// # Panics
    ///
    /// This panics when the minute is too small or too big. The minimum value is `0`. The maximum
    /// value is `59`.
    #[must_use]
    pub fn minute(self, minute: i8) -> Cron {
        self.try_minute(minute)
            .expect("value for minute is out of bounds")
    }

    /// Limit the minutes in the pattern to every `step`'s minute from `start` onwards.
    ///
    /// The fallible version of this method is [`Cron::try_minute_step_by`].
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `0`. The maximum start value is `59`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn minute_step_by(self, start: i8, step: usize) -> Cron {
        self.try_minute_step_by(start, step)
            .expect("value for minute step is out of bounds")
    }

    /// Limit the minutes in the pattern to specific values from an iterator.
    ///
    /// The fallible version of this method is [`Cron::try_minutes`].
    ///
    /// # Panics
    ///
    /// This panics when any of the minute values produced by the iterator is minute is too small
    /// or too big. The minimum value is `0`. The maximum value is `59`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn minutes<I: IntoIterator<Item = i8>>(self, minutes: I) -> Cron {
        self.try_minutes(minutes)
            .expect("value for minutes is out of bounds")
    }

    /// Limit the pattern to a specific second.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different seconds.
    /// Alternatively, you can use [`.seconds()`][Cron::seconds] to feed seconds from an iterator.
    ///
    /// The fallible version of this method is [`Cron::try_second`].
    ///
    /// # Panics
    ///
    /// This panics when the second is too small or too big. The minimum value is `0`. The maximum
    /// value is `59`.
    #[must_use]
    pub fn second(self, second: i8) -> Cron {
        self.try_second(second)
            .expect("value for second is out of bounds")
    }

    /// Limit the seconds in the pattern to every `step`'s second from `start` onwards.
    ///
    /// The fallible version of this method is [`Cron::try_second_step_by`].
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `0`. The maximum start value is `59`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn second_step_by(self, start: i8, step: usize) -> Cron {
        self.try_second_step_by(start, step)
            .expect("value for seconds step is out of bounds")
    }

    /// Limit the seconds in the pattern to specific values from an iterator.
    ///
    /// The fallible version of this method is [`Cron::try_seconds`].
    ///
    /// # Panics
    ///
    /// This panics when any of the second values produced by the iterator is second is too small
    /// or too big. The minimum value is `0`. The maximum value is `59`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn seconds<I: IntoIterator<Item = i8>>(self, seconds: I) -> Cron {
        self.try_seconds(seconds)
            .expect("value for seconds is out of bounds")
    }
}

// Fallible builder methods.
impl Cron {
    /// Limit the pattern to a specific year.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different years.
    /// Alternatively, you can use [`.try_years()`][Cron::try_years] to feed years from an
    /// iterator.
    ///
    /// The panicking version of this method is [`Cron::year`].
    ///
    /// # Errors
    ///
    /// This returns an error when the year is too small or too big. The minimum value is `-9999`.
    /// The maximum value is `9999`.
    pub fn try_year(mut self, year: i16) -> Result<Cron, Error> {
        self.years.try_insert(year)?;
        Ok(self)
    }

    /// Limit the years in the pattern to every `step`'s year from `start` onwards.
    ///
    /// # Errors
    ///
    /// The returns an error if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `-9999`. The maximum start value is `9999`.
    ///
    /// The panicking version of this method is [`Cron::year_step_by`].
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0`.
    #[cfg(feature = "alloc")]
    pub fn try_year_step_by(self, start: i16, step: usize) -> Result<Cron, Error> {
        (start..=Years::MAX)
            .step_by(step)
            .try_fold(self, Cron::try_year)
    }

    /// Limit the years in the pattern to specific values from an iterator.
    ///
    /// The panicking version of this method is [`Cron::years`].
    ///
    /// # Errors
    ///
    /// This returns an error when any of the year values produced by the iterator is too small or
    /// too big. The minimum value is `-9999`. The maximum value is `9999`.
    #[cfg(feature = "alloc")]
    pub fn try_years<I: IntoIterator<Item = i16>>(self, years: I) -> Result<Cron, Error> {
        years.into_iter().try_fold(self, Cron::try_year)
    }

    /// Limit the pattern to a specific month.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different months.
    /// Alternatively, you can use [`.months()`][Cron::months] to feed months from an iterator.
    ///
    /// The panicking version of this method is [`Cron::month`].
    ///
    /// # Errors
    ///
    /// This returns an error when the month is too small or too big. The minimum value is `1`.
    /// The maximum value is `12`.
    pub fn try_month(mut self, month: i8) -> Result<Cron, Error> {
        self.months.try_insert(month)?;
        Ok(self)
    }

    /// Limit the months in the pattern to every `step`'s month from `start` onwards.
    ///
    /// The panicking version of this method is [`Cron::month_step_by`].
    ///
    /// # Errors
    ///
    /// The returns an error if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `1`. The maximum start value is `12`.
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0`.
    #[cfg(feature = "alloc")]
    pub fn try_month_step_by(self, start: i8, step: usize) -> Result<Cron, Error> {
        (start..=Months::MAX)
            .step_by(step)
            .try_fold(self, Cron::try_month)
    }

    /// Limit the months in the pattern to specific values from an iterator.
    ///
    /// The panicking version of this method is [`Cron::months`].
    ///
    /// # Errors
    ///
    /// This returns an error when any of the month values produced by the iterator is month is
    /// too small or too big. The minimum value is `1`. The maximum value is `12`.
    #[cfg(feature = "alloc")]
    pub fn try_months<I: IntoIterator<Item = i8>>(self, months: I) -> Result<Cron, Error> {
        months.into_iter().try_fold(self, Cron::try_month)
    }

    /// Limit the pattern to a specific day.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different days.
    /// Alternatively, you can use [`.days()`][Cron::days] to feed days from an iterator.
    ///
    /// The panicking version of this method is [`Cron::day`].
    ///
    /// # Errors
    ///
    /// This returns an error when the day is too small or too big. The minimum value is `1`.
    /// The maximum value is `31`.
    pub fn try_day(mut self, day: i8) -> Result<Cron, Error> {
        self.days.try_insert(day)?;
        Ok(self)
    }

    /// Limit the days in the pattern to every `step`'s day from `start` onwards.
    ///
    /// The panicking version of this method is [`Cron::day_step_by`].
    ///
    /// # Errors
    ///
    /// The returns an error if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `1`. The maximum start value is `31`.
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0`.
    #[cfg(feature = "alloc")]
    pub fn try_day_step_by(self, start: i8, step: usize) -> Result<Cron, Error> {
        (start..=Days::MAX)
            .step_by(step)
            .try_fold(self, Cron::try_day)
    }

    /// Limit the days in the pattern to specific values from an iterator.
    ///
    /// The panicking version of this method is [`Cron::days`].
    ///
    /// # Errors
    ///
    /// This returns an error when any of the day values produced by the iterator is day is
    /// too small or too big. The minimum value is `1`. The maximum value is `31`.
    #[cfg(feature = "alloc")]
    pub fn try_days<I: IntoIterator<Item = i8>>(self, days: I) -> Result<Cron, Error> {
        days.into_iter().try_fold(self, Cron::try_day)
    }

    /// Limit the pattern to a specific hour.
    ///
    /// The panicking version of this method is [`Cron::hour`].
    ///
    /// This method can be called multiple times to limit the pattern to multiple different hours.
    /// Alternatively, you can use [`.hours()`][Cron::hours] to feed hours from an iterator.
    ///
    /// # Errors
    ///
    /// This returns an error when the hour is too small or too big. The minimum value is `0`.
    /// The maximum value is `23`.
    pub fn try_hour(mut self, hour: i8) -> Result<Cron, Error> {
        self.hours.try_insert(hour)?;
        Ok(self)
    }

    /// Limit the hours in the pattern to every `step`'s hour from `start` onwards.
    ///
    /// The panicking version of this method is [`Cron::hour_step_by`].
    ///
    /// # Errors
    ///
    /// The returns an error if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `0`. The maximum start value is `23`.
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0`.
    #[cfg(feature = "alloc")]
    pub fn try_hour_step_by(self, start: i8, step: usize) -> Result<Cron, Error> {
        (start..=Hours::MAX)
            .step_by(step)
            .try_fold(self, Cron::try_hour)
    }

    /// Limit the hours in the pattern to specific values from an iterator.
    ///
    /// The panicking version of this method is [`Cron::hours`].
    ///
    /// # Errors
    ///
    /// This returns an error when any of the hour values produced by the iterator is hour is
    /// too small or too big. The minimum value is `0`. The maximum value is `23`.
    #[cfg(feature = "alloc")]
    pub fn try_hours<I: IntoIterator<Item = i8>>(self, hours: I) -> Result<Cron, Error> {
        hours.into_iter().try_fold(self, Cron::try_hour)
    }

    /// Limit the pattern to a specific minute.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different minutes.
    /// Alternatively, you can use [`.minutes()`][Cron::minutes] to feed minutes from an iterator.
    ///
    /// The panicking version of this method is [`Cron::minute`].
    ///
    /// # Errors
    ///
    /// This returns an error when the minute is too small or too big. The minimum value is `0`.
    /// The maximum value is `59`.
    pub fn try_minute(mut self, minute: i8) -> Result<Cron, Error> {
        self.minutes.try_insert(minute)?;
        Ok(self)
    }

    /// Limit the minutes in the pattern to every `step`'s minute from `start` onwards.
    ///
    /// The panicking version of this method is [`Cron::minute_step_by`].
    ///
    /// # Errors
    ///
    /// The returns an error if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `0`. The maximum start value is `59`.
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0`.
    #[cfg(feature = "alloc")]
    pub fn try_minute_step_by(self, start: i8, step: usize) -> Result<Cron, Error> {
        (start..=Minutes::MAX)
            .step_by(step)
            .try_fold(self, Cron::try_minute)
    }

    /// Limit the minutes in the pattern to specific values from an iterator.
    ///
    /// The panicking version of this method is [`Cron::minutes`].
    ///
    /// # Errors
    ///
    /// This returns an error when any of the minute values produced by the iterator is minute is
    /// too small or too big. The minimum value is `0`. The maximum value is `59`.
    #[cfg(feature = "alloc")]
    pub fn try_minutes<I: IntoIterator<Item = i8>>(self, minutes: I) -> Result<Cron, Error> {
        minutes.into_iter().try_fold(self, Cron::try_minute)
    }

    /// Limit the pattern to a specific second.
    ///
    /// This method can be called multiple times to limit the pattern to multiple different seconds.
    /// Alternatively, you can use [`.seconds()`][Cron::seconds] to feed seconds from an iterator.
    ///
    /// The panicking version of this method is [`Cron::second`].
    ///
    /// # Errors
    ///
    /// This returns an error when the second is too small or too big. The minimum value is `0`.
    /// The maximum value is `59`.
    pub fn try_second(mut self, second: i8) -> Result<Cron, Error> {
        self.seconds.try_insert(second)?;
        Ok(self)
    }

    /// Limit the seconds in the pattern to every `step`'s second from `start` onwards.
    ///
    /// The panicking version of this method is [`Cron::second_step_by`].
    ///
    /// # Errors
    ///
    /// The returns an error if the given step is `0` or if start is too small or too big. The
    /// minimum start value is `0`. The maximum start value is `59`.
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0`.
    #[cfg(feature = "alloc")]
    pub fn try_second_step_by(self, start: i8, step: usize) -> Result<Cron, Error> {
        (start..=Seconds::MAX)
            .step_by(step)
            .try_fold(self, Cron::try_second)
    }

    /// Limit the seconds in the pattern to specific values from an iterator.
    ///
    /// The panicking version of this method is [`Cron::seconds`].
    ///
    /// # Errors
    ///
    /// This returns an error when any of the second values produced by the iterator is second is
    /// too small or too big. The minimum value is `0`. The maximum value is `59`.
    #[cfg(feature = "alloc")]
    pub fn try_seconds<I: IntoIterator<Item = i8>>(self, seconds: I) -> Result<Cron, Error> {
        seconds.into_iter().try_fold(self, Cron::try_second)
    }
}

impl Cron {
    fn next_after_or_current(
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

    fn previous_before_or_current(
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

impl Pattern for Cron {
    fn next_after(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let instant = instant.checked_add(1.second()).ok()?;
        self.next_after_or_current(instant, range)
    }

    fn previous_before(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let instant = if instant.subsec_nanosecond() > 0 {
            instant
        } else {
            instant.checked_sub(1.second()).ok()?
        };
        self.previous_before_or_current(instant, range)
    }

    fn closest_to(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let instant = instant.max(range.start).min(range.end);

        let Some(next) = self.next_after_or_current(instant, range) else {
            return self.previous_before(instant, range);
        };

        if next == instant {
            return Some(next);
        }

        let Some(previous) = self.previous_before(instant, range) else {
            return Some(next);
        };

        if instant.duration_since(previous) >= instant.duration_until(next) {
            Some(next)
        } else {
            Some(previous)
        }
    }
}

impl private::Sealed for Cron {}

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
