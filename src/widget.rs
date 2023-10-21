use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::WidgetListState;

/// `WidgetListItem` holds the widget and the height of the widget.
#[derive(Clone)]
pub struct WidgetListItem<T> {
    /// The widget.
    pub content: T,

    /// The height of the widget.
    pub height: u16,

    /// A callback function that can be used to style an item
    /// based on its selection state.
    modify_fn: ModifyFn<Self>,
}

impl<T: Widget> WidgetListItem<T> {
    /// Constructs a new item given the widget and its height
    pub fn new(content: T, height: u16) -> Self {
        Self {
            content,
            height,
            modify_fn: default_modify_fn,
        }
    }

    /// Set a callback that can be used to modify the widget item
    /// based on the selection state.
    #[must_use]
    pub fn modify_fn(mut self, modify_fn: ModifyFn<Self>) -> Self {
        self.modify_fn = modify_fn;
        self
    }
}

impl<T: Widget> Widget for WidgetListItem<T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.content.render(area, buf);
    }
}

/// `ModifyFn` is a callback function that takes in the widget
/// and the current selection state and returns the (modified)
/// widget.
///
/// A selection state of None indicates that no other element
/// is selected. If the selection state is true, it indicates
/// that the current item is selected.
pub type ModifyFn<T> = fn(T, Option<bool>) -> T;

/// Default implementation of `modify_fn`. Does nothing to T.
fn default_modify_fn<T>(slf: T, _: Option<bool>) -> T {
    slf
}

#[derive(Clone)]
pub struct WidgetList<'a, T> {
    /// The lists items
    items: Vec<WidgetListItem<T>>,

    /// Style used as a base style for the widget
    style: Style,

    /// Block surrounding the widget list
    block: Option<Block<'a>>,

    /// Truncate widgets to fill full screen. Defaults to true.
    truncate: bool,
}

impl<'a, T> Default for WidgetList<'a, T> {
    fn default() -> Self {
        Self {
            items: vec![],
            style: Style::default(),
            block: None,
            truncate: true,
        }
    }
}

impl<'a, T: Widget> WidgetList<'a, T> {
    /// Instantiate a widget list with elements. The Elements must
    /// implement the [`Widget`] trait.
    #[must_use]
    pub fn new(items: Vec<WidgetListItem<T>>) -> Self {
        Self {
            items,
            style: Style::default(),
            block: None,
            truncate: true,
        }
    }

    /// Set the block style which surrounds the whole List.
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

    /// If truncate is true, the list fills the full screen
    /// and truncates the first or last item of the list.
    /// It is true by default.
    #[must_use]
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    /// Whether the widget list is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the length of the widget list
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<'a, T: Widget> StatefulWidget for WidgetList<'a, T> {
    type State = WidgetListState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Set the base style
        buf.set_style(area, self.style);
        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        // List is empty
        if self.is_empty() {
            return;
        }

        // Use the full width
        let width = area.width;

        // Maximum height
        let max_height = area.height;

        // The starting positions of the current item
        let x = area.left();
        let y0 = area.top();
        let mut y = y0;

        // Modify the widgets based on their selection state. Split out their heights for
        // efficiency as we have to iterate over the heights back and forth to determine
        // which widget is shown on the viewport.
        let (raw_heights, modified_items): (Vec<_>, Vec<_>) = self
            .items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = state.selected().map(|selected| selected == i);
                let item = (item.modify_fn)(item, is_selected);
                (item.height, item)
            })
            .unzip();

        // Determine which widgets to render and how much space they are assigned to.
        let view_heights = state.update_view_port(&raw_heights, max_height, self.truncate);

        // Iterate over the modified items
        let first = state.offset;
        let n = view_heights.len();
        for (item, height) in modified_items
            .into_iter()
            .skip(first)
            .take(n)
            .zip(view_heights)
        {
            let area = Rect::new(x, y, width, height);

            item.render(area, buf);

            y += height;
        }
    }
}
