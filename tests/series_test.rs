mod common;

use common::{series_full, series_take};
use jiff::{
    ToSpan,
    civil::{DateTime, date, datetime, time},
};
use recurring::pattern::{daily, hourly, yearly};
use recurring::{Combine, Event, Series};

#[test]
fn series_range() {
    let series = Series::new(.., daily(1));
    assert_eq!(series.start(), DateTime::MIN);
    assert_eq!(series.end(), DateTime::MAX);

    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 2, 1).at(0, 0, 0, 0);

    let series = Series::new(start..end, daily(1));
    assert_eq!(series.start(), start);
    assert_eq!(series.end(), end);

    let series = Series::new(..=end, daily(1));
    assert_eq!(series.start(), DateTime::MIN);
    assert_eq!(series.end(), end + 1.nanosecond());
}

#[test]
fn series_iter() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 1, 2).at(0, 0, 0, 0);

    let series = Series::new(start..end, hourly(6));
    let mut iter = series.iter();

    assert_eq!(iter.next(), Some(Event::at(start)));
    assert_eq!(iter.next_back(), Some(Event::at(end - 6.hours())));
    assert_eq!(iter.next(), Some(Event::at(start + 6.hours())));
    assert_eq!(iter.next(), Some(Event::at(start + 12.hours())));
    assert_eq!(iter.next_back(), Some(Event::at(end - 12.hours())));
    assert_eq!(iter.next(), Some(Event::at(end - 6.hours())));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), Some(Event::at(start + 6.hours())));
    assert_eq!(iter.next_back(), Some(Event::at(start)));
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn series_daily() {
    let start = datetime(2025, 1, 1, 1, 1, 1, 0);

    assert_eq!(
        series_take(start.., daily(2), 5),
        vec![
            Event::at(datetime(2025, 1, 1, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 3, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 5, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 7, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 9, 1, 1, 1, 0)),
        ]
    );
}

#[test]
fn series_daily_at() {
    let start = datetime(2025, 1, 1, 1, 1, 1, 0);

    assert_eq!(
        series_take(
            start..,
            daily(2)
                .at(time(2, 2, 2, 2))
                .and(daily(2).at(time(3, 3, 3, 3))),
            5
        ),
        vec![
            Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)),
            Event::at(datetime(2025, 1, 1, 3, 3, 3, 3)),
            Event::at(datetime(2025, 1, 3, 2, 2, 2, 2)),
            Event::at(datetime(2025, 1, 3, 3, 3, 3, 3)),
            Event::at(datetime(2025, 1, 5, 2, 2, 2, 2)),
        ]
    );
}

#[test]
fn series_daily_with_end() {
    let start = datetime(2025, 1, 1, 1, 1, 1, 0);
    let end = datetime(2025, 1, 5, 1, 1, 1, 0);

    assert_eq!(
        series_full(start..end, daily(2)),
        vec![
            Event::at(datetime(2025, 1, 1, 1, 1, 1, 0)),
            Event::at(datetime(2025, 1, 3, 1, 1, 1, 0)),
        ]
    );
}

#[test]
fn series_daily_with_end_and_duration() {
    let start = datetime(2025, 1, 1, 1, 1, 1, 0);
    let end = datetime(2025, 1, 5, 1, 1, 1, 0);
    let series = Series::new(start..end, daily(2))
        .with()
        .event_duration(1.hour())
        .build()
        .unwrap();

    let events: Vec<_> = series.iter().collect();
    let expected = vec![
        Event::new(
            datetime(2025, 1, 1, 1, 1, 1, 0),
            datetime(2025, 1, 1, 2, 1, 1, 0),
        ),
        Event::new(
            datetime(2025, 1, 3, 1, 1, 1, 0),
            datetime(2025, 1, 3, 2, 1, 1, 0),
        ),
    ];
    assert_eq!(events, expected);
}

#[test]
fn series_contains_event() {
    let start = datetime(2025, 1, 1, 1, 1, 1, 0);
    let series = Series::new(
        start..,
        daily(2)
            .at(time(2, 2, 2, 2))
            .and(daily(2).at(time(3, 3, 3, 3))),
    );
    assert!(!series.contains_event(datetime(2025, 1, 1, 1, 1, 1, 0)));
    assert!(series.contains_event(datetime(2025, 1, 1, 2, 2, 2, 2)));
    assert!(series.contains_event(datetime(2025, 1, 1, 3, 3, 3, 3)));
    assert!(!series.contains_event(datetime(2025, 1, 1, 2, 2, 2, 3)));
    assert!(!series.contains_event(datetime(2025, 1, 1, 3, 3, 3, 2)));
}

#[test]
fn series_relative_events() {
    let start = datetime(2025, 1, 1, 1, 1, 1, 0);
    let end = datetime(2025, 1, 3, 1, 1, 1, 0);
    let series = Series::new(
        start..end,
        daily(2)
            .at(time(2, 2, 2, 2))
            .and(daily(2).at(time(3, 3, 3, 3))),
    );
    assert_eq!(series.get_event(datetime(2025, 1, 1, 1, 1, 1, 0)), None);
    assert_eq!(
        series.get_event(datetime(2025, 1, 1, 2, 2, 2, 2)),
        Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)))
    );

    assert_eq!(
        series.get_event_after(datetime(2025, 1, 1, 2, 2, 2, 2)),
        Some(Event::at(datetime(2025, 1, 1, 3, 3, 3, 3)))
    );

    assert_eq!(
        series.get_event_before(datetime(2025, 1, 1, 2, 2, 2, 2)),
        None
    );

    assert_eq!(
        series.get_event_before(datetime(2025, 1, 1, 3, 3, 3, 3)),
        Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2))),
    );
}

