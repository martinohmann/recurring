use crate::Repeat;
use alloc::vec::Vec;
use core::ops::Range;
use jiff::{
    Span, SpanTotal, ToSpan, Unit,
    civil::{DateTime, Time},
};

#[derive(Debug, Clone)]
pub struct Secondly {
    interval: i32,
}

impl Secondly {
    pub fn new(interval: i32) -> Secondly {
        Secondly { interval }
    }
}

impl Repeat for Secondly {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_add(self.interval.seconds()).ok()
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_sub(self.interval.seconds()).ok()
    }

    fn is_aligned_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> bool {
        is_aligned_to_series(instant, bounds, self.interval.seconds())
    }

    fn align_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime> {
        align_to_series(instant, bounds, self.interval.seconds())
    }
}

#[derive(Debug, Clone)]
pub struct Minutely {
    interval: i32,
}

impl Minutely {
    pub fn new(interval: i32) -> Minutely {
        Minutely { interval }
    }
}

impl Repeat for Minutely {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_add(self.interval.minutes()).ok()
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_sub(self.interval.minutes()).ok()
    }

    fn is_aligned_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> bool {
        is_aligned_to_series(instant, bounds, self.interval.minutes())
    }

    fn align_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime> {
        align_to_series(instant, bounds, self.interval.minutes())
    }
}

#[derive(Debug, Clone)]
pub struct Hourly {
    interval: i32,
}

impl Hourly {
    pub fn new(interval: i32) -> Hourly {
        Hourly { interval }
    }
}

impl Repeat for Hourly {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_add(self.interval.hours()).ok()
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_sub(self.interval.hours()).ok()
    }

    fn is_aligned_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> bool {
        is_aligned_to_series(instant, bounds, self.interval.hours())
    }

    fn align_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime> {
        align_to_series(instant, bounds, self.interval.hours())
    }
}

#[derive(Debug, Clone)]
pub struct Daily {
    interval: i32,
    at: Vec<Time>,
}

impl Daily {
    pub fn new(interval: i32) -> Daily {
        Daily {
            interval,
            at: Vec::new(),
        }
    }

    #[must_use]
    pub fn at(mut self, time: Time) -> Daily {
        self.at.push(time);
        self.at.sort();
        self
    }
}

impl Repeat for Daily {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        if self.at.is_empty() {
            instant.checked_add(self.interval.days()).ok()
        } else {
            for time in &self.at {
                let date = instant.with().time(*time).build().ok()?;

                if date > instant {
                    return Some(date);
                }
            }

            let next_date = instant.checked_add(self.interval.days()).ok()?;

            next_date.with().time(self.at[0]).build().ok()
        }
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        if self.at.is_empty() {
            instant.checked_sub(self.interval.days()).ok()
        } else {
            for time in self.at.iter().rev() {
                let date = instant.with().time(*time).build().ok()?;

                if date < instant {
                    return Some(date);
                }
            }

            let next_date = instant.checked_sub(self.interval.days()).ok()?;

            next_date.with().time(*self.at.last().unwrap()).build().ok()
        }
    }

    fn is_aligned_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> bool {
        if self.at.is_empty() {
            return is_aligned_to_series(instant, bounds, self.interval.days());
        }

        instant
            .checked_sub(1.minute())
            .ok()
            .and_then(|start| self.next_event(start))
            == Some(instant)
    }

    fn align_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime> {
        if self.at.is_empty() {
            return align_to_series(instant, bounds, self.interval.days());
        }

        instant.with().time(*self.at.first().unwrap()).build().ok()
    }
}

#[derive(Debug, Clone)]
pub struct Monthly {
    interval: i32,
}

impl Monthly {
    pub fn new(interval: i32) -> Monthly {
        Monthly { interval }
    }
}

impl Repeat for Monthly {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_add(self.interval.months()).ok()
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_sub(self.interval.months()).ok()
    }

    fn is_aligned_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> bool {
        is_aligned_to_series(instant, bounds, self.interval.months())
    }

    fn align_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime> {
        align_to_series(instant, bounds, self.interval.months())
    }
}

#[derive(Debug, Clone)]
pub struct Yearly {
    interval: i32,
}

impl Yearly {
    pub fn new(interval: i32) -> Yearly {
        Yearly { interval }
    }
}

