#![cfg(feature = "alloc")]

mod common;

use common::{series_take, series_take_rev};
use jiff::civil::{DateTime, Weekday, date};
use pretty_assertions::assert_eq;
use recurring::pattern::{Cron, cron};
use recurring::{Event, Pattern};

#[test]
fn cron_default() {
    let start = date(2025, 1, 1).at(12, 0, 0, 0);

    assert_eq!(
        series_take(start.., cron(), 5),
        vec![
            Event::at(date(2025, 1, 1).at(12, 0, 0, 0)),
            Event::at(date(2025, 1, 1).at(12, 0, 1, 0)),
            Event::at(date(2025, 1, 1).at(12, 0, 2, 0)),
            Event::at(date(2025, 1, 1).at(12, 0, 3, 0)),
            Event::at(date(2025, 1, 1).at(12, 0, 4, 0)),
        ]
    );

    assert_eq!(
        series_take_rev(start.., cron(), 5),
        vec![
            Event::at(date(9999, 12, 31).at(23, 59, 59, 0)),
            Event::at(date(9999, 12, 31).at(23, 59, 58, 0)),
            Event::at(date(9999, 12, 31).at(23, 59, 57, 0)),
            Event::at(date(9999, 12, 31).at(23, 59, 56, 0)),
            Event::at(date(9999, 12, 31).at(23, 59, 55, 0)),
        ]
    );
}

#[test]
fn cron_daily_at() {
    let start = date(2025, 1, 1).at(12, 0, 0, 0);
    let end = DateTime::MAX;
    let pattern = cron().hours(10..12).minute(30).second(0);

    assert_eq!(
        series_take(start..end, pattern.clone(), 5),
        vec![
            Event::at(date(2025, 1, 2).at(10, 30, 0, 0)),
            Event::at(date(2025, 1, 2).at(11, 30, 0, 0)),
            Event::at(date(2025, 1, 3).at(10, 30, 0, 0)),
            Event::at(date(2025, 1, 3).at(11, 30, 0, 0)),
            Event::at(date(2025, 1, 4).at(10, 30, 0, 0)),
        ]
    );

    assert_eq!(
        series_take_rev(start..end, pattern, 5),
        vec![
            Event::at(date(9999, 12, 31).at(11, 30, 0, 0)),
            Event::at(date(9999, 12, 31).at(10, 30, 0, 0)),
            Event::at(date(9999, 12, 30).at(11, 30, 0, 0)),
            Event::at(date(9999, 12, 30).at(10, 30, 0, 0)),
            Event::at(date(9999, 12, 29).at(11, 30, 0, 0)),
        ]
    );
}

#[test]
fn cron_weekdays() {
    let start = date(2025, 5, 11).at(12, 0, 0, 0);
    let pattern = cron()
        .weekdays([Weekday::Monday, Weekday::Thursday])
        .hour(12)
        .minute(0)
        .second(0);

    assert_eq!(
        series_take(start.., pattern.clone(), 5),
        vec![
            Event::at(date(2025, 5, 12).at(12, 0, 0, 0)),
            Event::at(date(2025, 5, 15).at(12, 0, 0, 0)),
            Event::at(date(2025, 5, 19).at(12, 0, 0, 0)),
            Event::at(date(2025, 5, 22).at(12, 0, 0, 0)),
            Event::at(date(2025, 5, 26).at(12, 0, 0, 0)),
        ]
    );

    assert_eq!(
        series_take_rev(start.., pattern, 5),
        vec![
            Event::at(date(9999, 12, 30).at(12, 0, 0, 0)),
            Event::at(date(9999, 12, 27).at(12, 0, 0, 0)),
            Event::at(date(9999, 12, 23).at(12, 0, 0, 0)),
            Event::at(date(9999, 12, 20).at(12, 0, 0, 0)),
            Event::at(date(9999, 12, 16).at(12, 0, 0, 0)),
        ]
    );
}

#[test]
fn cron_next_after() {
    let range = DateTime::MIN..DateTime::MAX;
    let pattern = Cron::new().minute(3).second(5).second(10);

    assert_eq!(
        pattern.next_after(date(2025, 1, 1).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 3, 5, 0))
    );

    assert_eq!(
        pattern.next_after(date(2025, 1, 1).at(0, 3, 5, 0), &range),
        Some(date(2025, 1, 1).at(0, 3, 10, 0))
    );

    assert_eq!(
        pattern.next_after(date(2025, 1, 1).at(0, 3, 10, 0), &range),
        Some(date(2025, 1, 1).at(1, 3, 5, 0))
    );
}

