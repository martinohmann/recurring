use jiff::{Span, civil::DateTime};

// Calculates the number of spans between `start` and `end`.
#[inline]
pub(super) fn spans_until(span: Span, start: DateTime, end: DateTime) -> Option<f64> {
    let span_duration = span.to_duration(start).ok()?;
    let duration = start.duration_until(end);
    Some(duration.div_duration_f64(span_duration))
}

/// Extension trait for floating point numbers.
pub(super) trait FloatExt: Sized {
    /// Returns the smallest integer greater than `self`.
    ///
    /// This is similar to `f64::ceil` but will yield the next larger integer if `self` already has
    /// a zero fractional part.
    fn ceil_strict(self) -> Self;

    /// Returns the largest integer less than `self`.
    ///
    /// This is similar to `f64::floor` but will yield the next smaller integer if `self` already
    /// has a zero fractional part.
    fn floor_strict(self) -> Self;
}

impl FloatExt for f64 {
    #[inline]
    fn ceil_strict(self) -> f64 {
        if self.fract() == 0.0 {
            self + 1.0
        } else {
            self.ceil()
        }
    }

    #[inline]
    fn floor_strict(self) -> f64 {
        if self.fract() == 0.0 {
            self - 1.0
        } else {
            self.floor()
        }
    }
}
