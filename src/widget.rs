#![allow(clippy::cast_possible_truncation)]
use ratatui::{
    prelude::{Buffer, Rect},
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{traits::PreRenderContext, utils::update_view_port, ListState, PreRender};

/// A [`List`] is a widget for Ratatui that can render an arbitrary list of widgets.
/// It is generic over `T`, where each widget `T` should implement the [`PreRender`]
/// trait.
#[derive(Clone)]
pub struct List<'a, T: PreRender> {
    /// The list's items.
    pub items: Vec<T>,

    /// Style used as a base style for the widget.
    style: Style,

    /// Block surrounding the widget list.
    block: Option<Block<'a>>,

    /// Specifies the scroll axis. Either `Vertical` or `Horizontal`.
    scroll_axis: ScrollAxis,
}

impl<'a, T: PreRender> List<'a, T> {
    /// Instantiates a widget list with elements.
    ///
    /// # Arguments
    ///
    /// * `items` - A vector of elements implementing the [`PreRender`] trait.
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

impl<'a, T: PreRender> From<Vec<T>> for List<'a, T> {
    /// Instantiates a [`List`] from a vector of elements implementing
    /// the [`PreRender`] trait.
    fn from(items: Vec<T>) -> Self {
        Self::new(items)
    }
}

impl<'a, T: PreRender> StatefulWidget for List<'a, T> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let style = self.style;
        let scroll_axis = self.scroll_axis;

        let mut items = self.items;
        let mut block = self.block;
        state.set_num_elements(items.len());

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
        if items.is_empty() {
            return;
        }

        // Set the dimension along the scroll axis and the cross axis
        let (scroll_axis_size, cross_axis_size) = match scroll_axis {
            ScrollAxis::Vertical => (area.height, area.width),
            ScrollAxis::Horizontal => (area.width, area.height),
        };

        // Call the user provided callback to modify the items based on render info
        let mut main_axis_sizes = Vec::new();
        for (index, item) in items.iter_mut().enumerate() {
            let highlighted = state.selected.map_or(false, |j| index == j);

            let context = PreRenderContext {
                cross_axis_size,
                is_selected: highlighted,
                scroll_axis,
                index,
            };

            let main_axis_size = item.pre_render(&context);
            main_axis_sizes.push(main_axis_size);
        }

        // Determine which widgets to show on the viewport and how much space they
        // get assigned to.
        let viewport_layouts = update_view_port(state, &main_axis_sizes, scroll_axis_size);

        // Drain out elements that are shown on the view port from the vector of
        // all elements.
        let num_items_viewport = viewport_layouts.len();
        let (start, end) = (state.offset, num_items_viewport + state.offset);
        let items_viewport = items.drain(start..end);

        // The starting coordinates of the current item
        let (mut scroll_axis_pos, cross_axis_pos) = match scroll_axis {
            ScrollAxis::Vertical => (area.top(), area.left()),
            ScrollAxis::Horizontal => (area.left(), area.top()),
        };

        // Render the widgets on the viewport
        for (i, (viewport_layout, item)) in
            viewport_layouts.into_iter().zip(items_viewport).enumerate()
        {
            let area = match scroll_axis {
                ScrollAxis::Vertical => Rect::new(
                    cross_axis_pos,
                    scroll_axis_pos,
                    cross_axis_size,
                    viewport_layout.size,
                ),
                ScrollAxis::Horizontal => Rect::new(
                    scroll_axis_pos,
                    cross_axis_pos,
                    viewport_layout.size,
                    cross_axis_size,
                ),
            };

            // Check if the item needs to be truncated
            if viewport_layout.truncated_by > 0 {
                let trunc_top = i == 0 && num_items_viewport > 1;
                let tot_size = viewport_layout.size + viewport_layout.truncated_by;
                render_trunc(item, area, buf, tot_size, scroll_axis, trunc_top, style);
            } else {
                item.render(area, buf);
            }

            scroll_axis_pos += viewport_layout.size;
        }
    }
}

/// Renders a listable widget within a specified area of a buffer, potentially truncating the widget content based on scrolling direction.
/// `truncate_top` indicates whether to truncate the content from the top or bottom.
fn render_trunc<T: Widget>(
    item: T,
    available_area: Rect,
    buf: &mut Buffer,
    total_size: u16,
    scroll_axis: ScrollAxis,
    truncate_top: bool,
    style: Style,
) {
    // Create an intermediate buffer for rendering the truncated element
    let (width, height) = match scroll_axis {
        ScrollAxis::Vertical => (available_area.width, total_size),
        ScrollAxis::Horizontal => (total_size, available_area.height),
    };
    let mut hidden_buffer = Buffer::empty(Rect {
        x: available_area.left(),
        y: available_area.top(),
        width,
        height,
    });
    hidden_buffer.set_style(hidden_buffer.area, style);
    item.render(hidden_buffer.area, &mut hidden_buffer);

    // Copy the visible part from the intermediate buffer to the main buffer
    match scroll_axis {
        ScrollAxis::Vertical => {
            let offset = if truncate_top {
                total_size.saturating_sub(available_area.height)
            } else {
                0
            };
            for y in available_area.top()..available_area.bottom() {
                let y_off = y + offset;
                for x in available_area.left()..available_area.right() {
                    *buf.get_mut(x, y) = hidden_buffer.get(x, y_off).clone();
                }
            }
        }
        ScrollAxis::Horizontal => {
            let offset = if truncate_top {
                total_size.saturating_sub(available_area.width)
            } else {
                0
            };
            for x in available_area.left()..available_area.right() {
                let x_off = x + offset;
                for y in available_area.top()..available_area.bottom() {
                    *buf.get_mut(x, y) = hidden_buffer.get(x_off, y).clone();
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

    impl PreRender for TestItem {
        fn pre_render(&mut self, context: &PreRenderContext) -> u16 {
            let main_axis_size = match context.scroll_axis {
                ScrollAxis::Vertical => 3,
                ScrollAxis::Horizontal => 3,
            };
            main_axis_size
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
    fn empty_list() {
        // given
        let (area, mut buf, _, mut state) = init(2);
        let list = List::new(Vec::<TestItem>::new());

        // when
        list.render(area, &mut buf, &mut state);

        // then
        assert_buffer_eq(buf, Buffer::with_lines(vec!["     ", "     "]))
    }

    #[test]
    fn zero_size() {
        // given
        let (area, mut buf, list, mut state) = init(0);

        // when
        list.render(area, &mut buf, &mut state);

        // then
        assert_buffer_eq(buf, Buffer::empty(area))
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
