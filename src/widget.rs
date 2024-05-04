#![allow(clippy::cast_possible_truncation)]
use ratatui::{
    prelude::{Buffer, Rect},
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{traits::RenderContext, ListState, ListWidget};

/// A [`List`] is a widget for Ratatui that can render an arbitrary list of widgets.
/// It is generic over `T`, where each widget `T` should implement the [`ListableWidget`]
/// trait.
#[derive(Clone)]
pub struct List<'a, T: ListWidget> {
    /// The list's items.
    pub items: Vec<T>,

    /// Style used as a base style for the widget.
    style: Style,

    /// Block surrounding the widget list.
    block: Option<Block<'a>>,

    /// Specifies the scroll axis. Either `Vertical` or `Horizontal`.
    scroll_axis: ScrollAxis,
}

impl<'a, T: ListWidget> List<'a, T> {
    /// Instantiates a widget list with elements.
    ///
    /// # Arguments
    ///
    /// * `items` - A vector of elements implementing the [`ListableWidget`] trait.
    #[must_use]
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            style: Style::default(),
            block: None,
            scroll_axis: ScrollAxis::default(),
        }
    }

    /// Sets the block style that surrounds the whole List.
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the base style of the List.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Checks whether the widget list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the length of the widget list.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Set the scroll direction of the list.
    #[must_use]
    pub fn scroll_direction(mut self, scroll_axis: ScrollAxis) -> Self {
        self.scroll_axis = scroll_axis;
        self
    }
}

impl<'a, T: ListWidget> From<Vec<T>> for List<'a, T> {
    /// Instantiates a [`List`] from a vector of elements implementing
    /// the [`ListableWidget`] trait.
    fn from(items: Vec<T>) -> Self {
        Self::new(items)
    }
}

impl<'a, T: ListWidget> StatefulWidget for List<'a, T> {
    type State = ListState;
    /// Renders a mutable reference to a widget list
    #[allow(deprecated)]
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let style = self.style;
        let scroll_axis = self.scroll_axis;

        let raw_items = self.items;
        let mut block = self.block;
        state.set_num_elements(raw_items.len());

        // Set the base style
        buf.set_style(area, style);
        let area = match block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        // List is empty
        if raw_items.is_empty() {
            return;
        }

        // Set the dimension along the scroll axis and the cross axis
        let (scroll_axis_size, cross_axis_size) = match scroll_axis {
            ScrollAxis::Vertical => (area.height, area.width),
            ScrollAxis::Horizontal => (area.width, area.height),
        };

        // The starting coordinates of the current item
        let (mut scroll_axis_pos, cross_axis_pos) = match scroll_axis {
            ScrollAxis::Vertical => (area.top(), area.left()),
            ScrollAxis::Horizontal => (area.left(), area.top()),
        };

        // Call the user provided callback to modify the items based on render info
        let mut items = Vec::new();
        for (index, item) in raw_items.into_iter().enumerate() {
            let highlighted = state.selected().map_or(false, |j| index == j);

            let context = RenderContext {
                cross_axis_size,
                is_selected: highlighted,
                scroll_axis,
                index,
            };

            let (item, main_axis_size) = item.pre_render(&context);
            items.push((item, main_axis_size));
        }

        // Determine which widgets to show on the viewport and how much space they
        // get assigned to. The number of elements in `view_heights` is less than
        // the number of elements in `raw_heights` if not all widgets are shown
        // on the viewport.
        let heights: Vec<_> = items.iter().map(|x| x.1).collect();
        let sizes_scroll_direction = state.update_view_port(&heights, scroll_axis_size);

        // Drain out elements that are shown on the view port from the vector of
        // all elements.
        let first = state.offset;
        let last = sizes_scroll_direction.len() + first;
        let mut items_in_view = items.drain(first..last);

        // Iterate over the modified items
        let num_items = sizes_scroll_direction.len();
        for (i, size) in sizes_scroll_direction.into_iter().enumerate() {
            let area = match scroll_axis {
                ScrollAxis::Vertical => {
                    Rect::new(cross_axis_pos, scroll_axis_pos, cross_axis_size, size)
                }
                ScrollAxis::Horizontal => {
                    Rect::new(scroll_axis_pos, cross_axis_pos, size, cross_axis_size)
                }
            };
            if let Some(item) = items_in_view.next() {
                // Check if the item needs to be truncated
                if area.height < item.1 {
                    // Determine if truncation should happen at the top or the bottom
                    let truncate_top = i == 0 && num_items > 1;
                    render_truncated(item.0, item.1, area, buf, scroll_axis, truncate_top, style);
                } else {
                    item.0.render(area, buf);
                }
            }

            scroll_axis_pos += size;
        }
    }
}

