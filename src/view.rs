use std::collections::HashMap;

use ratatui_core::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Style, Styled},
    widgets::{StatefulWidget, Widget},
};
use ratatui_widgets::block::Block;
use ratatui_widgets::block::BlockExt;
use ratatui_widgets::scrollbar::Scrollbar;

use crate::{
    utils::{compute_viewport_layout, ViewportElement},
    ListState,
};

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

    /// Specifies the scroll direction. Either `Forward` or `Backward`.
    pub scroll_direction: ScrollDirection,

    /// The base style of the list view.
    pub style: Style,

    /// The base block surrounding the widget list.
    pub block: Option<Block<'a>>,

    /// The scrollbar.
    pub scrollbar: Option<Scrollbar<'a>>,

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
            scroll_direction: ScrollDirection::Forward,
            style: Style::default(),
            block: None,
            scrollbar: None,
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

    /// Sets the scrollbar of the List.
    #[must_use]
    pub fn scrollbar(mut self, scrollbar: Scrollbar<'a>) -> Self {
        self.scrollbar = Some(scrollbar);
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

    #[must_use]
    pub fn scroll_direction(mut self, scroll_direction: ScrollDirection) -> Self {
        self.scroll_direction = scroll_direction;
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

impl<'a, T: Copy + 'a> From<Vec<T>> for ListView<'a, T> {
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
type ListBuilderClosure<'a, T> = dyn Fn(&ListBuildContext) -> (T, u16) + 'a;

/// The builder for constructing list elements in a `ListView<T>`
pub struct ListBuilder<'a, T> {
    closure: Box<ListBuilderClosure<'a, T>>,
}

impl<'a, T> ListBuilder<'a, T> {
    /// Creates a new `ListBuilder` taking a closure as a parameter
    ///
    /// # Example
    /// ```
    /// use ratatui::text::Line;
    /// use tui_widget_list::ListBuilder;
    ///
    /// let builder = ListBuilder::new(|context| {
    ///     let mut item = Line::from(format!("Item {:0}", context.index));
    ///
    ///     // Return the size of the widget along the main axis.
    ///     let main_axis_size = 1;
    ///
    ///     (item, main_axis_size)
    /// });
    /// ```
    pub fn new<F>(closure: F) -> Self
    where
        F: Fn(&ListBuildContext) -> (T, u16) + 'a,
    {
        ListBuilder {
            closure: Box::new(closure),
        }
    }

    /// Method to call the stored closure.
    pub(crate) fn call_closure(&self, context: &ListBuildContext) -> (T, u16) {
        (self.closure)(context)
    }
}

/// Represents the scroll axis of a list.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAxis {
    /// Indicates vertical scrolling. This is the default.
    #[default]
    Vertical,

    /// Indicates horizontal scrolling.
    Horizontal,
}

impl ScrollAxis {
    /// Returns `(main_axis_size, cross_axis_size)` for the given area.
    pub(crate) fn sizes(self, area: Rect) -> (u16, u16) {
        match self {
            Self::Vertical => (area.height, area.width),
            Self::Horizontal => (area.width, area.height),
        }
    }

    /// Returns `(scroll_axis_pos, cross_axis_pos)` for the given area.
    pub(crate) fn origin(self, area: Rect) -> (u16, u16) {
        match self {
            Self::Vertical => (area.top(), area.left()),
            Self::Horizontal => (area.left(), area.top()),
        }
    }

    /// Builds a `Rect` from axis-agnostic positions and sizes.
    pub(crate) fn rect(
        self,
        scroll_axis_pos: u16,
        cross_axis_pos: u16,
        main_axis_size: u16,
        cross_axis_size: u16,
    ) -> Rect {
        match self {
            Self::Vertical => Rect::new(
                cross_axis_pos,
                scroll_axis_pos,
                cross_axis_size,
                main_axis_size,
            ),
            Self::Horizontal => Rect::new(
                scroll_axis_pos,
                cross_axis_pos,
                main_axis_size,
                cross_axis_size,
            ),
        }
    }
}

