use core::ops::RangeBounds;
use jiff::civil::DateTime;
use recurring::{Event, Repeat, Series};

pub fn series_take<B: RangeBounds<DateTime>, R: Repeat>(
    range: B,
    repeat: R,
    take: usize,
) -> Vec<Event> {
    let series = Series::new(range, repeat);
    series.iter().take(take).collect()
}
