//! This is the core implementation of the series which does not depend on a `DateTimeRange`.

use crate::{DateTimeRange, Event, Pattern};
use jiff::{Span, civil::DateTime};

#[derive(Debug, Clone)]
pub(crate) struct SeriesCore<P> {
    pub(crate) pattern: P,
    pub(crate) event_duration: Span,
}

impl<P> SeriesCore<P>
where
    P: Pattern,
{
    #[inline]
    pub(crate) fn new(pattern: P, event_duration: Span) -> SeriesCore<P> {
        SeriesCore {
            pattern,
            event_duration,
        }
    }

    #[inline]
    pub(crate) fn pattern(&self) -> &P {
        &self.pattern
    }

    #[inline]
    pub(crate) fn event_duration(&self) -> Span {
        self.event_duration
    }

    #[inline]
    pub(crate) fn get(&self, instant: DateTime, range: DateTimeRange) -> Option<Event> {
        self.pattern
            .closest_to(instant, range)
            .filter(|&start| start == instant)
            .and_then(|start| self.get_event_unchecked(start))
    }

    #[inline]
    pub fn get_containing(&self, instant: DateTime, range: DateTimeRange) -> Option<Event> {
        self.pattern
            .closest_to(instant, range)
            .filter(|&start| start <= instant)
            .or_else(|| self.pattern.previous_before(instant, range))
            .and_then(|start| self.get_event_unchecked(start))
            .filter(|event| event.contains(instant))
    }

    #[inline]
    pub(crate) fn get_next_after(&self, instant: DateTime, range: DateTimeRange) -> Option<Event> {
        self.pattern
            .next_after(instant, range)
            .and_then(|next| self.get_event_unchecked(next))
    }

    #[inline]
    pub(crate) fn get_previous_before(
        &self,
        instant: DateTime,
        range: DateTimeRange,
    ) -> Option<Event> {
        self.pattern
            .previous_before(instant, range)
            .and_then(|previous| self.get_event_unchecked(previous))
    }

    #[inline]
    pub(crate) fn get_closest_to(&self, instant: DateTime, range: DateTimeRange) -> Option<Event> {
        self.pattern
            .closest_to(instant, range)
            .and_then(|closest| self.get_event_unchecked(closest))
    }

    #[inline]
    fn get_event_unchecked(&self, start: DateTime) -> Option<Event> {
        if self.event_duration.is_positive() {
            let end = start.checked_add(self.event_duration).ok()?;
            Some(Event::new_unchecked(start, Some(end)))
        } else {
            Some(Event::at(start))
        }
    }
}