/// Represents the scroll direction of a list.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    /// Indicates forward scrolling (top to bottom or left to right). This is the default.
    #[default]
    Forward,

    /// Indicates backward scrolling (bottom to top or right to left).
    Backward,
}

impl<T: Widget> StatefulWidget for ListView<'_, T> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render base style and optional surrounding block
        buf.set_style(area, self.style);
        if let Some(ref block) = self.block {
            block.render(area, buf);
        }
        let inner_area = self.block.inner_if_some(area);

        // Store layout information for post-render queries (e.g. hit testing)
        state.set_num_elements(self.item_count);
        state.set_infinite_scrolling(self.infinite_scrolling);
        state.set_inner_area(inner_area);
        state.set_scroll_axis(self.scroll_axis);
        state.set_scroll_direction(self.scroll_direction);

        if self.item_count == 0 {
            return;
        }

        // Resolve which items are visible and how they fit on the viewport
        let (main_axis_size, cross_axis_size) = self.scroll_axis.sizes(inner_area);
        let (mut scroll_axis_pos, cross_axis_pos) = self.scroll_axis.origin(inner_area);

        let mut viewport = resolve_viewport(
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

        // Backward direction: align items to the end of the axis
        if self.scroll_direction == ScrollDirection::Backward {
            let total_visible: u16 = (start..end)
                .filter_map(|i| viewport.get(&i))
                .map(|e| e.main_axis_size.saturating_sub(e.truncation.value()))
                .sum();
            scroll_axis_pos += main_axis_size.saturating_sub(total_visible);
        }

        // Render each visible item and cache sizes for hit testing
        let mut cached_sizes: HashMap<usize, u16> = HashMap::new();
        for i in start..end {
            let Some(element) = viewport.remove(&i) else {
                break;
            };

            let visible_main_axis_size = element
                .main_axis_size
                .saturating_sub(element.truncation.value());

            cached_sizes.insert(i, visible_main_axis_size);

            render_clipped(
                element.widget,
                self.scroll_axis.rect(
                    scroll_axis_pos,
                    cross_axis_pos,
                    visible_main_axis_size,
                    cross_axis_size,
                ),
                buf,
                element.main_axis_size,
                &element.truncation,
                self.style,
                self.scroll_axis,
            );

            scroll_axis_pos += visible_main_axis_size;
        }

        state.set_visible_main_axis_sizes(cached_sizes);

        if let Some(scrollbar) = self.scrollbar {
            scrollbar.render(area, buf, &mut state.scrollbar_state);
        }
    }
}

fn resolve_viewport<T>(
    state: &mut ListState,
    builder: &ListBuilder<T>,
    item_count: usize,
    main_axis_size: u16,
    cross_axis_size: u16,
    scroll_axis: ScrollAxis,
    scroll_padding: u16,
) -> HashMap<usize, ViewportElement<T>> {
    let viewport = compute_viewport_layout(
        state,
        builder,
        item_count,
        main_axis_size,
        cross_axis_size,
        scroll_axis,
        scroll_padding,
    );
    state.update_scrollbar_state(
        builder,
        item_count,
        main_axis_size,
        cross_axis_size,
        scroll_axis,
    );
    viewport
}

/// Renders a widget into `buf`, clipping it if partially visible.
fn render_clipped<T: Widget>(
    item: T,
    available_area: Rect,
    buf: &mut Buffer,
    untruncated_size: u16,
    truncation: &Truncation,
    base_style: Style,
    scroll_axis: ScrollAxis,
) {
    if truncation.value() == 0 {
        item.render(available_area, buf);
        return;
    }

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
    }
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

    fn test_data(total_height: u16) -> (Rect, Buffer, ListView<'static, TestItem>, ListState) {
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
