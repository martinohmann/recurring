use alloc::collections::btree_set::{self, BTreeSet};
use core::ops::RangeInclusive;

pub(super) type Weekdays = TimeUnits<1, 7>;
pub(super) type Months = TimeUnits<1, 12>;
pub(super) type Days = TimeUnits<1, 31>;
pub(super) type Hours = TimeUnits<0, 23>;
pub(super) type Minutes = TimeUnits<0, 59>;
pub(super) type Seconds = TimeUnits<0, 59>;

/// A bounded set of time units.
///
/// The `Default` value of this type contains the full range of possible time units between `MIN`
/// (inclusive) and `MAX` (inclusive).
#[derive(Debug, Clone, Default)]
pub(super) struct TimeUnits<const MIN: i8, const MAX: i8> {
    set: BTreeSet<i8>,
}

impl<const MIN: i8, const MAX: i8> TimeUnits<MIN, MAX> {
    pub(super) const MIN: i8 = MIN;
    pub(super) const MAX: i8 = MAX;

    #[inline]
    const fn full_range() -> RangeInclusive<i8> {
        Self::MIN..=Self::MAX
    }

    #[inline]
    fn clamp_to_bounds(range: RangeInclusive<i8>) -> RangeInclusive<i8> {
        Self::MIN.max(*range.start())..=Self::MAX.min(*range.end())
    }

    pub(super) fn insert(&mut self, value: i8) -> bool {
        assert!(Self::full_range().contains(&value));
        self.set.insert(value)
    }

    pub(super) fn contains(&self, second: i8) -> bool {
        if self.set.is_empty() {
            Self::full_range().contains(&second)
        } else {
            self.set.contains(&second)
        }
    }

    pub(super) fn range(&self, range: RangeInclusive<i8>) -> TimeUnitRange<'_> {
        if self.set.is_empty() {
            TimeUnitRange::Range(Self::clamp_to_bounds(range))
        } else {
            TimeUnitRange::Set(self.set.range(range))
        }
    }
}

pub(super) enum TimeUnitRange<'a> {
    Range(RangeInclusive<i8>),
    Set(btree_set::Range<'a, i8>),
}

impl Iterator for TimeUnitRange<'_> {
    type Item = i8;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TimeUnitRange::Range(range) => range.next(),
            TimeUnitRange::Set(iter) => iter.next().copied(),
        }
    }
}

impl DoubleEndedIterator for TimeUnitRange<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            TimeUnitRange::Range(range) => range.next_back(),
            TimeUnitRange::Set(iter) => iter.next_back().copied(),
        }
    }
}

/// A bounded set of years.
///
/// The `Default` value of this type contains the full range of possible years.
#[derive(Debug, Clone, Default)]
pub(super) struct Years {
    set: BTreeSet<i16>,
}

impl Years {
    pub(super) const MIN: i16 = -9999;
    pub(super) const MAX: i16 = 9999;

    #[inline]
    const fn full_range() -> RangeInclusive<i16> {
        Self::MIN..=Self::MAX
    }

    #[inline]
    fn clamp_to_bounds(range: RangeInclusive<i16>) -> RangeInclusive<i16> {
        Self::MIN.max(*range.start())..=Self::MAX.min(*range.end())
    }

    pub(super) fn insert(&mut self, value: i16) -> bool {
        assert!(Self::full_range().contains(&value));
        self.set.insert(value)
    }

    pub(super) fn range(&self, range: RangeInclusive<i16>) -> YearRange<'_> {
        if self.set.is_empty() {
            YearRange::Range(Self::clamp_to_bounds(range))
        } else {
            YearRange::Set(self.set.range(range))
        }
    }
}

pub(super) enum YearRange<'a> {
    Range(RangeInclusive<i16>),
    Set(btree_set::Range<'a, i16>),
}

impl Iterator for YearRange<'_> {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            YearRange::Range(range) => range.next(),
            YearRange::Set(iter) => iter.next().copied(),
        }
    }
}

impl DoubleEndedIterator for YearRange<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            YearRange::Range(range) => range.next_back(),
            YearRange::Set(iter) => iter.next_back().copied(),
        }
    }
}
