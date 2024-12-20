use std::marker::PhantomData;

use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Style, Styled},
    widgets::{block::BlockExt, Block, StatefulWidget, Widget},
};

use crate::{utils::layout_on_viewport, ListState};

/// A struct representing a list view.
/// The widget displays a scrollable list of items.
#[allow(clippy::module_name_repetitions)]
pub struct ListView<'a, T> {
    /// The total number of items in the list
    pub item_count: usize,

    ///  A `ListBuilder<T>` responsible for constructing the items in the list.
    pub builder: ListBuilder<'a, T>,

    /// Specifies the scroll axis. Either `Vertical` or `Horizontal`.
    pub scroll_axis: ScrollAxis,

    /// The base style of the list view.
    pub style: Style,

    /// The base block surrounding the widget list.
    pub block: Option<Block<'a>>,

    /// The scroll padding.
    pub(crate) scroll_padding: u16,

    /// Whether infinite scrolling is enabled or not.
    /// Disabled by default.
    pub(crate) infinite_scrolling: bool,
}

impl<'a, T> ListView<'a, T> {
    /// Creates a new `ListView` with a builder an item count.
    #[must_use]
    pub fn new(builder: ListBuilder<'a, T>, item_count: usize) -> Self {
        Self {
            builder,
            item_count,
            scroll_axis: ScrollAxis::Vertical,
            style: Style::default(),
            block: None,
            scroll_padding: 0,
            infinite_scrolling: true,
        }
    }

    /// Checks whether the widget list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.item_count == 0
    }

    /// Returns the length of the widget list.
    #[must_use]
    pub fn len(&self) -> usize {
        self.item_count
    }

    /// Sets the block style that surrounds the whole List.
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the base style of the List.
    #[must_use]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Set the scroll axis of the list.
    #[must_use]
    pub fn scroll_axis(mut self, scroll_axis: ScrollAxis) -> Self {
        self.scroll_axis = scroll_axis;
        self
    }

    /// Set the scroll padding of the list.
    #[must_use]
    pub fn scroll_padding(mut self, scroll_padding: u16) -> Self {
        self.scroll_padding = scroll_padding;
        self
    }

    /// Specify whether infinite scrolling should be enabled or not.
    #[must_use]
    pub fn infinite_scrolling(mut self, infinite_scrolling: bool) -> Self {
        self.infinite_scrolling = infinite_scrolling;
        self
    }
}

impl<T> Styled for ListView<'_, T> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

impl<'render, T: Copy + 'render> From<Vec<T>> for ListView<'render, T> {
    fn from(value: Vec<T>) -> Self {
        let item_count = value.len();
        let builder = ListBuilder::new(move |context| (value[context.index], 1));

        ListView::new(builder, item_count)
    }
}

/// This structure holds information about the item's position, selection
/// status, scrolling behavior, and size along the cross axis.
pub struct ListBuildContext {
    /// The position of the item in the list.
    pub index: usize,

    /// A boolean flag indicating whether the item is currently selected.
    pub is_selected: bool,

    /// Defines the axis along which the list can be scrolled.
    pub scroll_axis: ScrollAxis,

    /// The size of the item along the cross axis.
    pub cross_axis_size: u16,
}

/// A type alias for the closure.
type ListBuilderClosure<'render, T> = dyn Fn(&ListBuildContext) -> (T, u16) + 'render;

/// The builder to for constructing list elements in a `ListView<T>`
pub struct ListBuilder<'render, T> {
    closure: Box<ListBuilderClosure<'render, T>>,
    // _phantom: PhantomData<&'render T>,
}

impl<'render, T> ListBuilder<'render, T> {
    /// Creates a new `ListBuilder` taking a closure as a parameter
    pub fn new<F>(closure: F) -> Self
    where
        F: Fn(&ListBuildContext) -> (T, u16) + 'render,
    {
        ListBuilder {
            closure: Box::new(closure),
            // _phantom: PhantomData::default(),
        }
    }

    /// Method to call the stored closure.
    pub(crate) fn call_closure(&self, context: &ListBuildContext) -> (T, u16) {
        (self.closure)(context)
    }
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

impl<T: Widget> StatefulWidget for ListView<'_, T> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.set_num_elements(self.item_count);
        state.set_infinite_scrolling(self.infinite_scrolling);

        // Set the base style
        buf.set_style(area, self.style);

        // Set the base block
        self.block.render(area, buf);
        let area = self.block.inner_if_some(area);

        // List is empty
        if self.item_count == 0 {
            return;
        }

        // Set the dimension along the scroll axis and the cross axis
        let (main_axis_size, cross_axis_size) = match self.scroll_axis {
            ScrollAxis::Vertical => (area.height, area.width),
            ScrollAxis::Horizontal => (area.width, area.height),
        };

        // The coordinates of the first item with respect to the top left corner
        let (mut scroll_axis_pos, cross_axis_pos) = match self.scroll_axis {
            ScrollAxis::Vertical => (area.top(), area.left()),
            ScrollAxis::Horizontal => (area.left(), area.top()),
        };

        // Determine which widgets to show on the viewport and how much space they
        // get assigned to.
        let mut viewport = layout_on_viewport(
            state,
            &self.builder,
            self.item_count,
            main_axis_size,
            cross_axis_size,
            self.scroll_axis,
            self.scroll_padding,
        );