#[test]
fn series_get_event_containing() {
    let start = datetime(2025, 1, 1, 1, 1, 1, 0);
    let end = datetime(2025, 1, 3, 1, 1, 1, 0);
    let series = Series::new(start..end, daily(2).at(time(2, 2, 2, 2)));
    assert_eq!(
        series.get_event_containing(datetime(2025, 1, 1, 1, 1, 1, 0)),
        None
    );
    assert_eq!(
        series.get_event_containing(datetime(2025, 1, 1, 2, 2, 2, 2)),
        Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)))
    );
    assert_eq!(
        series.get_event_containing(datetime(2025, 1, 1, 2, 2, 2, 3)),
        None
    );

    let series = Series::new(start..end, daily(2).at(time(2, 2, 2, 2)))
        .with()
        .event_duration(1.hour())
        .build()
        .unwrap();

    assert_eq!(
        series.get_event_containing(datetime(2025, 1, 1, 2, 2, 2, 3)),
        Some(Event::new(
            datetime(2025, 1, 1, 2, 2, 2, 2),
            datetime(2025, 1, 1, 3, 2, 2, 2)
        ))
    );

    assert_eq!(
        series.get_event_containing(datetime(2025, 1, 1, 3, 2, 2, 1)),
        Some(Event::new(
            datetime(2025, 1, 1, 2, 2, 2, 2),
            datetime(2025, 1, 1, 3, 2, 2, 2)
        ))
    );

    assert_eq!(
        series.get_event_containing(datetime(2025, 1, 1, 3, 2, 2, 2)),
        None
    );
}

#[test]
fn series_first_event() {
    let start = datetime(2025, 1, 1, 1, 1, 1, 0);
    let end = datetime(2025, 1, 3, 1, 1, 1, 0);
    let series = Series::new(start..end, daily(2).at(time(2, 2, 2, 2)));
    assert_eq!(
        series.first_event(),
        Some(Event::at(datetime(2025, 1, 1, 2, 2, 2, 2)))
    );
}

#[test]
fn series_last_event() {
    let start = datetime(2025, 1, 1, 1, 1, 1, 0);
    let end = datetime(2025, 1, 10, 1, 1, 1, 0);
    let series = Series::new(start..end, daily(2).at(time(2, 2, 2, 2)));
    assert_eq!(
        series.last_event(),
        Some(Event::at(datetime(2025, 1, 9, 2, 2, 2, 2)))
    );

    let series = Series::new(start.., daily(2).at(time(2, 2, 2, 2)));
    assert_eq!(
        series.last_event(),
        Some(Event::at(datetime(9999, 12, 30, 2, 2, 2, 2)))
    );
}

#[test]
fn series_last_event_unbounded() {
    let start = date(2025, 1, 1).at(1, 1, 1, 0);
    let series = Series::new(start.., hourly(2));
    assert_eq!(
        series.last_event(),
        Some(Event::at(date(9999, 12, 31).at(23, 1, 1, 0)))
    );
}

#[test]
fn series_get_closest_event() {
    let start = datetime(2025, 1, 1, 0, 0, 0, 0);
    let series = Series::new(start.., hourly(1));

    assert_eq!(
        series.get_closest_event(datetime(2024, 12, 31, 0, 0, 0, 0)),
        Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
    );
    assert_eq!(
        series.get_closest_event(datetime(2025, 1, 1, 0, 0, 0, 0)),
        Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
    );
    assert_eq!(
        series.get_closest_event(datetime(2025, 1, 1, 0, 29, 0, 999)),
        Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
    );
    assert_eq!(
        series.get_closest_event(datetime(2025, 1, 1, 0, 30, 0, 0)),
        Some(Event::at(datetime(2025, 1, 1, 1, 0, 0, 0)))
    );
    assert_eq!(
        series.get_closest_event(DateTime::MIN),
        Some(Event::at(datetime(2025, 1, 1, 0, 0, 0, 0)))
    );
    assert_eq!(
        series.get_closest_event(DateTime::MAX),
        Some(Event::at(datetime(9999, 12, 31, 23, 0, 0, 0)))
    );
}

#[test]
fn series_overlapping_last_event() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 2, 1).at(0, 0, 0, 0);
    let series = Series::new(start..end, daily(1))
        .with()
        .event_duration(50.hours())
        .build()
        .unwrap();

    assert_eq!(series.end(), date(2025, 1, 29).at(22, 0, 0, 0));
    assert_eq!(
        series.last_event(),
        Some(Event::new(
            date(2025, 1, 29).at(0, 0, 0, 0),
            date(2025, 1, 31).at(2, 0, 0, 0)
        ))
    );
}

#[test]
fn series_event_durations() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 2, 1).at(0, 0, 0, 0);
    let series = Series::new(start..end, daily(1));

    assert!(
        series
            .clone()
            .with()
            .event_duration(30.days().hours(23).minutes(59).seconds(59).nanoseconds(999))
            .build()
            .is_ok()
    );

    assert!(
        series
            .clone()
            .with()
            .event_duration(1.month())
            .build()
            .is_err()
    );

    assert!(
        series
            .clone()
            .with()
            .event_duration(1.year())
            .build()
            .is_err()
    );
}

#[test]
fn series_leap_year() {
    let start = date(2024, 1, 1).at(0, 0, 0, 0);

    assert_eq!(
        series_take(start.., yearly(1), 5),
        vec![
            Event::at(date(2024, 1, 1).at(0, 0, 0, 0)),
            Event::at(date(2025, 1, 1).at(0, 0, 0, 0)),
            Event::at(date(2026, 1, 1).at(0, 0, 0, 0)),
            Event::at(date(2027, 1, 1).at(0, 0, 0, 0)),
            Event::at(date(2028, 1, 1).at(0, 0, 0, 0)),
        ]
    );
}
