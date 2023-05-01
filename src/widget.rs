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

/// Base requirements for a Widget used in a `WidgetList`.
pub trait ListableWidget: Widget {
    /// The height of the widget.
    fn height(&self) -> u16;

    /// Highlight the selected widget
    #[must_use]
    fn highlight(self) -> Self;
}

pub struct WidgetList<'a, T> {
    /// The lists items
    items: Vec<T>,

    /// Style used as a base style for the widget
    style: Style,

    /// Block surrounding the widget list
    block: Option<Block<'a>>,
}

impl<'a, T: ListableWidget> WidgetList<'a, T> {
    /// Instantiate a widget list with elements. The Elements must
    /// implement [`ListableWidget`]
    #[must_use]
    pub fn new(items: Vec<T>) -> Self {
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
}

impl<'a, T: ListableWidget> StatefulWidget for WidgetList<'a, T> {
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
        state.update_offset(self.items.iter().map(ListableWidget::height), max_height);

        // Iterate over all items
        let mut i = state.offset;
        for item in self.items.into_iter().skip(state.offset) {
            // Get the area of the item that we draw
            let height = item.height();
            if y - y0 + height > max_height {
                // Out of bounds
                break;
            }
            let area = Rect::new(x, y, width, height);

            // Render the widget
            let mut widget = item;
            if let Some(selected) = state.selected() {
                if selected == i {
                    widget = widget.highlight();
                }
            }
            widget.render(area, buf);

            // Update the vertical offset
            y += height;
            i += 1;
        }
    }
}
