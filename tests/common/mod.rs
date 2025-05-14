#![allow(dead_code)]

use core::ops::RangeBounds;
use jiff::civil::DateTime;
use recurring::{Event, Pattern, Series};

pub fn series_full<B: RangeBounds<DateTime>, P: Pattern>(range: B, pattern: P) -> Vec<Event> {
    let series = Series::new(range, pattern);
    series.iter().collect()
}

pub fn series_take<B: RangeBounds<DateTime>, P: Pattern>(
    range: B,
    pattern: P,
    take: usize,
) -> Vec<Event> {
    let series = Series::new(range, pattern);
    series.iter().take(take).collect()
}

pub fn series_take_rev<B: RangeBounds<DateTime>, P: Pattern>(
    range: B,
    pattern: P,
    take: usize,
) -> Vec<Event> {
    let series = Series::new(range, pattern);
    series.iter().rev().take(take).collect()
}
