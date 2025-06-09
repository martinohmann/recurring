use crate::pattern::Interval;
use crate::{DateTimeRange, Error, Pattern, private};
use jiff::{
    ToSpan,
    civil::{DateTime, Time},
};

/// A recurrence pattern for daily events which may also include fixed times of the day.
///
/// # Example
///
/// ```
/// use recurring::pattern::Daily;
/// use jiff::civil::time;
///
/// let every_two_days_at_twelve = Daily::new(1).at(time(12, 0, 0, 0));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Daily {
    interval: Interval,
    at: Option<Time>,
}

impl Daily {
    /// Creates a new `Daily` from an interval of days.
    ///
    /// The fallible version of this method is [`Daily::try_new`].
    ///
    /// # Panics
    ///
    /// Panics if `interval` is negative or zero.
    ///
    /// # Example
    ///
    /// ```
    /// use recurring::pattern::Daily;
    ///
    /// let every_two_days = Daily::new(2);
    /// ```
    pub fn new<I: ToSpan>(interval: I) -> Daily {
        Daily {
            interval: Interval::new(interval.days()),
            at: None,
        }
    }

    /// Creates a new `Daily` from an interval of days.
    ///
    /// The panicking version of this method is [`Daily::new`].
    ///
    /// # Errors
    ///
    /// Returns an `Error` if `interval` is negative or zero.
    ///
    /// # Example
    ///
    /// ```
    /// use recurring::pattern::Daily;
    ///
    /// assert!(Daily::try_new(1).is_ok());
    /// assert!(Daily::try_new(0).is_err());
    /// assert!(Daily::try_new(-1).is_err());
    /// ```
    pub fn try_new<I: ToSpan>(interval: I) -> Result<Daily, Error> {
        Ok(Daily {
            interval: Interval::try_new(interval.days())?,
            at: None,
        })
    }

    /// Sets the exact time of day for the daily recurrence.
    ///
    /// # Example
    ///
    /// ```
    /// use jiff::civil::time;
    /// use recurring::pattern::daily;
    ///
    /// let every_day_at_twelve = daily(1).at(time(12, 0, 0, 0));
    ///
    /// let every_day_at_midnight_and_twelve = every_day_at_twelve.at(time(0, 0, 0, 0));
    /// ```
    #[must_use]
    pub fn at<T: Into<Time>>(mut self, time: T) -> Daily {
        self.at = Some(time.into());
        self
    }

    fn range_adjusted<F>(&self, f: F, instant: DateTime, range: DateTimeRange) -> Option<DateTime>
    where
        F: FnOnce(&Interval, DateTime, DateTimeRange) -> Option<DateTime>,
    {
        let Some(time) = self.at else {
            return f(&self.interval, instant, range);
        };

        let start = if range.start.time() <= time {
            range.start
        } else {
            self.interval.next_after(range.start, range)?
        };

        let start = start.with().time(time).build().ok()?;
        let range = DateTimeRange::from(start..range.end);

        f(&self.interval, instant, range)
    }
}

impl Pattern for Daily {
    fn next_after(&self, instant: DateTime, range: DateTimeRange) -> Option<DateTime> {
        self.range_adjusted(Interval::next_after, instant, range)
    }

    fn previous_before(&self, instant: DateTime, range: DateTimeRange) -> Option<DateTime> {
        self.range_adjusted(Interval::previous_before, instant, range)
    }

    fn closest_to(&self, instant: DateTime, range: DateTimeRange) -> Option<DateTime> {
        self.range_adjusted(Interval::closest_to, instant, range)
    }
}

impl private::Sealed for Daily {}
