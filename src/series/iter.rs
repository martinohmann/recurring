use crate::series::Series;
use crate::{Event, Pattern};
use jiff::civil::DateTime;

/// An iterator over the events of a [`Series`].
///
/// This struct is created by the [`.iter()`][Series::iter] method of a `Series`. See its
/// documentation for more.
#[derive(Debug, Clone)]
pub struct Iter<'a, P> {
    series: &'a Series<P>,
    first: bool,
    cursor_front: Option<DateTime>,
    cursor_back: Option<DateTime>,
}

impl<'a, P: Pattern> Iter<'a, P> {
    pub(crate) fn new(series: &'a Series<P>) -> Iter<'a, P> {
        Iter {
            series,
            first: true,
            cursor_front: Some(series.start()),
            cursor_back: Some(series.end()),
        }
    }
}

impl<P> Iterator for Iter<'_, P>
where
    P: Pattern,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let cursor = self.cursor_front.take()?;
        let event = if self.first {
            self.first = false;
            self.series.first_event()?
        } else {
            self.series.get_event_after(cursor)?
        };

        self.cursor_front = Some(event.start());
        Some(event)
    }
}

impl<P> DoubleEndedIterator for Iter<'_, P>
where
    P: Pattern,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let cursor = self.cursor_back.take()?;
        let event = self.series.get_event_before(cursor)?;
        self.cursor_back = Some(event.start());
        Some(event)
    }
}
