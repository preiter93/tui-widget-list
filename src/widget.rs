use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

#[derive(Debug, Clone, Default)]
pub struct WidgetListState {
    /// The fist item on the screen
    offset: usize,

    /// The selected item
    selected: Option<usize>,
}

impl WidgetListState {
    /// Return the currently selected items index
    #[must_use]
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Select an item by its index
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        if index.is_none() {
            self.offset = 0;
        }
    }

    /// Here we check and if necessary update the viewport.
    /// For this we start with the first item on the screen and iterate
    /// until we have reached the maximum height. If the selected value
    /// is within the bounds we do nothing. If the selected value is out
    /// of bounds, we adjust the offset accordingly.
    fn update_offset<I>(&mut self, heights: I, max_height: u16)
    where
        I: Iterator<Item = u16> + ExactSizeIterator + DoubleEndedIterator + Clone,
    {
        // We can return early if no value is selected
        let selected = match self.selected {
            Some(selected) => selected,
            None => return,
        };
        let heights: Vec<u16> = heights.collect();
        let offset = self.offset;

        // If the selected value is smaller then the offset, we roll
        // the offset so that the selected value is at the top
        if selected < offset {
            self.offset = selected;
            return;
        }

        // Check if the selected item is in the current view
        let (mut y, mut i) = (0, offset);
        for height in heights.iter().skip(offset) {
            // Out of bounds
            if y + height > max_height {
                break;
            }
            // Selected value is within view/bounds, so we are good
            if selected <= i {
                return;
            }
            y += height;
            i += 1;
        }

        // The selected item is out of bounds. We iterate backwards from the selected
        // item and determine the first widget that still fits on the screen.
        let (mut y, mut i) = (0, selected);
        let last_elem = heights.len() - 1;
        for height in heights.iter().rev().skip(last_elem - selected) {
            // out of bounds
            if y + height > max_height {
                break;
            }
            y += height;
            i -= 1;
        }

        // Update the offset
        self.offset = i + 1;
    }
}

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
}

impl<'a, T> Default for WidgetList<'a, T> {
    fn default() -> Self {
        Self {
            items: vec![],
            style: Style::default(),
            block: None,
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

        // Use the full width
        let width = area.width;

        // Maximum height
        let max_height = area.height;

        // The starting positions of the current item
        let x = area.left();
        let y0 = area.top();
        let mut y = y0;

        // Update the offset which might have changed between two frames
        state.update_offset(self.items.iter().map(|item| item.height), max_height);

        // Iterate over all items
        let mut i = state.offset;
        for item in self.items.into_iter().skip(state.offset) {
            // Get the area of the item that we draw
            let height = item.height;
            if y - y0 + height > max_height {
                // Out of bounds
                break;
            }
            let area = Rect::new(x, y, width, height);

            // Render the widget
            let is_selected = state.selected().map(|selected| selected == i);
            (item.modify_fn)(item, is_selected).render(area, buf);

            // Update the vertical offset
            y += height;
            i += 1;
        }
    }
}