/// Renders a listable widget within a specified area of a buffer, potentially truncating the widget content based on scrolling direction.
/// `truncate_top` indicates whether to truncate the content from the top or bottom.
fn render_truncated<T: ListWidget>(
    item: T,
    main_axis_size: u16,
    area: Rect,
    buf: &mut Buffer,
    scroll_axis: ScrollAxis,
    truncate_top: bool,
    base_style: Style,
) {
    let item_size = main_axis_size;
    // Create an intermediate buffer for rendering the truncated element
    let (width, height) = match scroll_axis {
        ScrollAxis::Vertical => (area.width, item_size),
        ScrollAxis::Horizontal => (item_size, area.height),
    };
    let mut hidden_buffer = Buffer::empty(Rect {
        x: area.left(),
        y: area.top(),
        width,
        height,
    });
    hidden_buffer.set_style(hidden_buffer.area, base_style);
    item.render(hidden_buffer.area, &mut hidden_buffer);

    // Copy the visible part from the intermediate buffer to the main buffer
    match scroll_axis {
        ScrollAxis::Vertical => {
            let offset = if truncate_top {
                item_size.saturating_sub(area.height)
            } else {
                0
            };
            for x in area.left()..area.right() {
                for y in area.top()..area.bottom() {
                    *buf.get_mut(x, y) = hidden_buffer.get(x, y + offset).clone();
                }
            }
        }
        ScrollAxis::Horizontal => {
            let offset = if truncate_top {
                item_size.saturating_sub(area.width)
            } else {
                0
            };
            for x in area.left()..area.right() {
                for y in area.top()..area.bottom() {
                    *buf.get_mut(x, y) = hidden_buffer.get(x + offset, y).clone();
                }
            }
        }
    };
}

/// Represents the scroll axis of a list.
#[derive(Debug, Default, Clone, Copy)]
pub enum ScrollAxis {
    /// Indicates vertical scrolling. This is the default.
    #[default]
    Vertical,

    /// Indicates horizontal scrolling.
    Horizontal,
}

#[cfg(test)]
mod test {
    use super::*;
    use ratatui::widgets::Borders;

    struct TestItem {}
    impl Widget for TestItem {
        fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized,
        {
            Block::default().borders(Borders::ALL).render(area, buf);
        }
    }

    impl ListWidget for TestItem {
        fn pre_render(self, context: &RenderContext) -> (Self, u16) {
            let main_axis_size = match context.scroll_axis {
                ScrollAxis::Vertical => 3,
                ScrollAxis::Horizontal => 3,
            };
            (self, main_axis_size)
        }
    }

    fn init(height: u16) -> (Rect, Buffer, List<'static, TestItem>, ListState) {
        let area = Rect::new(0, 0, 5, height);
        (
            area,
            Buffer::empty(area),
            List::new(vec![TestItem {}, TestItem {}, TestItem {}]),
            ListState::default(),
        )
    }

    #[test]
    fn not_truncated() {
        // given
        let (area, mut buf, list, mut state) = init(9);

        // when
        list.render(area, &mut buf, &mut state);

        // then
        assert_buffer_eq(
            buf,
            Buffer::with_lines(vec![
                "┌───┐",
                "│   │",
                "└───┘",
                "┌───┐",
                "│   │",
                "└───┘",
                "┌───┐",
                "│   │",
                "└───┘",
            ]),
        )
    }

    #[test]
    fn bottom_is_truncated() {
        // given
        let (area, mut buf, list, mut state) = init(8);

        // when
        list.render(area, &mut buf, &mut state);

        // then
        assert_buffer_eq(
            buf,
            Buffer::with_lines(vec![
                "┌───┐",
                "│   │",
                "└───┘",
                "┌───┐",
                "│   │",
                "└───┘",
                "┌───┐",
                "│   │",
            ]),
        )
    }

    #[test]
    fn top_is_truncated() {
        // given
        let (area, mut buf, list, mut state) = init(8);
        state.select(Some(2));

        // when
        list.render(area, &mut buf, &mut state);

        // then
        assert_buffer_eq(
            buf,
            Buffer::with_lines(vec![
                "│   │",
                "└───┘",
                "┌───┐",
                "│   │",
                "└───┘",
                "┌───┐",
                "│   │",
                "└───┘",
            ]),
        )
    }

    fn assert_buffer_eq(actual: Buffer, expected: Buffer) {
        if actual.area != expected.area {
            panic!(
                "buffer areas not equal expected: {:?} actual: {:?}",
                expected, actual
            );
        }
        let diff = expected.diff(&actual);
        if !diff.is_empty() {
            panic!(
                "buffer contents not equal\nexpected: {:?}\nactual: {:?}",
                expected, actual,
            );
        }
        assert_eq!(actual, expected, "buffers not equal");
    }
}
