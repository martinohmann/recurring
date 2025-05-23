use jiff::ToSpan;
use jiff::civil::{DateTime, date, time};
use recurring::Pattern;
use recurring::pattern::Daily;

#[test]
fn daily_next_after() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 2, 1).at(0, 0, 0, 0);
    let range = start..end;
    let daily = Daily::new(2);
    assert_eq!(
        daily.next_after(DateTime::MIN, &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );
    assert_eq!(
        daily.next_after(range.start.checked_sub(1.nanosecond()).unwrap(), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        daily.next_after(range.start, &range),
        Some(date(2025, 1, 3).at(0, 0, 0, 0))
    );

    assert_eq!(
        daily.next_after(date(2025, 1, 1).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 3).at(0, 0, 0, 0))
    );

    assert_eq!(
        daily.next_after(range.end - 1.day().nanoseconds(1), &range),
        Some(date(2025, 1, 31).at(0, 0, 0, 0))
    );
    assert_eq!(daily.next_after(range.end - 1.day(), &range), None);
    assert_eq!(daily.next_after(range.end, &range), None);
    assert_eq!(daily.next_after(DateTime::MAX, &range), None);
}

#[test]
fn daily_at_next_after() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 2, 1).at(0, 0, 0, 0);
    let range = start..end;
    let daily = Daily::new(2).at(time(12, 0, 0, 0));
    assert_eq!(
        daily.next_after(DateTime::MIN, &range),
        Some(date(2025, 1, 1).at(12, 0, 0, 0))
    );
    assert_eq!(
        daily.next_after(range.start.checked_sub(1.nanosecond()).unwrap(), &range),
        Some(date(2025, 1, 1).at(12, 0, 0, 0))
    );

    assert_eq!(
        daily.next_after(range.start, &range),
        Some(date(2025, 1, 1).at(12, 0, 0, 0))
    );

    assert_eq!(
        daily.next_after(date(2025, 1, 1).at(12, 0, 0, 0), &range),
        Some(date(2025, 1, 3).at(12, 0, 0, 0))
    );

    assert_eq!(
        daily.next_after(date(2025, 1, 1).at(11, 59, 59, 999), &range),
        Some(date(2025, 1, 1).at(12, 0, 0, 0))
    );

    assert_eq!(
        daily.next_after(range.end - 2.days(), &range),
        Some(date(2025, 1, 31).at(12, 0, 0, 0))
    );
    assert_eq!(daily.next_after(range.end - 12.hours(), &range), None);
    assert_eq!(daily.next_after(range.end, &range), None);
    assert_eq!(daily.next_after(DateTime::MAX, &range), None);
}

#[test]
fn daily_previous_before() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 2, 1).at(0, 0, 0, 0);
    let range = start..end;
    let daily = Daily::new(2);
    assert_eq!(daily.previous_before(DateTime::MIN, &range), None);
    assert_eq!(daily.previous_before(range.start, &range), None);
    assert_eq!(
        daily.previous_before(range.start + 1.second(), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        daily.previous_before(date(2025, 1, 1).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        daily.previous_before(date(2025, 1, 4).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 3).at(0, 0, 0, 0))
    );

    assert_eq!(
        daily.previous_before(range.end, &range),
        Some(date(2025, 1, 31).at(0, 0, 0, 0))
    );
}

#[test]
fn daily_at_previous_before() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 2, 1).at(0, 0, 0, 0);
    let range = start..end;
    let daily = Daily::new(2).at(time(12, 0, 0, 0));
    assert_eq!(daily.previous_before(DateTime::MIN, &range), None);
    assert_eq!(daily.previous_before(range.start, &range), None);
    assert_eq!(
        daily.previous_before(range.start + 12.hours(), &range),
        None
    );
    assert_eq!(
        daily.previous_before(range.start + 12.hours().seconds(1), &range),
        Some(date(2025, 1, 1).at(12, 0, 0, 0))
    );

    assert_eq!(
        daily.previous_before(date(2025, 1, 4).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 3).at(12, 0, 0, 0))
    );

    assert_eq!(
        daily.previous_before(range.end, &range),
        Some(date(2025, 1, 31).at(12, 0, 0, 0))
    );
}

#[test]
fn daily_closest_to() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 2, 1).at(0, 0, 0, 0);
    let range = start..end;
    let daily = Daily::new(2);

    assert_eq!(
        daily.closest_to(date(2025, 1, 1).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        daily.closest_to(date(2025, 1, 2).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 3).at(0, 0, 0, 0))
    );

    assert_eq!(
        daily.closest_to(date(2025, 1, 2).at(0, 0, 0, 0) - 1.nanosecond(), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        daily.closest_to(date(2024, 12, 31).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        daily.closest_to(date(2025, 2, 1).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 31).at(0, 0, 0, 0))
    );

    assert_eq!(
        daily.closest_to(date(2025, 2, 10).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 31).at(0, 0, 0, 0))
    );
}

#[test]
fn daily_closest_to_datetime_max() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let range = start..DateTime::MAX;
    let daily = Daily::new(2);

    assert_eq!(
        daily.closest_to(DateTime::MAX, &range),
        Some(date(9999, 12, 30).at(0, 0, 0, 0))
    );
}