#[test]
fn cron_previous_before() {
    let range = DateTime::MIN..DateTime::MAX;
    let pattern = Cron::new().minute(3).seconds([5, 10]);

    assert_eq!(
        pattern.previous_before(date(2025, 1, 1).at(0, 3, 5, 0), &range),
        Some(date(2024, 12, 31).at(23, 3, 10, 0))
    );

    assert_eq!(
        pattern.previous_before(date(2024, 12, 31).at(23, 3, 10, 0), &range),
        Some(date(2024, 12, 31).at(23, 3, 5, 0))
    );

    assert_eq!(
        pattern.previous_before(date(2024, 12, 31).at(23, 3, 5, 0), &range),
        Some(date(2024, 12, 31).at(22, 3, 10, 0))
    );
}

#[test]
fn cron_closest_to() {
    let range = DateTime::MIN..DateTime::MAX;
    let pattern = Cron::new().hour(1).minute(30).second(0);

    assert_eq!(
        pattern.closest_to(date(2025, 1, 1).at(1, 29, 59, 0), &range),
        Some(date(2025, 1, 1).at(1, 30, 0, 0))
    );

    assert_eq!(
        pattern.closest_to(date(2025, 1, 1).at(1, 30, 1, 0), &range),
        Some(date(2025, 1, 1).at(1, 30, 0, 0))
    );

    assert_eq!(
        pattern.closest_to(date(2025, 1, 1).at(14, 0, 0, 0), &range),
        Some(date(2025, 1, 2).at(1, 30, 0, 0))
    );

    assert_eq!(
        pattern.closest_to(date(2025, 1, 1).at(1, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(1, 30, 0, 0))
    );
}

#[test]
fn cron_closest_to_datetime_max() {
    let range = DateTime::MIN..DateTime::MAX;
    let pattern = Cron::new().hour(1).minute(30).second(0);

    assert_eq!(
        pattern.closest_to(DateTime::MAX, &range),
        Some(date(9999, 12, 31).at(1, 30, 0, 0))
    );
}

#[test]
fn cron_step_by() {
    let start = date(2025, 5, 11).at(12, 0, 0, 0);
    let pattern = cron().hour_step_by(12, 4).minute_step_by(0, 30).second(0);

    assert_eq!(
        series_take(start.., pattern.clone(), 10),
        vec![
            Event::at(date(2025, 5, 11).at(12, 0, 0, 0)),
            Event::at(date(2025, 5, 11).at(12, 30, 0, 0)),
            Event::at(date(2025, 5, 11).at(16, 0, 0, 0)),
            Event::at(date(2025, 5, 11).at(16, 30, 0, 0)),
            Event::at(date(2025, 5, 11).at(20, 0, 0, 0)),
            Event::at(date(2025, 5, 11).at(20, 30, 0, 0)),
            Event::at(date(2025, 5, 12).at(12, 0, 0, 0)),
            Event::at(date(2025, 5, 12).at(12, 30, 0, 0)),
            Event::at(date(2025, 5, 12).at(16, 0, 0, 0)),
            Event::at(date(2025, 5, 12).at(16, 30, 0, 0)),
        ]
    );

    assert_eq!(
        series_take_rev(start.., pattern, 10),
        vec![
            Event::at(date(9999, 12, 31).at(20, 30, 0, 0)),
            Event::at(date(9999, 12, 31).at(20, 0, 0, 0)),
            Event::at(date(9999, 12, 31).at(16, 30, 0, 0)),
            Event::at(date(9999, 12, 31).at(16, 0, 0, 0)),
            Event::at(date(9999, 12, 31).at(12, 30, 0, 0)),
            Event::at(date(9999, 12, 31).at(12, 0, 0, 0)),
            Event::at(date(9999, 12, 30).at(20, 30, 0, 0)),
            Event::at(date(9999, 12, 30).at(20, 0, 0, 0)),
            Event::at(date(9999, 12, 30).at(16, 30, 0, 0)),
            Event::at(date(9999, 12, 30).at(16, 0, 0, 0)),
        ]
    );
}
