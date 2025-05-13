use core::ops::Range;
use jiff::{Span, SpanTotal, Unit, civil::DateTime};

// Precision tolerance for float division errors.
const PRECISION_TOLERANCE: f64 = 0.000_001;

// Returns true if the periods have a .0 fraction or are very close to it.
//
// This handles precision errors caused by float division.
#[inline]
pub(super) fn is_period_boundary(periods: f64) -> bool {
    periods.fract() < PRECISION_TOLERANCE
}

// Calculates the number of `period`s from the range's start until `instant`.
#[inline]
pub(super) fn periods_in_range_until(
    span: Span,
    range: &Range<DateTime>,
    instant: DateTime,
) -> Option<f64> {
    let period_seconds = span_seconds(span)?;
    let end = instant.max(range.start).min(range.end);
    seconds_until(range.start, end).map(|seconds| seconds / period_seconds)
}

#[inline]
fn seconds_until(start: DateTime, end: DateTime) -> Option<f64> {
    start.until(end).ok().and_then(span_seconds)
}

#[inline]
fn span_seconds(span: Span) -> Option<f64> {
    span.total(SpanTotal::from(Unit::Second).days_are_24_hours())
        .ok()
}
