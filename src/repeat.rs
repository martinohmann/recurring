use crate::Repeat;
use alloc::vec::Vec;
use core::ops::Range;
use jiff::{
    Span, SpanTotal, ToSpan, Unit,
    civil::{DateTime, Time},
};

#[derive(Debug, Clone)]
pub struct Interval(Span);

impl Interval {
    pub fn new(span: Span) -> Interval {
        Interval(span)
    }
}

impl Repeat for Interval {
    fn next_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_add(self.0).ok()
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        instant.checked_sub(self.0).ok()
    }

    fn is_aligned_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> bool {
        is_aligned_to_series(instant, bounds, self.0)
    }

    fn align_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime> {
        align_to_series(instant, bounds, self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Daily {
    interval: Interval,
    at: Vec<Time>,
}

impl Daily {
    pub fn new(interval: i32) -> Daily {
        Daily {
            interval: Interval::new(interval.days()),
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
            self.interval.next_event(instant)
        } else {
            for time in &self.at {
                let date = instant.with().time(*time).build().ok()?;

                if date > instant {
                    return Some(date);
                }
            }

            let next_date = instant.checked_add(self.interval.0).ok()?;

            next_date.with().time(self.at[0]).build().ok()
        }
    }

    fn previous_event(&self, instant: DateTime) -> Option<DateTime> {
        if self.at.is_empty() {
            self.interval.previous_event(instant)
        } else {
            for time in self.at.iter().rev() {
                let date = instant.with().time(*time).build().ok()?;

                if date < instant {
                    return Some(date);
                }
            }

            let next_date = instant.checked_sub(self.interval.0).ok()?;

            next_date.with().time(*self.at.last().unwrap()).build().ok()
        }
    }

    fn is_aligned_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> bool {
        if self.at.is_empty() {
            self.interval.is_aligned_to_series(instant, bounds)
        } else {
            instant
                .checked_sub(1.minute())
                .ok()
                .and_then(|start| self.next_event(start))
                == Some(instant)
        }
    }

    fn align_to_series(&self, instant: DateTime, bounds: &Range<DateTime>) -> Option<DateTime> {
        if self.at.is_empty() {
            self.interval.align_to_series(instant, bounds)
        } else {
            let aligned = self.interval.align_to_series(instant, bounds)?;

            for time in &self.at {
                let date = aligned.with().time(*time).build().ok()?;

                if bounds.contains(&date) {
                    return Some(date);
                }
            }

            None
        }
    }
}

pub fn interval(span: Span) -> Interval {
    Interval::new(span)
}

pub fn secondly(interval: i32) -> Interval {
    Interval::new(interval.seconds())
}

pub fn minutely(interval: i32) -> Interval {
    Interval::new(interval.minutes())
}

pub fn hourly(interval: i32) -> Interval {
    Interval::new(interval.hours())
}

pub fn daily(interval: i32) -> Daily {
    Daily::new(interval)
}

pub fn monthly(interval: i32) -> Interval {
    Interval::new(interval.months())
}

pub fn yearly(interval: i32) -> Interval {
    Interval::new(interval.years())
}

fn is_aligned_to_series(instant: DateTime, bounds: &Range<DateTime>, interval: Span) -> bool {
    const ALLOWED_INTERVAL_ERROR: f64 = 0.000_001;

    intervals_until(bounds.start, instant, interval)
        .is_some_and(|intervals| intervals.fract() < ALLOWED_INTERVAL_ERROR)
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

    let end = instant.min(bounds.end);

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

        let bounds = start..DateTime::MAX;

        assert!(is_aligned_to_series(
            date(9999, 12, 31).at(22, 0, 0, 0),
            &bounds,
            2.hour()
        ),);
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

        let bounds = start..DateTime::MAX;

        assert_eq!(
            align_to_series(DateTime::MAX, &bounds, 2.hour()),
            Some(date(9999, 12, 31).at(22, 0, 0, 0))
        );
    }
}
