//! Types for specifying repeat intervals.

mod daily;
mod interval;
mod timespec;
mod utils;

pub use self::daily::Daily;
pub use self::interval::Interval;
pub use self::timespec::TimeSpec;
use jiff::{Span, ToSpan};

/// Creates an interval for repeating events.
///
/// # Panics
///
/// Panics if `span` is negative or zero.
///
/// # Example
///
/// ```
/// use jiff::ToSpan;
/// use recurring::repeat::interval;
///
/// let every_day_and_a_half = interval(1.day().hours(12));
/// ```
#[inline]
pub fn interval(span: Span) -> Interval {
    Interval::new(span)
}

/// Creates an interval for repeating events on a per-second basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::secondly;
///
/// let every_ten_seconds = secondly(10);
/// ```
#[inline]
pub fn secondly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.seconds())
}

/// Creates an interval for repeating events on a per-minute basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::minutely;
///
/// let every_thirty_minutes = minutely(30);
/// ```
#[inline]
pub fn minutely<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.minutes())
}

/// Creates an interval for repeating events on a hourly basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::hourly;
///
/// let every_twelve_hours = hourly(12);
/// ```
#[inline]
pub fn hourly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.hours())
}

/// Creates an interval for repeating events on a daily basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::daily;
///
/// let every_two_days = daily(2);
/// ```
#[inline]
pub fn daily<I: ToSpan>(interval: I) -> Daily {
    Daily::new(interval)
}

/// Creates an interval for repeating events on a monthly basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::monthly;
///
/// let every_three_months = monthly(3);
/// ```
#[inline]
pub fn monthly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.months())
}

/// Creates an interval for repeating events on a yearly basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::repeat::yearly;
///
/// let every_five_years = yearly(5);
/// ```
#[inline]
pub fn yearly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.years())
}
