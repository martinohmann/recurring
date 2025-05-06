use core::ops::Range;
use jiff::{Span, SpanTotal, Unit, civil::DateTime};

pub(super) fn closest_event(
    interval: Span,
    instant: DateTime,
    range: &Range<DateTime>,
) -> Option<DateTime> {
    let intervals = intervals_until_closest_event(interval, instant, range)?;
    range.start.checked_add(intervals * interval).ok()
}

fn intervals_until_closest_event(
    interval: Span,
    instant: DateTime,
    range: &Range<DateTime>,
) -> Option<i64> {
    if instant <= range.start {
        return Some(0);
    }

    let end = instant.min(range.end);
    let intervals = intervals_until(interval, range.start, end)?;
    let mut intervals_rounded = intervals.round();

    if end == range.end && intervals_rounded >= intervals {
        // The series would hit the end bound exactly or due to rounding up. We need to substract
        // an interval because the series end bound is exclusive.
        intervals_rounded -= 1.0;
    }

    #[allow(clippy::cast_possible_truncation)]
    Some(intervals_rounded as i64)
}

fn intervals_until(interval: Span, start: DateTime, end: DateTime) -> Option<f64> {
    let interval_seconds = span_seconds(interval)?;
    seconds_until(start, end).map(|seconds| seconds / interval_seconds)
}

fn seconds_until(start: DateTime, end: DateTime) -> Option<f64> {
    start.until(end).ok().and_then(span_seconds)
}

fn span_seconds(span: Span) -> Option<f64> {
    span.total(SpanTotal::from(Unit::Second).days_are_24_hours())
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::ToSpan;
    use jiff::civil::date;

    #[test]
    fn test_closest_event() {
        let start = date(2025, 1, 1).at(0, 0, 0, 0);
        let end = date(2025, 1, 3).at(0, 0, 0, 0);
        let bounds = start..end;

        assert_eq!(
            closest_event(1.hour(), date(2025, 1, 1).at(0, 0, 0, 0), &bounds),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            closest_event(1.hour(), date(2025, 1, 1).at(0, 30, 0, 0), &bounds),
            Some(date(2025, 1, 1).at(1, 0, 0, 0))
        );

        assert_eq!(
            closest_event(
                1.hour(),
                date(2025, 1, 1)
                    .at(0, 30, 0, 0)
                    .checked_sub(1.nanosecond())
                    .unwrap(),
                &bounds,
            ),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            closest_event(1.hour(), date(2024, 12, 31).at(0, 30, 0, 0), &bounds),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            closest_event(1.hour(), date(2025, 1, 3).at(0, 0, 0, 0), &bounds),
            Some(date(2025, 1, 2).at(23, 0, 0, 0))
        );

        assert_eq!(
            closest_event(1.hour(), date(2025, 2, 10).at(0, 30, 0, 0), &bounds),
            Some(date(2025, 1, 2).at(23, 0, 0, 0))
        );

        let bounds = start..DateTime::MAX;

        assert_eq!(
            closest_event(2.hours(), DateTime::MAX, &bounds),
            Some(date(9999, 12, 31).at(22, 0, 0, 0))
        );
    }
}