        let (start, end) = (
            state.view_state.offset,
            viewport.len() + state.view_state.offset,
        );
        for i in start..end {
            let Some(element) = viewport.remove(&i) else {
                break;
            };
            let visible_main_axis_size = element
                .main_axis_size
                .saturating_sub(element.truncation.value());
            let area = match self.scroll_axis {
                ScrollAxis::Vertical => Rect::new(
                    cross_axis_pos,
                    scroll_axis_pos,
                    cross_axis_size,
                    visible_main_axis_size,
                ),
                ScrollAxis::Horizontal => Rect::new(
                    scroll_axis_pos,
                    cross_axis_pos,
                    visible_main_axis_size,
                    cross_axis_size,
                ),
            };

            // Render truncated widgets.
            if element.truncation.value() > 0 {
                render_truncated(
                    element.widget,
                    area,
                    buf,
                    element.main_axis_size,
                    &element.truncation,
                    self.style,
                    self.scroll_axis,
                );
            } else {
                element.widget.render(area, buf);
            }

            scroll_axis_pos += visible_main_axis_size;
        }
    }
}

/// Render a truncated widget into a buffer. The method renders the widget fully into
/// a hidden buffer and moves the visible content into `buf`.
fn render_truncated<T: Widget>(
    item: T,
    available_area: Rect,
    buf: &mut Buffer,
    untruncated_size: u16,
    truncation: &Truncation,
    base_style: Style,
    scroll_axis: ScrollAxis,
) {
    // Create an hidden buffer for rendering the truncated element
    let (width, height) = match scroll_axis {
        ScrollAxis::Vertical => (available_area.width, untruncated_size),
        ScrollAxis::Horizontal => (untruncated_size, available_area.height),
    };
    let mut hidden_buffer = Buffer::empty(Rect {
        x: available_area.left(),
        y: available_area.top(),
        width,
        height,
    });
    hidden_buffer.set_style(hidden_buffer.area, base_style);
    item.render(hidden_buffer.area, &mut hidden_buffer);

    // Copy the visible part from the hidden buffer to the main buffer
    match scroll_axis {
        ScrollAxis::Vertical => {
            let offset = match truncation {
                Truncation::Top(value) => *value,
                _ => 0,
            };
            for y in available_area.top()..available_area.bottom() {
                let y_off = y + offset;
                for x in available_area.left()..available_area.right() {
                    if let Some(to) = buf.cell_mut(Position::new(x, y)) {
                        if let Some(from) = hidden_buffer.cell(Position::new(x, y_off)) {
                            *to = from.clone();
                        }
                    }
                }
            }
        }
        ScrollAxis::Horizontal => {
            let offset = match truncation {
                Truncation::Top(value) => *value,
                _ => 0,
            };
            for x in available_area.left()..available_area.right() {
                let x_off = x + offset;
                for y in available_area.top()..available_area.bottom() {
                    if let Some(to) = buf.cell_mut(Position::new(x, y)) {
                        if let Some(from) = hidden_buffer.cell(Position::new(x_off, y)) {
                            *to = from.clone();
                        }
                    }
                }
            }
        }
    };
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) enum Truncation {
    #[default]
    None,
    Top(u16),
    Bot(u16),
}

impl Truncation {
    pub(crate) fn value(&self) -> u16 {
        match self {
            Self::Top(value) | Self::Bot(value) => *value,
            Self::None => 0,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ListBuilder;
    use ratatui::widgets::Block;

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

    fn test_data<'render>(
        total_height: u16,
    ) -> (Rect, Buffer, ListView<'static, TestItem>, ListState) {
        let area = Rect::new(0, 0, 5, total_height);
        let list = ListView::new(ListBuilder::new(|_| (TestItem {}, 3)), 3);
        (area, Buffer::empty(area), list, ListState::default())
    }

    #[test]
    fn not_truncated() {
        // given
        let (area, mut buf, list, mut state) = test_data(9);

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
        let area = Rect::new(0, 0, 5, 2);
        let mut buf = Buffer::empty(area);
        let mut state = ListState::default();
        let builder = ListBuilder::new(|_| (TestItem {}, 0));
        let list = ListView::new(builder, 0);

        // when
        list.render(area, &mut buf, &mut state);

        // then
        assert_buffer_eq(buf, Buffer::with_lines(vec!["     ", "     "]))
    }

    #[test]
    fn zero_size() {
        // given
        let (area, mut buf, list, mut state) = test_data(0);

        // when
        list.render(area, &mut buf, &mut state);

        // then
        assert_buffer_eq(buf, Buffer::empty(area))
    }

    #[test]
    fn truncated_bot() {
        // given
        let (area, mut buf, list, mut state) = test_data(8);

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
    fn truncated_top() {
        // given
        let (area, mut buf, list, mut state) = test_data(8);
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

    #[test]
    fn scroll_up() {
        let (area, mut buf, list, mut state) = test_data(8);
        // Select last element and render
        state.select(Some(2));
        list.render(area, &mut buf, &mut state);
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
        );

        // Select first element and render
        let (_, mut buf, list, _) = test_data(8);
        state.select(Some(1));
        list.render(area, &mut buf, &mut state);
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
