#![cfg(feature = "alloc")]

mod common;

use common::{series_take, series_take_rev};
use jiff::civil::{DateTime, date};
use pretty_assertions::assert_eq;
use recurring::pattern::{cron, hourly};
use recurring::{Combine, Event, Pattern};

#[test]
fn combined() {
    let start = date(2025, 1, 1).at(12, 0, 0, 0);
    let end = date(2025, 12, 31).at(23, 59, 59, 0);
    let pattern = cron()
        .hour(10)
        .minute(30)
        .second(0)
        .and(cron().hour(12).minute(45).second(30))
        .and(hourly(6));

    assert_eq!(
        series_take(start..end, pattern.clone(), 10),
        vec![
            Event::at(date(2025, 1, 1).at(12, 0, 0, 0)),
            Event::at(date(2025, 1, 1).at(12, 45, 30, 0)),
            Event::at(date(2025, 1, 1).at(18, 0, 0, 0)),
            Event::at(date(2025, 1, 2).at(0, 0, 0, 0)),
            Event::at(date(2025, 1, 2).at(6, 0, 0, 0)),
            Event::at(date(2025, 1, 2).at(10, 30, 0, 0)),
            Event::at(date(2025, 1, 2).at(12, 0, 0, 0)),
            Event::at(date(2025, 1, 2).at(12, 45, 30, 0)),
            Event::at(date(2025, 1, 2).at(18, 0, 0, 0)),
            Event::at(date(2025, 1, 3).at(0, 0, 0, 0)),
        ]
    );

    assert_eq!(
        series_take_rev(start..end, pattern, 10),
        vec![
            Event::at(date(2025, 12, 31).at(18, 0, 0, 0)),
            Event::at(date(2025, 12, 31).at(12, 45, 30, 0)),
            Event::at(date(2025, 12, 31).at(12, 0, 0, 0)),
            Event::at(date(2025, 12, 31).at(10, 30, 0, 0)),
            Event::at(date(2025, 12, 31).at(6, 0, 0, 0)),
            Event::at(date(2025, 12, 31).at(0, 0, 0, 0)),
            Event::at(date(2025, 12, 30).at(18, 0, 0, 0)),
            Event::at(date(2025, 12, 30).at(12, 45, 30, 0)),
            Event::at(date(2025, 12, 30).at(12, 0, 0, 0)),
            Event::at(date(2025, 12, 30).at(10, 30, 0, 0)),
        ]
    );
}

#[test]
fn combined_closest_to() {
    let start = date(2025, 1, 1).at(12, 0, 0, 0);
    let end = date(2025, 12, 31).at(12, 0, 0, 0);
    let range = start..end;
    let pattern = cron()
        .hour(10)
        .minute(30)
        .second(0)
        .and(cron().hour(12).minute(45).second(30))
        .and(hourly(6));

    assert_eq!(
        pattern.closest_to(date(2024, 12, 31).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 1).at(12, 0, 0, 0)),
    );
    assert_eq!(
        pattern.closest_to(date(2025, 1, 1).at(12, 0, 0, 0), &range),
        Some(date(2025, 1, 1).at(12, 0, 0, 0)),
    );
    assert_eq!(
        pattern.closest_to(date(2025, 1, 2).at(8, 14, 59, 0), &range),
        Some(date(2025, 1, 2).at(6, 0, 0, 0)),
    );
    assert_eq!(
        pattern.closest_to(date(2025, 1, 2).at(8, 15, 0, 0), &range),
        Some(date(2025, 1, 2).at(6, 0, 0, 0)),
    );
    assert_eq!(
        pattern.closest_to(date(2025, 1, 2).at(8, 15, 0, 1), &range),
        Some(date(2025, 1, 2).at(10, 30, 0, 0)),
    );
    assert_eq!(
        pattern.closest_to(date(2025, 1, 2).at(8, 15, 1, 0), &range),
        Some(date(2025, 1, 2).at(10, 30, 0, 0)),
    );
    assert_eq!(
        pattern.closest_to(DateTime::MAX, &range),
        Some(date(2025, 12, 31).at(10, 30, 0, 0)),
    );
}
