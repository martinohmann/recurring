//! Patterns for recurring events.

mod combined;
mod cron;
mod daily;
mod interval;
mod timeunit;
mod utils;

pub use combined::Combined;
pub use cron::Cron;
pub use daily::Daily;
pub use interval::Interval;
use jiff::{Span, ToSpan};

/// Creates a cron recurrence pattern.
///
/// Unless further methods are called on the returned `Cron`, this will produce events at every
/// second.
///
/// # Example
///
/// ```
/// use jiff::ToSpan;
/// use recurring::pattern::cron;
///
/// let pattern = cron().second(10);
/// ```
#[inline]
pub fn cron() -> Cron {
    Cron::new()
}

/// Creates a recurrence pattern for events recurring on a fixed interval.
///
/// # Panics
///
/// Panics if `span` is negative or zero.
///
/// # Example
///
/// ```
/// use jiff::ToSpan;
/// use recurring::pattern::interval;
///
/// let every_day_and_a_half = interval(1.day().hours(12));
/// ```
#[inline]
pub fn interval(span: Span) -> Interval {
    Interval::new(span)
}

/// Creates a recurrence pattern for events recurring on a per-second interval.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::pattern::secondly;
///
/// let every_ten_seconds = secondly(10);
/// ```
#[inline]
pub fn secondly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.seconds())
}

/// Creates a recurrence pattern for events recurring on a per-minute interval.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::pattern::minutely;
///
/// let every_thirty_minutes = minutely(30);
/// ```
#[inline]
pub fn minutely<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.minutes())
}

/// Creates a recurrence pattern for events recurring on an hourly basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::pattern::hourly;
///
/// let every_twelve_hours = hourly(12);
/// ```
#[inline]
pub fn hourly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.hours())
}

/// Creates a recurrence pattern for events recurring on a daily basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::pattern::daily;
///
/// let every_two_days = daily(2);
/// ```
#[inline]
pub fn daily<I: ToSpan>(interval: I) -> Daily {
    Daily::new(interval)
}

/// Creates a recurrence pattern for events recurring on a monthly basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::pattern::monthly;
///
/// let every_three_months = monthly(3);
/// ```
#[inline]
pub fn monthly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.months())
}

/// Creates a recurrence pattern for events recurring on a yearly basis.
///
/// # Panics
///
/// Panics if `interval` is negative or zero.
///
/// # Example
///
/// ```
/// use recurring::pattern::yearly;
///
/// let every_five_years = yearly(5);
/// ```
#[inline]
pub fn yearly<I: ToSpan>(interval: I) -> Interval {
    Interval::new(interval.years())
}
