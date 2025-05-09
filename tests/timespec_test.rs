mod common;

use common::series_take;
use jiff::civil::{DateTime, date};
use recurring::repeat::spec;
use recurring::{Event, Repeat};

#[test]
fn timespec_daily_at() {
    let start = date(2025, 1, 1).at(12, 0, 0, 0);
    let end = DateTime::MAX;
    let range = start..end;
    let repeat = spec().hours(10..12).minute(30).second(0);

    assert_eq!(
        series_take(start..end, repeat.clone(), 5),
        vec![
            Event::at(date(2025, 1, 2).at(10, 30, 0, 0)),
            Event::at(date(2025, 1, 2).at(11, 30, 0, 0)),
            Event::at(date(2025, 1, 3).at(10, 30, 0, 0)),
            Event::at(date(2025, 1, 3).at(11, 30, 0, 0)),
            Event::at(date(2025, 1, 4).at(10, 30, 0, 0)),
        ]
    );

    assert_eq!(
        repeat.closest_event(date(2025, 1, 2).at(11, 30, 0, 0), &range),
        Some(date(2025, 1, 2).at(11, 30, 0, 0))
    );
}
