use alloc::collections::btree_set::{self, BTreeSet};
use core::ops::RangeInclusive;

pub type Weekdays = I8TimeUnit<1, 7>;
pub type Months = I8TimeUnit<1, 12>;
pub type Days = I8TimeUnit<1, 31>;
pub type Hours = I8TimeUnit<0, 23>;
pub type Minutes = I8TimeUnit<0, 59>;
pub type Seconds = I8TimeUnit<0, 59>;

#[derive(Debug, Clone, Default)]
pub struct I8TimeUnit<const MIN: i8, const MAX: i8> {
    set: BTreeSet<i8>,
}

impl<const MIN: i8, const MAX: i8> I8TimeUnit<MIN, MAX> {
    pub const MIN: i8 = MIN;
    pub const MAX: i8 = MAX;

    pub fn insert(&mut self, value: i8) -> bool {
        assert!(Self::all().contains(&value));
        self.set.insert(value)
    }

    #[inline]
    pub const fn all() -> RangeInclusive<i8> {
        Self::MIN..=Self::MAX
    }

    pub fn contains(&self, second: i8) -> bool {
        if self.set.is_empty() {
            Self::all().contains(&second)
        } else {
            self.set.contains(&second)
        }
    }

    pub fn range(&self, range: RangeInclusive<i8>) -> I8RangeIter<'_> {
        if self.set.is_empty() {
            I8RangeIter::Range(range)
        } else {
            I8RangeIter::Set(self.set.range(range))
        }
    }
}

pub enum I8RangeIter<'a> {
    Range(RangeInclusive<i8>),
    Set(btree_set::Range<'a, i8>),
}

impl Iterator for I8RangeIter<'_> {
    type Item = i8;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            I8RangeIter::Range(range) => range.next(),
            I8RangeIter::Set(iter) => iter.next().copied(),
        }
    }
}

impl DoubleEndedIterator for I8RangeIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            I8RangeIter::Range(range) => range.next_back(),
            I8RangeIter::Set(iter) => iter.next_back().copied(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Years {
    set: BTreeSet<i16>,
}

impl Years {
    pub const MIN: i16 = -9999;
    pub const MAX: i16 = 9999;

    pub fn insert(&mut self, value: i16) -> bool {
        assert!(Self::all().contains(&value));
        self.set.insert(value)
    }

    #[inline]
    pub const fn all() -> RangeInclusive<i16> {
        Self::MIN..=Self::MAX
    }

    pub fn range(&self, range: RangeInclusive<i16>) -> YearRangeIter<'_> {
        if self.set.is_empty() {
            YearRangeIter::Range(range)
        } else {
            YearRangeIter::Set(self.set.range(range))
        }
    }
}

pub enum YearRangeIter<'a> {
    Range(RangeInclusive<i16>),
    Set(btree_set::Range<'a, i16>),
}

impl Iterator for YearRangeIter<'_> {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            YearRangeIter::Range(range) => range.next(),
            YearRangeIter::Set(iter) => iter.next().copied(),
        }
    }
}

impl DoubleEndedIterator for YearRangeIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            YearRangeIter::Range(range) => range.next_back(),
            YearRangeIter::Set(iter) => iter.next_back().copied(),
        }
    }
}
