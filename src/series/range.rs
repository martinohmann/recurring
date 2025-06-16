use crate::series::SeriesCore;
use crate::{DateTimeRange, Event, Pattern};
use jiff::civil::DateTime;

/// An iterator over a sub-range of events in a [`Series`][crate::Series].
///
/// This struct is created by the [`.range()`][crate::Series::range] method of a `Series`. See its
/// documentation for more.
#[derive(Debug, Clone)]
pub struct Range<'a, P> {
    core: &'a SeriesCore<P>,
    range: DateTimeRange,
    first: bool,
    cursor_front: Option<DateTime>,
    cursor_back: Option<DateTime>,
}

impl<'a, P: Pattern> Range<'a, P> {
    pub(crate) fn new(core: &'a SeriesCore<P>, range: DateTimeRange) -> Range<'a, P> {
        Range {
            core,
            range,
            first: true,
            cursor_front: Some(range.start),
            cursor_back: Some(range.end),
        }
    }
}

impl<P> Iterator for Range<'_, P>
where
    P: Pattern,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let cursor = self.cursor_front.take()?;
        let event = if self.first {
            self.first = false;
            self.core.get_closest_to(cursor, self.range)?
        } else {
            self.core.get_next_after(cursor, self.range)?
        };

        self.cursor_front = Some(event.start());
        Some(event)
    }
}

impl<P> DoubleEndedIterator for Range<'_, P>
where
    P: Pattern,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let cursor = self.cursor_back.take()?;
        let event = self.core.get_previous_before(cursor, self.range)?;
        self.cursor_back = Some(event.start());
        Some(event)
    }
}