impl Repeat for Yearly {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_add(self.interval.years()).ok()
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_sub(self.interval.years()).ok()
    }

    fn is_aligned_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> bool {
        is_aligned_to_series(instant, bounds, self.interval.years())
    }

    fn align_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime> {
        align_to_series(instant, bounds, self.interval.years())
    }
}

pub fn secondly(interval: i32) -> Secondly {
    Secondly::new(interval)
}

pub fn minutely(interval: i32) -> Minutely {
    Minutely::new(interval)
}

pub fn hourly(interval: i32) -> Hourly {
    Hourly::new(interval)
}

pub fn daily(interval: i32) -> Daily {
    Daily::new(interval)
}

pub fn monthly(interval: i32) -> Monthly {
    Monthly::new(interval)
}

pub fn yearly(interval: i32) -> Yearly {
    Yearly::new(interval)
}

fn is_aligned_to_series(instant: DateTime, bounds: &Range<DateTime>, interval: Span) -> bool {
    intervals_until(bounds.start, instant, interval)
        .is_some_and(|intervals| intervals.trunc() == intervals)
}

fn align_to_series(
    instant: DateTime,
    bounds: &Range<DateTime>,
    interval: Span,
) -> Option<DateTime> {
    let intervals = get_alignment_intervals(instant, bounds, interval)?;
    bounds.start.checked_add(intervals * interval).ok()
}

fn get_alignment_intervals(
    instant: DateTime,
    bounds: &Range<DateTime>,
    interval: Span,
) -> Option<i64> {
    if instant <= bounds.start {
        return Some(0);
    }

    let end = if instant >= bounds.end {
        bounds.end
    } else {
        instant
    };

    let mut intervals = intervals_until(bounds.start, end, interval)?;

    if end == bounds.end && intervals.round() >= intervals {
        // The series would hit the end bound exactly or due to rounding up. We need to substract
        // an interval because the series end bound is exclusive.
        intervals -= 1.0;
    }

    #[allow(clippy::cast_possible_truncation)]
    Some(intervals.round() as i64)
}

fn intervals_until(start: DateTime, end: DateTime, interval: Span) -> Option<f64> {
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
    use jiff::civil::date;

    #[test]
    fn test_is_aligned_to_series() {
        let start = date(2025, 1, 1).at(0, 0, 0, 0);
        let bounds = start..DateTime::MAX;

        assert!(!is_aligned_to_series(
            date(2025, 1, 1).at(0, 30, 0, 0),
            &bounds,
            1.hour()
        ));
        assert!(is_aligned_to_series(
            date(2025, 1, 1).at(0, 0, 0, 0),
            &bounds,
            1.hour()
        ));
        assert!(is_aligned_to_series(
            date(2025, 1, 1).at(1, 0, 0, 0),
            &bounds,
            1.hour()
        ));
    }

    #[test]
    fn test_align_to_series() {
        let start = date(2025, 1, 1).at(0, 0, 0, 0);
        let end = date(2025, 1, 3).at(0, 0, 0, 0);
        let bounds = start..end;

        assert_eq!(
            align_to_series(date(2025, 1, 1).at(0, 0, 0, 0), &bounds, 1.hour()),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            align_to_series(date(2025, 1, 1).at(0, 30, 0, 0), &bounds, 1.hour()),
            Some(date(2025, 1, 1).at(1, 0, 0, 0))
        );

        assert_eq!(
            align_to_series(
                date(2025, 1, 1)
                    .at(0, 30, 0, 0)
                    .checked_sub(1.nanosecond())
                    .unwrap(),
                &bounds,
                1.hour()
            ),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            align_to_series(date(2024, 12, 31).at(0, 30, 0, 0), &bounds, 1.hour()),
            Some(date(2025, 1, 1).at(0, 0, 0, 0))
        );

        assert_eq!(
            align_to_series(date(2025, 1, 3).at(0, 0, 0, 0), &bounds, 1.hour()),
            Some(date(2025, 1, 2).at(23, 0, 0, 0))
        );

        assert_eq!(
            align_to_series(date(2025, 2, 10).at(0, 30, 0, 0), &bounds, 1.hour()),
            Some(date(2025, 1, 2).at(23, 0, 0, 0))
        );
    }
}
