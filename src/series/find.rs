use crate::series::Series;
use crate::{Event, Pattern};
use jiff::{
    Zoned,
    civil::{Date, DateTime},
};

/// Options for controlling the behaviour of [`Series::find`].
#[derive(Debug, Clone, Copy, Default)]
pub enum FindMode {
    /// Find an event at an exact instant.
    At,
    /// Find the next event after an instant.
    #[default]
    NextAfter,
    /// Find the previous event before an instant.
    PreviousBefore,
    /// Find the event closest to an instant.
    ///
    /// This mode finds an event happening either at, before or after an instant.
    ClosestTo,
    /// Find the event containing an instant.
    ///
    /// For series' with a positive non-zero event duration, this mode finds an event that either starts at
    /// an instant, or that contains the instant within its duration. For series with an event
    /// duration of zero, this behaves exactly like [`FindMode::At`].
    Containing,
}

/// Options for [`Series::find`].
///
/// This type provides a way to configure the behaviour for finding events in a series. In
/// particular, `Series::find` accepts anything that implements the `Into<SeriesFind>` trait. There
/// are some trait implementations that therefore make calling `Series::find` in some common cases
/// more ergonomic:
///
/// * `From<DateTime> for SeriesFind` will construct a series find configuration which finds the
///   next series event after a given datetime. The `Date`, `Zoned` and `&Zoned` types can be used
///   instead of `DateTime` as well.
/// * `From<(FindMode, T)> for SeriesFind where T: Into<SeriesFind>` will construct a series
///   find configuration using a certain [`FindMode`]. This enables finding the next, previous,
///   closest, containing or exact event for a given datetime.
///
/// Note that in the default configuration, `SeriesFind` search for the next event after the
/// datetime provided by the user.
#[derive(Debug, Clone)]
pub struct SeriesFind {
    instant: DateTime,
    mode: FindMode,
}

impl SeriesFind {
    /// Creates a new `SeriesFind` configuration for finding series events close to a given
    /// `instant`.
    #[inline]
    pub fn new(instant: DateTime) -> SeriesFind {
        SeriesFind {
            instant,
            mode: FindMode::default(),
        }
    }

    /// Set the find mode.
    ///
    /// This defaults to [`FindMode::NextAfter`], which finds the next event after the datetime
    /// provided by the user.
    ///
    /// # Example: find the event closest to a given datetime
    ///
    /// ```
    /// use jiff::{ToSpan, civil::{date, DateTime}};
    /// use recurring::{Event, Series, pattern::hourly};
    /// use recurring::series::{SeriesFind, FindMode};
    ///
    /// let start = date(2025, 1, 1).at(0, 0, 0, 0);
    /// let series = Series::new(start.., hourly(1));
    ///
    /// let instant = date(2025, 4, 1).at(12, 34, 56, 0);
    /// let event = series.find(SeriesFind::new(instant).mode(FindMode::ClosestTo));
    ///
    /// assert_eq!(event, Some(Event::at(date(2025, 4, 1).at(13, 0, 0, 0))));
    /// # Ok::<(), Box<dyn core::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn mode(mut self, mode: FindMode) -> SeriesFind {
        self.mode = mode;
        self
    }

    /// Finds a series event according to the configuration of `self`.
    pub(crate) fn find<P: Pattern>(self, series: &Series<P>) -> Option<Event> {
        let Series { core, range } = series;

        match self.mode {
            FindMode::At => core.get(self.instant, *range),
            FindMode::NextAfter => core.get_next_after(self.instant, *range),
            FindMode::PreviousBefore => core.get_previous_before(self.instant, *range),
            FindMode::ClosestTo => core.get_closest_to(self.instant, *range),
            FindMode::Containing => core.get_containing(self.instant, *range),
        }
    }
}

impl From<DateTime> for SeriesFind {
    fn from(instant: DateTime) -> Self {
        SeriesFind::new(instant)
    }
}

impl From<Date> for SeriesFind {
    fn from(date: Date) -> Self {
        SeriesFind::new(date.into())
    }
}

impl From<Zoned> for SeriesFind {
    fn from(zoned: Zoned) -> Self {
        SeriesFind::new(zoned.datetime())
    }
}

impl From<&Zoned> for SeriesFind {
    fn from(zoned: &Zoned) -> Self {
        SeriesFind::new(zoned.datetime())
    }
}

impl<T: Into<SeriesFind>> From<(FindMode, T)> for SeriesFind {
    fn from((mode, find): (FindMode, T)) -> Self {
        let find: SeriesFind = find.into();
        find.mode(mode)
    }
}
