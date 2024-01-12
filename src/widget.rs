use ratatui::{
    prelude::{Buffer, Rect},
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{ListState, Listable};

/// A [`List`] is a widget that can be used in Ratatui to
/// render an arbitrary list of widgets. It is generic over
/// T, where each T should implement the [`Listable`] trait.
#[derive(Clone)]
pub struct List<'a, T: Listable> {
    /// The lists items.
    pub items: Vec<T>,

    /// Style used as a base style for the widget.
    style: Style,

    /// Block surrounding the widget list.
    block: Option<Block<'a>>,

    /// Truncate widgets to fill full screen. Defaults to true.
    truncate: bool,
}

impl<'a, T: Listable> List<'a, T> {
    /// Instantiate a widget list with elements. The Elements must
    /// implement the [`WidgetItem`] trait.
    #[must_use]
    pub fn new(items: Vec<T>) -> Self {
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

impl<'a, T: Listable> From<Vec<T>> for List<'a, T> {
    /// Instantiates a [`List`] from a vector of elements implementing
    /// the [`Listable`] trait.
    fn from(items: Vec<T>) -> Self {
        Self::new(items)
    }
}

impl<'a, T: Listable> StatefulWidget for List<'a, T> {
    type State = ListState;
    // Renders a mutable reference to a widget list
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut items = self.items;
        let mut block = self.block;
        state.set_num_elements(items.len());

        // Set the base style
        buf.set_style(area, self.style);
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

        // Use the full width
        let width = area.width;

        // Maximum height
        let max_height = area.height as usize;

        // The starting positions of the current item
        let x = area.left();
        let y0 = area.top();
        let mut y = y0;

        // Split out the highlighted item
        let mut highlighted: Option<T> = None;
        if let Some(index) = state.selected() {
            if index < items.len() {
                highlighted = Some(items.remove(index).highlight());
            }
        }

        // Modify the widgets based on their selection state. Split out their heights
        // for efficiency as we have to iterate over the heights back and forth to
        // determine which widget is shown on the viewport.
        let mut raw_heights: Vec<_> = items.iter().map(Listable::height).collect();

        // Insert the height of the highlighted item back in
        if let (Some(index), Some(h)) = (state.selected, &mut highlighted) {
            raw_heights.insert(index, h.height());
        }

        // Determine which widgets to show on the viewport and how much space they
        // get assigned to. The number of elements in `view_heights` is less than
        // the number of elements in `raw_heights` if not all widgets are shown
        // on the viewport.
        let view_heights = state.update_view_port(&raw_heights, max_height, self.truncate);

        // Drain out elements that are shown on the view port from the vector of
        // all elements.
        let first = state.offset;
        let mut last = view_heights.len() + first;
        if highlighted.is_some() {
            last = last.saturating_sub(1);
        }
        let mut view_items = items.drain(first..last);

        // Iterate over the modified items
        let offset = state.offset;
        let num_items = view_heights.len();
        for (i, height) in view_heights.into_iter().enumerate() {
            let area = Rect::new(x, y, width, height as u16);
            if state.selected().is_some_and(|s| s == i + offset) {
                if let Some(item) = highlighted.take() {
                    render_item(item, area, buf, i, num_items);
                }
            } else if let Some(item) = view_items.next() {
                render_item(item, area, buf, i, num_items);
            }
            y += height as u16;
        }
    }
}

fn render_item<T: Listable>(item: T, area: Rect, buf: &mut Buffer, pos: usize, num_items: usize) {
    // Check if the first element is truncated and needs special handling
    let item_height = item.height() as u16;
    if pos == 0 && num_items > 1 && area.height < item_height {
        // Create an intermediate buffer for rendering the truncated element
        let mut hidden_buffer = Buffer::empty(Rect {
            x: area.left(),
            y: area.top(),
            width: area.width,
            height: item_height,
        });
        item.render(hidden_buffer.area, &mut hidden_buffer);

        // Copy the visible part from the intermediate buffer to the main buffer
        let offset = item_height.saturating_sub(area.height);
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                *buf.get_mut(x, y) = hidden_buffer.get(x, y + offset).clone();
            }
        }
    } else {
        item.render(area, buf);
    }
}
