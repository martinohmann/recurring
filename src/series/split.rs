use crate::Pattern;
use crate::error::{Error, err};
use crate::series::Series;
use core::ops::RangeBounds;
use jiff::{
    Zoned,
    civil::{Date, DateTime},
};

/// Options for controlling the behaviour of [`Series::split_off`].
#[derive(Debug, Clone, Copy, Default)]
pub enum SplitMode {
    /// Split the series exactly where specified.
    #[default]
    At,
    /// Split the series at the next event after the specified instant.
    NextAfter,
    /// Split the series at the previous event before the specified instant.
    PreviousBefore,
    /// Split the series at the event closest to the specified instant.
    ClosestTo,
}

/// Options for [`Series::split_off`].
///
/// This type provides a way to configure the behaviour for splitting a series. In particular,
/// `Series::split_off` accepts anything that implements the `Into<SeriesSplit>` trait. There are
/// some trait implementations that therefore make calling `Series::split_off` in some common cases
/// more ergonomic:
///
/// * `From<DateTime> for SeriesSplit` will construct a series split configuration which splits the
///   series at a given datetime. The `Date`, `Zoned` and `&Zoned` types can be used instead of
///   `DateTime` as well.
/// * `From<(SplitMode, T)> for SeriesSplit where T: Into<SeriesSplit>` will construct a series
///   split configuration using a certain [`SplitMode`]. This enables splitting at the next,
///   previous or closest event.
///
/// Note that in the default configuration, series are always split at the exact datetime provided
/// by the user.
#[derive(Debug, Clone, Copy)]
pub struct SeriesSplit {
    instant: DateTime,
    mode: SplitMode,
}

impl SeriesSplit {
    /// Creates a new `SeriesSplit` configuration for splitting a series at a given `instant`.
    #[inline]
    pub fn new(instant: DateTime) -> SeriesSplit {
        SeriesSplit {
            instant,
            mode: SplitMode::default(),
        }
    }

    /// Set the split mode.
    ///
    /// This defaults to [`SplitMode::At`], which splits the series at the exact datetime provided
    /// by the user.
    ///
    /// # Example: split at the closest event to a given datetime
    ///
    /// ```
    /// use jiff::{ToSpan, civil::{date, DateTime}};
    /// use recurring::{Event, Series, pattern::hourly};
    /// use recurring::series::{SeriesSplit, SplitMode};
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let mut s1 = Series::new(start.., hourly(1));
    ///
    /// let instant = date(2025, 4, 1).at(12, 34, 56, 0);
    ///
    /// let s2 = s1.split_off(SeriesSplit::new(instant).mode(SplitMode::ClosestTo))?;
    ///
    /// assert_eq!(s1.end(), date(2025, 4, 1).at(13, 0, 0, 0));
    /// assert_eq!(s2.start(), date(2025, 4, 1).at(13, 0, 0, 0));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[must_use]
    pub fn mode(mut self, mode: SplitMode) -> SeriesSplit {
        self.mode = mode;
        self
    }

    /// Splits off a part of the series according to the configuration of `self`.
    pub(crate) fn split_off<P: Pattern>(self, series: &mut Series<P>) -> Result<Series<P>, Error> {
        let cutoff_point = self.find_cutoff_point(series)?;
        let left = series.with().end(cutoff_point).build()?;
        let right = series
            .with()
            .start(cutoff_point)
            // Ensure the new series uses the same fixpoint to not mess up relative patterns.
            .fixpoint(series.fixpoint())
            .build()?;

        *series = left;
        Ok(right)
    }

    fn find_cutoff_point<P: Pattern>(&self, series: &Series<P>) -> Result<DateTime, Error> {
        match self.mode {
            SplitMode::At => {
                if series.range.contains(&self.instant) {
                    Ok(self.instant)
                } else {
                    Err(err!("{} not within series range", self.instant))
                }
            }
            SplitMode::NextAfter => series
                .pattern()
                .next_after(self.instant, series.range)
                .ok_or_else(|| err!("no series event after {}", self.instant)),
            SplitMode::PreviousBefore => series
                .pattern()
                .previous_before(self.instant, series.range)
                .ok_or_else(|| err!("no series event before {}", self.instant)),
            SplitMode::ClosestTo => series
                .pattern()
                .closest_to(self.instant, series.range)
                .ok_or_else(|| err!("no series event close to {}", self.instant)),
        }
    }
}

impl From<DateTime> for SeriesSplit {
    fn from(instant: DateTime) -> Self {
        SeriesSplit::new(instant)
    }
}

impl From<Date> for SeriesSplit {
    fn from(date: Date) -> Self {
        SeriesSplit::new(date.into())
    }
}

impl From<Zoned> for SeriesSplit {
    fn from(zoned: Zoned) -> Self {
        SeriesSplit::new(zoned.datetime())
    }
}

impl From<&Zoned> for SeriesSplit {
    fn from(zoned: &Zoned) -> Self {
        SeriesSplit::new(zoned.datetime())
    }
}

impl<T: Into<SeriesSplit>> From<(SplitMode, T)> for SeriesSplit {
    fn from((mode, split): (SplitMode, T)) -> Self {
        let split: SeriesSplit = split.into();
        split.mode(mode)
    }
}
