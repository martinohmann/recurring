use jiff::ToSpan;
use jiff::civil::{DateTime, date};
use recurring::Pattern;
use recurring::pattern::Interval;

#[test]
fn interval_next_after() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 1, 3).at(0, 0, 0, 0);
    let range = start..end;
    let interval = Interval::new(1.hour());
    assert_eq!(
        interval.next_after(DateTime::MIN, &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );
    assert_eq!(
        interval.next_after(range.start.checked_sub(1.nanosecond()).unwrap(), &range,),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        interval.next_after(range.start, &range),
        Some(date(2025, 1, 1).at(1, 0, 0, 0))
    );

    assert_eq!(
        interval.next_after(date(2025, 1, 1).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(1, 0, 0, 0))
    );

    assert_eq!(
        interval.next_after(range.end - 1.hour().nanoseconds(1), &range),
        Some(date(2025, 1, 2).at(23, 0, 0, 0))
    );
    assert_eq!(interval.next_after(range.end - 1.hour(), &range), None,);
    assert_eq!(interval.next_after(range.end, &range), None);
    assert_eq!(interval.next_after(DateTime::MAX, &range), None);
}

#[test]
fn interval_offset_next_after() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 1, 3).at(0, 0, 0, 0);
    let range = start..end;
    let interval = Interval::new(1.hour()).offset(10.minutes());
    assert_eq!(
        interval.next_after(DateTime::MIN, &range),
        Some(date(2025, 1, 1).at(0, 10, 0, 0))
    );
    assert_eq!(
        interval.next_after(range.start.checked_sub(1.nanosecond()).unwrap(), &range,),
        Some(date(2025, 1, 1).at(0, 10, 0, 0))
    );

    assert_eq!(
        interval.next_after(range.start, &range),
        Some(date(2025, 1, 1).at(0, 10, 0, 0))
    );

    assert_eq!(
        interval.next_after(date(2025, 1, 1).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(1, 10, 0, 0))
    );

    assert_eq!(
        interval.next_after(range.end - 1.hour(), &range),
        Some(date(2025, 1, 2).at(23, 10, 0, 0))
    );
    assert_eq!(interval.next_after(range.end - 50.minutes(), &range), None);
    assert_eq!(interval.next_after(range.end, &range), None);
    assert_eq!(interval.next_after(DateTime::MAX, &range), None);

    let interval = Interval::new(1.hour()).offset(2.hours().minutes(30));

    assert_eq!(
        interval.next_after(DateTime::MIN, &range),
        Some(date(2025, 1, 1).at(2, 30, 0, 0))
    );
    assert_eq!(
        interval.next_after(range.end - 30.minutes().nanoseconds(1), &range),
        Some(date(2025, 1, 2).at(23, 30, 0, 0))
    );
    assert_eq!(interval.next_after(range.end - 30.minutes(), &range), None);

    let interval = Interval::new(1.hour()).offset(3.days());

    assert_eq!(interval.next_after(DateTime::MIN, &range), None);
}

#[test]
fn interval_previous_before() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 1, 3).at(0, 0, 0, 0);
    let range = start..end;
    let interval = Interval::new(1.hour());
    assert_eq!(interval.previous_before(DateTime::MIN, &range), None);
    assert_eq!(interval.previous_before(range.start, &range), None);
    assert_eq!(
        interval.previous_before(range.start + 1.second(), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        interval.previous_before(date(2025, 1, 1).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        interval.previous_before(date(2025, 1, 3).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 2).at(23, 0, 0, 0))
    );

    assert_eq!(
        interval.previous_before(range.end, &range),
        Some(date(2025, 1, 2).at(23, 0, 0, 0))
    );
}

#[test]
fn interval_offset_previous_before() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 1, 3).at(0, 0, 0, 0);
    let range = start..end;
    let interval = Interval::new(1.hour()).offset(30.minutes());
    assert_eq!(interval.previous_before(DateTime::MIN, &range), None);
    assert_eq!(interval.previous_before(range.start, &range), None);
    assert_eq!(
        interval.previous_before(range.start + 30.minutes(), &range),
        None
    );
    assert_eq!(
        interval.previous_before(range.start + 30.minutes().seconds(1), &range),
        Some(date(2025, 1, 1).at(0, 30, 0, 0))
    );

    assert_eq!(
        interval.previous_before(date(2025, 1, 1).at(0, 31, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 30, 0, 0))
    );

    assert_eq!(
        interval.previous_before(date(2025, 1, 3).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 2).at(23, 30, 0, 0))
    );

    assert_eq!(
        interval.previous_before(range.end, &range),
        Some(date(2025, 1, 2).at(23, 30, 0, 0))
    );
}

#[test]
fn interval_closest_to() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 1, 3).at(0, 0, 0, 0);
    let range = start..end;
    let interval = Interval::new(1.hour());

    assert_eq!(
        interval.closest_to(date(2025, 1, 1).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        interval.closest_to(date(2025, 1, 1).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(1, 0, 0, 0))
    );

    assert_eq!(
        interval.closest_to(
            date(2025, 1, 1)
                .at(0, 30, 0, 0)
                .checked_sub(1.nanosecond())
                .unwrap(),
            &range,
        ),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        interval.closest_to(date(2024, 12, 31).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        interval.closest_to(date(2025, 1, 3).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 2).at(23, 0, 0, 0))
    );

    assert_eq!(
        interval.closest_to(date(2025, 2, 10).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 2).at(23, 0, 0, 0))
    );
}

#[test]
fn interval_offset_closest_to() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let range = start..DateTime::MAX;
    let interval = Interval::new(1.hour()).offset(5.minutes());

    assert_eq!(
        interval.closest_to(date(2025, 1, 1).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 5, 0, 0))
    );
    assert_eq!(
        interval.closest_to(date(2025, 1, 1).at(0, 35, 0, 0), &range),
        Some(date(2025, 1, 1).at(1, 5, 0, 0))
    );
}

#[test]
fn interval_closest_to_datetime_max() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let range = start..DateTime::MAX;
    let interval = Interval::new(2.hours());

    assert_eq!(
        interval.closest_to(DateTime::MAX, &range),
        Some(date(9999, 12, 31).at(22, 0, 0, 0))
    );
}
