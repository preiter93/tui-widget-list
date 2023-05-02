use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

#[derive(Debug, Clone, Default)]
pub struct WidgetListState {
    /// The index of the fist item on the screen
    offset: usize,

    /// The height of each visible item.
    view_heights: Vec<u16>,

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
    fn update_offset<I>(&mut self, heights: I, max_height: u16, truncate: bool)
    where
        I: Iterator<Item = u16> + ExactSizeIterator + DoubleEndedIterator + Clone,
    {
        let heights: Vec<u16> = heights.collect();

        // Select the first element if none is selected
        let selected = self.selected.unwrap_or(0);

        // If the selected value is smaller then the offset, we roll
        // the offset so that the selected value is at the top
        if selected < self.offset {
            self.offset = selected;
        }

        // The items heights on the viewport will be calculated on the fly.
        self.view_heights.clear();

        // Check if the selected item is in the current view
        let (mut y, mut i) = (0, self.offset);
        let mut found = false;
        for height in heights.iter().skip(self.offset) {
            // Out of bounds
            if y + height > max_height {
                if found {
                    if truncate {
                        // Truncate the last widget to fit into the view
                        self.view_heights.push(max_height - y);
                    }
                    return;
                }
                break;
            }
            // Selected value is within view/bounds, so we are good
            if selected == i {
                found = true;
            }
            y += height;
            i += 1;
            self.view_heights.push(*height);
        }

        // The selected item is out of bounds. We iterate backwards from the selected
        // item and determine the first widget that still fits on the screen.
        self.view_heights.clear();
        let (mut y, mut i) = (0, selected);
        let last_elem = heights.len() - 1;
        for height in heights.iter().rev().skip(last_elem - selected) {
            // out of bounds
            if y + height > max_height {
                if truncate {
                    // Truncate the first widget to fit into the view.
                    // At the moment this will truncate the bottom of the first item, which
                    // looks a bit strange, but I have not figured out how to truncate a
                    // widget from the top.
                    self.view_heights.insert(0, max_height - y);
                    self.offset = i;
                } else {
                    self.offset = i + 1;
                }
                break;
            }
            self.view_heights.insert(0, *height);
            y += height;
            i -= 1;
        }
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

        // Use the full width
        let width = area.width;

        // Maximum height
        let max_height = area.height;

        // The starting positions of the current item
        let x = area.left();
        let y0 = area.top();
        let mut y = y0;

        // Update the offset which might have changed between two frames
        state.update_offset(
            self.items.iter().map(|item| item.height),
            max_height,
            self.truncate,
        );

        // Iterate over all items
        let first = state.offset;
        let n = state.view_heights.len();
        for (i, item) in self.items.into_iter().skip(first).take(n).enumerate() {
            // Set the drawing area of the current item
            let height = state.view_heights.get(i).unwrap();
            let area = Rect::new(x, y, width, *height);

            // Render the item
            let is_selected = state.selected().map(|selected| selected == i + first);
            (item.modify_fn)(item, is_selected).render(area, buf);

            // Update the offset
            y += height;
        }
    }
}
