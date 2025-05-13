use jiff::ToSpan;
use jiff::civil::{DateTime, date};
use recurring::Repeat;
use recurring::repeat::Period;

#[test]
fn period_next_after() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 1, 3).at(0, 0, 0, 0);
    let range = start..end;
    let period = Period::new(1.hour());
    assert_eq!(
        period.next_after(DateTime::MIN, &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );
    assert_eq!(
        period.next_after(range.start.checked_sub(1.nanosecond()).unwrap(), &range,),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        period.next_after(range.start, &range),
        Some(date(2025, 1, 1).at(1, 0, 0, 0))
    );

    assert_eq!(
        period.next_after(date(2025, 1, 1).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(1, 0, 0, 0))
    );

    assert_eq!(
        period.next_after(range.end - 1.hour().nanoseconds(1), &range),
        Some(date(2025, 1, 2).at(23, 0, 0, 0))
    );
    assert_eq!(period.next_after(range.end - 1.hour(), &range), None,);
    assert_eq!(period.next_after(range.end, &range), None);
    assert_eq!(period.next_after(DateTime::MAX, &range), None);
}

#[test]
fn period_previous_before() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 1, 3).at(0, 0, 0, 0);
    let range = start..end;
    let period = Period::new(1.hour());
    assert_eq!(period.previous_before(DateTime::MIN, &range), None);
    assert_eq!(period.previous_before(range.start, &range), None);
    assert_eq!(
        period.previous_before(range.start + 1.second(), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        period.previous_before(date(2025, 1, 1).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        period.previous_before(date(2025, 1, 3).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 2).at(23, 0, 0, 0))
    );

    assert_eq!(
        period.previous_before(range.end, &range),
        Some(date(2025, 1, 2).at(23, 0, 0, 0))
    );
}

#[test]
fn period_closest_to() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let end = date(2025, 1, 3).at(0, 0, 0, 0);
    let range = start..end;
    let period = Period::new(1.hour());

    assert_eq!(
        period.closest_to(date(2025, 1, 1).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        period.closest_to(date(2025, 1, 1).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(1, 0, 0, 0))
    );

    assert_eq!(
        period.closest_to(
            date(2025, 1, 1)
                .at(0, 30, 0, 0)
                .checked_sub(1.nanosecond())
                .unwrap(),
            &range,
        ),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        period.closest_to(date(2024, 12, 31).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 1).at(0, 0, 0, 0))
    );

    assert_eq!(
        period.closest_to(date(2025, 1, 3).at(0, 0, 0, 0), &range),
        Some(date(2025, 1, 2).at(23, 0, 0, 0))
    );

    assert_eq!(
        period.closest_to(date(2025, 2, 10).at(0, 30, 0, 0), &range),
        Some(date(2025, 1, 2).at(23, 0, 0, 0))
    );
}

#[test]
fn period_closest_to_datetime_max() {
    let start = date(2025, 1, 1).at(0, 0, 0, 0);
    let range = start..DateTime::MAX;
    let period = Period::new(2.hours());

    assert_eq!(
        period.closest_to(DateTime::MAX, &range),
        Some(date(9999, 12, 31).at(22, 0, 0, 0))
    );
}
