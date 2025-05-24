use jiff::{Span, civil::DateTime};

/// Multiplier to increase the spans added to a given `DateTime` in bulk by the
/// `advance_until_` functions. This was chosen arbitrarily but seems to be plenty fast for the
/// worst case (see tests below).
const SPAN_MULTIPLIER: i64 = 10;

/// Advances `start` by `span` until the addition of another `span` would surpass `upper_bound` or
/// cause an overflow.
///
/// The returned value is guaranteed to be less than or equal to `upper_bound`.
#[inline]
pub(super) fn advance_by_until(start: DateTime, span: Span, upper_bound: DateTime) -> DateTime {
    assert!(start <= upper_bound);
    let (advanced, span_multiplier) = advance_by_until_fast(start, span, upper_bound);
    advance_by_until_slow(advanced, span, span_multiplier, upper_bound)
}

// Advances `current` by adding larger growing multiples of `span` until `upper_bound`.
//
// Returns the last `DateTime` smaller than `upper_bound` along with the last span multiplier that
// didn't cause an overflow.
#[inline]
fn advance_by_until_fast(
    mut current: DateTime,
    span: Span,
    upper_bound: DateTime,
) -> (DateTime, i64) {
    let mut span_multiplier = SPAN_MULTIPLIER;

    // This tries to add larger growing multiples of `span` until hitting the `upper_bound`,
    // overflowing `DateTime` or overflowing the `i64` span multiplier.
    while let Ok(span_multiple) = span.checked_mul(span_multiplier) {
        let Ok(next) = current.checked_add(span_multiple) else {
            // We overflowed `DateTime`.
            return (current, span_multiplier);
        };

        if next > upper_bound {
            return (current, span_multiplier);
        }

        current = next;

        let Some(multiplier) = span_multiplier.checked_mul(SPAN_MULTIPLIER) else {
            // The next `span_multiplier` would overflow `i64`. We have to abort and switch to the
            // slow path.
            return (current, span_multiplier);
        };

        span_multiplier = multiplier;
    }

    // `span` multiplication with `span_multiplier` overflowed. Return the previous one that
    // didn't.
    (current, span_multiplier / SPAN_MULTIPLIER)
}

// Advances `current` by adding multiples of `span` until `upper_bound`.
//
// Returns the closest span-aligned `DateTime` that's less than or equal to `upper_bound`.
#[inline]
fn advance_by_until_slow(
    mut current: DateTime,
    span: Span,
    mut span_multiplier: i64,
    upper_bound: DateTime,
) -> DateTime {
    // This adds as many large multiples of `span` as possible until hitting the `upper_bound`. It
    // then attempts to do the same with smaller multiples of `span` until it ends up adding single
    // spans until the bound is hit, yielding the span-aligned `DateTime` right before
    // `upper_bound`.
    while span_multiplier > 0 {
        let span_multiple = span_multiplier * span;

        while let Ok(next) = current.checked_add(span_multiple) {
            if next > upper_bound {
                break;
            }

            current = next;
        }

        span_multiplier /= SPAN_MULTIPLIER;
    }

    current
}

/// Returns the closest `DateTime` to `instant` from the provided datetimes `left` and `right`.
///
/// If distances are equal, this will always favor `right`.
pub(super) fn closest_to(instant: DateTime, left: DateTime, right: DateTime) -> DateTime {
    if left.duration_until(instant).abs() < right.duration_until(instant).abs() {
        left
    } else {
        right
    }
}

/// Pick the "best" of `left` and `right`.
///
/// Returns either `left` or `right` if only one of them is `Some(_)`. If both are `Some` this
/// returns the result of `f`, otherwise `None`.
#[inline]
pub(super) fn pick_best<F: FnOnce(DateTime, DateTime) -> DateTime>(
    left: Option<DateTime>,
    right: Option<DateTime>,
    f: F,
) -> Option<DateTime> {
    match (left, right) {
        (Some(left), Some(right)) => Some(f(left, right)),
        (left, right) => left.or(right),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::{ToSpan, civil::date};
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case(
        date(2025, 1, 1).at(0, 0, 0, 0),
        1.minute(),
        date(2025, 1, 1).at(0, 0, 0, 0),
        date(2025, 1, 1).at(0, 0, 0, 0)
    )]
    #[case(
        date(2025, 1, 1).at(0, 0, 0, 0),
        1.minute(),
        date(2025, 1, 10).at(1, 1, 30, 0),
        date(2025, 1, 10).at(1, 1, 0, 0)
    )]
    #[case(
        date(2024, 1, 1).at(2, 2, 2, 0),
        1.year(),
        date(2027, 1, 10).at(1, 1, 30, 0),
        date(2027, 1, 1).at(2, 2, 2, 0)
    )]
    #[case(
        date(2024, 1, 1).at(2, 2, 2, 0),
        1.month(),
        date(2024, 10, 10).at(1, 1, 30, 0),
        date(2024, 10, 1).at(2, 2, 2, 0)
    )]
    #[case(
        date(2025, 1, 1).at(2, 2, 2, 0),
        1.day(),
        date(2027, 1, 3).at(2, 2, 2, 0),
        date(2027, 1, 3).at(2, 2, 2, 0)
    )]
    fn advance_by_until_cases(
        #[case] start: DateTime,
        #[case] span: Span,
        #[case] end: DateTime,
        #[case] expected: DateTime,
    ) {
        assert_eq!(advance_by_until(start, span, end), expected)
    }

    #[test]
    fn advance_by_until_worst_case() {
        assert_eq!(
            advance_by_until(DateTime::MIN, 1.nanosecond(), DateTime::MAX),
            date(9999, 12, 31).at(23, 59, 59, 999_999_999)
        );
    }
}
