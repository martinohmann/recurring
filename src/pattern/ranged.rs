use crate::error::Error;
use alloc::collections::btree_set::{self, BTreeSet};
use core::ops::RangeInclusive;

pub(super) type Years = RangedI16Set<-9999, 9999>;
pub(super) type Months = RangedI8Set<1, 12>;
pub(super) type Weekdays = RangedI8Set<1, 7>;
pub(super) type Days = RangedI8Set<1, 31>;
pub(super) type Hours = RangedI8Set<0, 23>;
pub(super) type Minutes = RangedI8Set<0, 59>;
pub(super) type Seconds = RangedI8Set<0, 59>;

/// A ranged set of i8 values.
///
/// The `Default` value of this type contains the full range of possible values between `MIN`
/// (inclusive) and `MAX` (inclusive).
#[derive(Debug, Clone, Default)]
pub(super) struct RangedI8Set<const MIN: i8, const MAX: i8>(BTreeSet<i8>);

impl<const MIN: i8, const MAX: i8> RangedI8Set<MIN, MAX> {
    pub(super) const MIN: i8 = MIN;
    pub(super) const MAX: i8 = MAX;

    #[inline]
    fn is_within_bounds(value: i8) -> bool {
        (Self::MIN..=Self::MAX).contains(&value)
    }

    pub(super) fn try_insert(&mut self, value: i8) -> Result<bool, Error> {
        if Self::is_within_bounds(value) {
            Ok(self.0.insert(value))
        } else {
            Err(Error::range(value, Self::MIN, Self::MAX))
        }
    }

    pub(super) fn contains(&self, value: i8) -> bool {
        if self.0.is_empty() {
            Self::is_within_bounds(value)
        } else {
            self.0.contains(&value)
        }
    }

    pub(super) fn range(&self, range: RangeInclusive<i8>) -> RangeIter<'_, i8> {
        if self.0.is_empty() {
            let (start, end) = range.into_inner();
            RangeIter::Range(start.max(Self::MIN)..=end.min(Self::MAX))
        } else {
            RangeIter::SetRange(self.0.range(range))
        }
    }
}

/// A ranged set of i16 values.
///
/// The `Default` value of this type contains the full range of possible values between `MIN`
/// (inclusive) and `MAX` (inclusive).
#[derive(Debug, Clone, Default)]
pub(super) struct RangedI16Set<const MIN: i16, const MAX: i16>(BTreeSet<i16>);

impl<const MIN: i16, const MAX: i16> RangedI16Set<MIN, MAX> {
    pub(super) const MIN: i16 = MIN;
    pub(super) const MAX: i16 = MAX;

    #[inline]
    fn is_within_bounds(value: i16) -> bool {
        (Self::MIN..=Self::MAX).contains(&value)
    }

    pub(super) fn try_insert(&mut self, value: i16) -> Result<bool, Error> {
        if Self::is_within_bounds(value) {
            Ok(self.0.insert(value))
        } else {
            Err(Error::range(value, Self::MIN, Self::MAX))
        }
    }

    pub(super) fn range(&self, range: RangeInclusive<i16>) -> RangeIter<'_, i16> {
        if self.0.is_empty() {
            let (start, end) = range.into_inner();
            RangeIter::Range(start.max(Self::MIN)..=end.min(Self::MAX))
        } else {
            RangeIter::SetRange(self.0.range(range))
        }
    }
}

pub(super) enum RangeIter<'a, T> {
    Range(RangeInclusive<T>),
    SetRange(btree_set::Range<'a, T>),
}

impl<T: Copy> Iterator for RangeIter<'_, T>
where
    RangeInclusive<T>: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RangeIter::Range(range) => range.next(),
            RangeIter::SetRange(iter) => iter.next().copied(),
        }
    }
}

impl<T: Copy> DoubleEndedIterator for RangeIter<'_, T>
where
    RangeInclusive<T>: DoubleEndedIterator<Item = T>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            RangeIter::Range(range) => range.next_back(),
            RangeIter::SetRange(iter) => iter.next_back().copied(),
        }
    }
}
