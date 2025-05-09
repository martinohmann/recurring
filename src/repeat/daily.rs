use crate::{Error, Repeat, repeat::Interval};
use alloc::vec::Vec;
use core::ops::Range;
use jiff::{
    ToSpan,
    civil::{DateTime, Time},
};

/// An interval for daily events which may also include fixed times of the day.
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
    interval: Interval,
    at: Vec<Time>,
}

impl Daily {
    /// Creates a new `Daily` from an interval of days.
    ///
    /// For a fallible alternative see [`Daily::try_new`].
    ///
    /// # Panics
    ///
    /// Panics if `interval` is negative or zero.
    ///
    /// # Example
    ///
    /// ```
    /// use recurring::repeat::Daily;
    ///
    /// let every_two_days = Daily::new(2);
    /// ```
    pub fn new<I: ToSpan>(interval: I) -> Daily {
        Daily {
            interval: Interval::new(interval.days()),
            at: Vec::new(),
        }
    }

    /// Creates a new `Daily` from an interval of days.
    ///
    /// For an infallible alternative that panics on invalid spans instead see [`Daily::new`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if `interval` is negative or zero.
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
    pub fn try_new<I: ToSpan>(interval: I) -> Result<Daily, Error> {
        Ok(Daily {
            interval: Interval::try_new(interval.days())?,
            at: Vec::new(),
        })
    }

    /// Adds a time to the daily repeat interval.
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

    /// Adds times to the daily repeat interval.
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
    fn next_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        if self.at.is_empty() {
            return self.interval.next_event(instant, range);
        }

        let closest = self.closest_event(instant, range)?;
        if closest > instant {
            return Some(closest);
        }

        if let Some(after) = self.get_daily_after(instant) {
            return Some(after);
        }

        self.interval
            .next_event(instant, range)
            .and_then(|next| self.get_daily_after(next))
    }

    fn previous_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        if self.at.is_empty() {
            return self.interval.previous_event(instant, range);
        }

        let closest = self.closest_event(instant, range)?;
        if closest < instant {
            return Some(closest);
        }

        if let Some(before) = self.get_daily_before(instant) {
            return Some(before);
        }

        self.interval
            .previous_event(instant, range)
            .and_then(|previous| self.get_daily_before(previous))
    }

    fn closest_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        if self.at.is_empty() {
            return self.interval.closest_event(instant, range);
        }

        let closest = self.interval.closest_event(instant, range)?;

        self.at
            .iter()
            .filter_map(|time| closest.with().time(*time).build().ok())
            .filter(|date| range.contains(date))
            .min_by_key(|date| date.duration_since(instant).abs())
    }
}
