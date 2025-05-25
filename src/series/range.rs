use crate::error::{Error, err};
use core::ops::{Bound, Range, RangeBounds};
use jiff::civil::DateTime;

/// Representation of the time range of a [`Series`][crate::Series].
///
/// This type is similar to [`Range<DateTime>`] but carries additional functionality used by the
/// [`Pattern`][crate::Pattern] trait. There's usually no need for users of this crate to
/// instantiate values of this type unless you want to interact with methods of the `Pattern` trait
/// directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeriesRange {
    /// The lower bound of the series (inclusive).
    pub(crate) start: DateTime,
    /// The upper bound of the series (exclusive).
    pub(crate) end: DateTime,
    /// An optional (inclusive) fixpoint used as a starting point for `Pattern` implementations
    /// that are relative to some point in time.
    ///
    /// If this is `None`, these implementations should use `start` as their fixpoint.
    pub(crate) fixpoint: Option<DateTime>,
}

impl SeriesRange {
    /// Creates a new `SeriesRange` from a start (inclusive) and an end (exclusive).
    #[inline]
    pub(crate) const fn new(start: DateTime, end: DateTime) -> SeriesRange {
        SeriesRange {
            start,
            end,
            fixpoint: None,
        }
    }

    /// Sets the (inclusive) fixpoint for relative recurrence patterns.
    ///
    /// This is used as a starting point for `Pattern` implementations that are relative to some
    /// point in time.
    ///
    /// # Errors
    ///
    /// Returns an error if `fixpoint` is greater than the range start.
    #[inline]
    pub(crate) fn with_fixpoint(mut self, fixpoint: DateTime) -> Result<SeriesRange, Error> {
        if fixpoint > self.start {
            return Err(err!(
                "fixpoint ({fixpoint}) must be less than or equal to range start ({})",
                self.start
            ));
        }
        self.fixpoint = Some(fixpoint);
        Ok(self)
    }

    /// Returns the (inclusive) fixpoint for relative recurrence patterns.
    ///
    /// Unless [`SeriesRange::with_fixpoint`] was called with a specific value, this returns the
    /// same value as [`SeriesRange::start`].
    #[inline]
    pub(crate) fn fixpoint(&self) -> DateTime {
        self.fixpoint.unwrap_or(self.start)
    }

    /// The lower bound of the series (inclusive).
    #[inline]
    pub const fn start(&self) -> DateTime {
        self.start
    }

    /// The upper bound of the series (exclusive).
    #[inline]
    pub const fn end(&self) -> DateTime {
        self.end
    }
}

impl RangeBounds<DateTime> for SeriesRange {
    fn start_bound(&self) -> Bound<&DateTime> {
        Bound::Included(&self.start)
    }

    fn end_bound(&self) -> Bound<&DateTime> {
        Bound::Excluded(&self.end)
    }
}

impl From<SeriesRange> for Range<DateTime> {
    fn from(range: SeriesRange) -> Self {
        range.start..range.end
    }
}

impl From<Range<DateTime>> for SeriesRange {
    fn from(range: Range<DateTime>) -> Self {
        SeriesRange::new(range.start, range.end)
    }
}
