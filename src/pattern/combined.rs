use crate::pattern::utils::{closest_to, pick_best};
use crate::{DateTimeRange, Pattern, private};
use core::cmp::Ord;
use jiff::civil::DateTime;

/// A combination of two recurrence patterns.
///
/// This type is returned by the `.and()` method of the [`Combine`][crate::Combine] trait and
/// allows combining values of the types from the [`pattern` module][crate::pattern] into more
/// complex recurrence patterns.
///
/// See the documentation of the [`Combine`][crate::Combine] trait for usage examples and more
/// context.
#[derive(Debug, Clone, Default)]
pub struct Combined<L, R> {
    left: L,
    right: R,
}

impl<L, R> Combined<L, R>
where
    L: Pattern,
    R: Pattern,
{
    /// Create a new `Combined` from two recurrence patterns.
    ///
    /// Consider using the [`.and()`][crate::Combine::and] method of the `Combine` trait instead
    /// because it's more convenient.
    pub fn new(left: L, right: R) -> Combined<L, R> {
        Combined { left, right }
    }
}

impl<L, R> Pattern for Combined<L, R>
where
    L: Pattern,
    R: Pattern,
{
    fn next_after(&self, instant: DateTime, range: DateTimeRange) -> Option<DateTime> {
        let left = self.left.next_after(instant, range);
        let right = self.right.next_after(instant, range);
        pick_best(left, right, Ord::min)
    }

    fn previous_before(&self, instant: DateTime, range: DateTimeRange) -> Option<DateTime> {
        let left = self.left.previous_before(instant, range);
        let right = self.right.previous_before(instant, range);
        pick_best(left, right, Ord::max)
    }

    fn closest_to(&self, instant: DateTime, range: DateTimeRange) -> Option<DateTime> {
        let left = self.left.closest_to(instant, range);
        let right = self.right.closest_to(instant, range);
        pick_best(left, right, |left, right| closest_to(instant, left, right))
    }
}

impl<L, R> private::Sealed for Combined<L, R> {}
