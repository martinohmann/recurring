mod common;

use common::{series_take, series_take_rev};
use jiff::civil::{DateTime, Weekday, date};
use recurring::repeat::{TimeSpec, spec};
use recurring::{Event, Repeat};

#[test]
fn timespec_default() {
    let start = date(2025, 1, 1).at(12, 0, 0, 0);

    assert_eq!(
        series_take(start.., spec(), 5),
        vec![
            Event::at(date(2025, 1, 1).at(12, 0, 0, 0)),
            Event::at(date(2025, 1, 1).at(12, 0, 1, 0)),
            Event::at(date(2025, 1, 1).at(12, 0, 2, 0)),
            Event::at(date(2025, 1, 1).at(12, 0, 3, 0)),
            Event::at(date(2025, 1, 1).at(12, 0, 4, 0)),
        ]
    );

    assert_eq!(
        series_take_rev(start.., spec(), 5),
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
fn timespec_daily_at() {
    let start = date(2025, 1, 1).at(12, 0, 0, 0);
    let end = DateTime::MAX;
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
        series_take_rev(start..end, repeat, 5),
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
fn timespec_weekdays() {
    let start = date(2025, 5, 11).at(12, 0, 0, 0);
    let repeat = spec()
        .weekdays([Weekday::Monday, Weekday::Thursday])
        .hour(12)
        .minute(0)
        .second(0);

    assert_eq!(
        series_take(start.., repeat.clone(), 5),
        vec![
            Event::at(date(2025, 5, 12).at(12, 0, 0, 0)),
            Event::at(date(2025, 5, 15).at(12, 0, 0, 0)),
            Event::at(date(2025, 5, 19).at(12, 0, 0, 0)),
            Event::at(date(2025, 5, 22).at(12, 0, 0, 0)),
            Event::at(date(2025, 5, 26).at(12, 0, 0, 0)),
        ]
    );

    assert_eq!(
        series_take_rev(start.., repeat, 5),
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
fn timespec_next_after() {
    let range = DateTime::MIN..DateTime::MAX;
    let ts = TimeSpec::new().minute(3).second(5).second(10);

    assert_eq!(
        ts.next_after(date(2025, 1, 1).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 3, 5, 0))
    );

    assert_eq!(
        ts.next_after(date(2025, 1, 1).at(0, 3, 5, 0), &range),
        Some(date(2025, 1, 1).at(0, 3, 10, 0))
    );

    assert_eq!(
        ts.next_after(date(2025, 1, 1).at(0, 3, 10, 0), &range),
        Some(date(2025, 1, 1).at(1, 3, 5, 0))
    );
}

#[test]
fn timespec_previous_before() {
    let range = DateTime::MIN..DateTime::MAX;
    let ts = TimeSpec::new().minute(3).seconds([5, 10]);

    assert_eq!(
        ts.previous_before(date(2025, 1, 1).at(0, 3, 5, 0), &range),
        Some(date(2024, 12, 31).at(23, 3, 10, 0))
    );

    assert_eq!(
        ts.previous_before(date(2024, 12, 31).at(23, 3, 10, 0), &range),
        Some(date(2024, 12, 31).at(23, 3, 5, 0))
    );

    assert_eq!(
        ts.previous_before(date(2024, 12, 31).at(23, 3, 5, 0), &range),
        Some(date(2024, 12, 31).at(22, 3, 10, 0))
    );
}

#[test]
fn timespec_closest_to() {
    let range = DateTime::MIN..DateTime::MAX;
    let ts = TimeSpec::new().hour(1).minute(30).second(0);

    assert_eq!(
        ts.closest_to(date(2025, 1, 1).at(1, 29, 59, 0), &range),
        Some(date(2025, 1, 1).at(1, 30, 0, 0))
    );

    assert_eq!(
        ts.closest_to(date(2025, 1, 1).at(1, 30, 1, 0), &range),
        Some(date(2025, 1, 1).at(1, 30, 0, 0))
    );

    assert_eq!(
        ts.closest_to(date(2025, 1, 1).at(14, 0, 0, 0), &range),
        Some(date(2025, 1, 2).at(1, 30, 0, 0))
    );

    assert_eq!(
        ts.closest_to(date(2025, 1, 1).at(1, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(1, 30, 0, 0))
    );
}

#[test]
fn timespec_closest_to_datetime_max() {
    let range = DateTime::MIN..DateTime::MAX;
    let ts = TimeSpec::new().hour(1).minute(30).second(0);

    assert_eq!(
        ts.closest_to(DateTime::MAX, &range),
        Some(date(9999, 12, 31).at(1, 30, 0, 0))
    );
}
