use crate::Repeat;
use core::cmp::Ord;
use core::ops::Range;
use jiff::civil::DateTime;

/// A combination of two `Repeat` values.
///
/// This type is returned by the `.and()` method of the [`Combine`][crate::Combine] trait and
/// exists to support building more complex repetition pattern than supported by the individual
/// types from the [`repeat` module][crate::repeat].
#[derive(Debug, Clone, Default)]
pub struct Combined<L, R> {
    left: L,
    right: R,
}

impl<L, R> Combined<L, R>
where
    L: Repeat,
    R: Repeat,
{
    /// Create a new `Combined` from two `Repeat` values.
    ///
    /// It's usually more convenient to use the [`.and()`][crate::Combine::and] of the `Combine`
    /// trait instead of calling this directly.
    pub fn new(left: L, right: R) -> Combined<L, R> {
        Combined { left, right }
    }
}

impl<L, R> Repeat for Combined<L, R>
where
    L: Repeat,
    R: Repeat,
{
    fn next_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let left = self.left.next_event(instant, range);
        let right = self.right.next_event(instant, range);
        either_or(left, right, Ord::min)
    }

    fn previous_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let left = self.left.previous_event(instant, range);
        let right = self.right.previous_event(instant, range);
        either_or(left, right, Ord::max)
    }

    fn closest_event(&self, instant: DateTime, range: &Range<DateTime>) -> Option<DateTime> {
        let left = self.left.closest_event(instant, range);
        let right = self.right.closest_event(instant, range);
        either_or(left, right, |left, right| {
            if left.duration_until(instant).abs() <= right.duration_until(instant).abs() {
                left
            } else {
                right
            }
        })
    }
}

/// Returns either `left` or `right` if only one of them is `Some(_)`. If both are `Some` returns
/// the result of `or_fn`, otherwise `None`.
#[inline]
fn either_or<F: FnOnce(DateTime, DateTime) -> DateTime>(
    left: Option<DateTime>,
    right: Option<DateTime>,
    or_fn: F,
) -> Option<DateTime> {
    match (left, right) {
        (Some(left), Some(right)) => Some(or_fn(left, right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}
