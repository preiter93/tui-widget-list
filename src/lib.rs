//! # Widget list for TUI
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

    /// Here we check and if necessary update the `y_offset` value.
    /// For this we start with the first item on the screen and iterate
    /// until we have reached the maximum height. If the selected value
    /// is within the bounds we do nothing. If the selected value is out
    /// of bounds, we adjust the offset accordingly.
    pub fn update_offset<I>(&mut self, heights: I, max_height: u16)
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

        // The starting pos of the current widget
        let (mut y, mut i) = (0, offset);
        for height in heights.iter().skip(offset) {
            // Out of bounds
            if y + height > max_height {
                break;
            }
            // Selected value is within view/bounds
            if selected <= i {
                return;
            }
            y += height;
            i += 1;
        }

        // The selected value is out of bounds. We can determine the first item
        // that fits on the screen by iterating backwards
        let (mut y, mut i) = (0, selected);
        for height in heights.iter().rev().skip(heights.len() - selected - 1) {
            y += height;
            i -= 1;
            // out of bounds
            if y + height > max_height {
                break;
            }
        }

        // Update the offset
        self.offset = i + 1;
    }
}

pub trait ListableWidget {
    /// The height of the widget.
    fn height(&self) -> u16;

    /// Draws the widget.
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// Draws the widget in its selected state.
    fn render_selected(&self, area: Rect, buf: &mut Buffer);
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
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            style: Style::default(),
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

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
        let mut y = area.top();

        // Update the offset which might have changed between two frames
        state.update_offset(self.items.iter().map(|item| item.height()), max_height);

        // Iterate over all items
        let mut i = state.offset;
        for item in self.items.iter().skip(state.offset) {
            // Get the area of the item that we draw
            let height = item.height();
            if y + height > max_height {
                // Out of bounds
                break;
            }
            let area = Rect::new(x, y, width, height);

            // Render the content
            let is_selected = state.selected().map_or(false, |s| i == s);
            if is_selected {
                item.render_selected(area, buf);
            } else {
                item.render(area, buf);
            }

            // Update the vertical offset
            y += height;
            i += 1;
        }
    }
}
