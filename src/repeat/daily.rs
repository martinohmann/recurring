use crate::{Error, Repeat, private, repeat::Period};
use alloc::vec::Vec;
use core::ops::Range;
use jiff::{
    ToSpan,
    civil::{DateTime, Time},
};

/// A repeat behaviour for daily events which may also include fixed times of the day.
///
/// # Example
///
/// ```
/// use recurring::repeat::Daily;
/// use jiff::civil::time;
///
/// let every_two_days_at_twelve = Daily::new(1).at(time(12, 0, 0, 0));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Daily {
    period: Period,
    at: Vec<Time>,
}

impl Daily {
    /// Creates a new `Daily` from a period of days.
    ///
    /// For a fallible alternative see [`Daily::try_new`].
    ///
    /// # Panics
    ///
    /// Panics if `period` is negative or zero.
    ///
    /// # Example
    ///
    /// ```
    /// use recurring::repeat::Daily;
    ///
    /// let every_two_days = Daily::new(2);
    /// ```
    pub fn new<I: ToSpan>(period: I) -> Daily {
        Daily {
            period: Period::new(period.days()),
            at: Vec::new(),
        }
    }

    /// Creates a new `Daily` from a period of days.
    ///
    /// For an infallible alternative that panics on invalid spans instead see [`Daily::new`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if `period` is negative or zero.
    ///
    /// # Example
    ///
    /// ```
    /// use recurring::repeat::Daily;
    ///
    /// assert!(Daily::try_new(1).is_ok());
    /// assert!(Daily::try_new(0).is_err());
    /// assert!(Daily::try_new(-1).is_err());
    /// ```
    pub fn try_new<I: ToSpan>(period: I) -> Result<Daily, Error> {
        Ok(Daily {
            period: Period::try_new(period.days())?,
            at: Vec::new(),
        })
    }

    /// Adds a time to the daily repeat period.
    ///
    /// # Example
    ///
    /// ```
    /// use recurring::repeat::daily;
    /// use jiff::civil::time;
    ///
    /// let every_day_at_twelve = daily(1).at(time(12, 0, 0, 0));
    ///
    /// let every_day_at_midnight_and_twelve = every_day_at_twelve.at(time(0, 0, 0, 0));
    /// ```
    #[must_use]
    pub fn at<T: Into<Time>>(self, time: T) -> Daily {
        self.at_times([time])
    }

    /// Adds times to the daily repeat period.
    ///
    /// # Example
    ///
    /// ```
    /// use recurring::repeat::daily;
    /// use jiff::civil::time;
    ///
    /// let every_day_at_midnight_and_twelve = daily(1)
    ///     .at_times([time(0, 0, 0, 0), time(12, 0, 0, 0)]);
    /// ```
    #[must_use]
    pub fn at_times<T: IntoIterator<Item = impl Into<Time>>>(mut self, times: T) -> Daily {
        self.at.extend(times.into_iter().map(Into::into));
        self.at.sort();
        self.at.dedup();
        self
    }

    fn get_daily_before(&self, instant: DateTime) -> Option<DateTime> {
        for time in self.at.iter().rev() {
            let date = instant.with().time(*time).build().ok()?;

            if date < instant {
                return Some(date);
            }
        }

        None
    }

    fn get_daily_after(&self, instant: DateTime) -> Option<DateTime> {
        for time in &self.at {
            let date = instant.with().time(*time).build().ok()?;

            if date > instant {
                return Some(date);
            }
        }

        None
    }
}

impl Repeat for Daily {
    fn next_after(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        if self.at.is_empty() {
            return self.period.next_after(instant, range);
        }

        let closest = self.closest_to(instant, range)?;
        if closest > instant {
            return Some(closest);
        }

        if let Some(after) = self.get_daily_after(instant) {
            return Some(after);
        }

        self.period
            .next_after(instant, range)
            .and_then(|next| self.get_daily_after(next))
    }

    fn previous_before(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        if self.at.is_empty() {
            return self.period.previous_before(instant, range);
        }

        let closest = self.closest_to(instant, range)?;
        if closest < instant {
            return Some(closest);
        }

        if let Some(before) = self.get_daily_before(instant) {
            return Some(before);
        }

        self.period
            .previous_before(instant, range)
            .and_then(|previous| self.get_daily_before(previous))
    }

    fn closest_to(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        if self.at.is_empty() {
            return self.period.closest_to(instant, range);
        }

        let closest = self.period.closest_to(instant, range)?;

        self.at
            .iter()
            .filter_map(|time| closest.with().time(*time).build().ok())
            .filter(|date| range.contains(date))
            .min_by_key(|date| date.duration_since(instant).abs())
    }
}

impl private::Sealed for Daily {}
