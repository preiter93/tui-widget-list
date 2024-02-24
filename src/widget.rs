#![allow(clippy::cast_possible_truncation)]
use ratatui::{
    prelude::{Buffer, Rect},
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{ListState, ListableWidget};

/// A [`List`] is a widget for Ratatui that can render an arbitrary list of widgets.
/// It is generic over `T`, where each widget `T` should implement the [`ListableWidget`]
/// trait.
#[derive(Clone)]
pub struct List<'a, T: ListableWidget> {
    /// The list's items.
    pub items: Vec<T>,

    /// Style used as a base style for the widget.
    style: Style,

    /// Block surrounding the widget list.
    block: Option<Block<'a>>,

    /// Truncate widgets to fill the full screen. Defaults to true.
    truncate: bool,

    /// Specifies the scroll direction.
    scroll_direction: ScrollAxis,
}

impl<'a, T: ListableWidget> List<'a, T> {
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
            truncate: true,
            scroll_direction: ScrollAxis::default(),
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

    /// If `truncate` is true, the list fills the full screen and truncates
    /// the first or last item of the list. It is true by default.
    #[must_use]
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
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
    pub fn scroll_direction(mut self, scroll_direction: ScrollAxis) -> Self {
        self.scroll_direction = scroll_direction;
        self
    }
}

impl<'a, T: ListableWidget> From<Vec<T>> for List<'a, T> {
    /// Instantiates a [`List`] from a vector of elements implementing
    /// the [`ListableWidget`] trait.
    fn from(items: Vec<T>) -> Self {
        Self::new(items)
    }
}

impl<'a, T: ListableWidget> StatefulWidget for List<'a, T> {
    type State = ListState;
    /// Renders a mutable reference to a widget list
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let scroll_direction = self.scroll_direction;

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

        // Set the dimension along the scroll axis and the cross axis
        let (size_scroll_axis, size_cross_axis) = match scroll_direction {
            ScrollAxis::Vertical => (area.height as usize, area.width),
            ScrollAxis::Horizontal => (area.width as usize, area.height),
        };

        // The starting coordinates of the current item
        let (mut pos_scroll_axis, pos_cross_axis) = match scroll_direction {
            ScrollAxis::Vertical => (area.top(), area.left()),
            ScrollAxis::Horizontal => (area.left(), area.top()),
        };

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
        let mut raw_item_sizes: Vec<_> = items
            .iter()
            .map(|item| item.size(&scroll_direction))
            .collect();

        // Insert the height of the highlighted item back in
        if let (Some(index), Some(h)) = (state.selected, &mut highlighted) {
            raw_item_sizes.insert(index, h.size(&scroll_direction));
        }

        // Determine which widgets to show on the viewport and how much space they
        // get assigned to. The number of elements in `view_heights` is less than
        // the number of elements in `raw_heights` if not all widgets are shown
        // on the viewport.
        let sizes_scroll_direction =
            state.update_view_port(&raw_item_sizes, size_scroll_axis, self.truncate);

        // Drain out elements that are shown on the view port from the vector of
        // all elements.
        let first = state.offset;
        let mut last = sizes_scroll_direction.len() + first;
        if highlighted.is_some() {
            last = last.saturating_sub(1);
        }
        let mut items_in_view = items.drain(first..last);

        // Iterate over the modified items
        let offset = state.offset;
        let num_items = sizes_scroll_direction.len();
        for (i, size) in sizes_scroll_direction.into_iter().enumerate() {
            let area = match scroll_direction {
                ScrollAxis::Vertical => Rect::new(
                    pos_cross_axis,
                    pos_scroll_axis,
                    size_cross_axis,
                    size as u16,
                ),
                ScrollAxis::Horizontal => Rect::new(
                    pos_scroll_axis,
                    pos_cross_axis,
                    size as u16,
                    size_cross_axis,
                ),
            };
            if state.selected().is_some_and(|s| s == i + offset) {
                if let Some(item) = highlighted.take() {
                    render_item(item, area, buf, i, num_items, &scroll_direction);
                }
            } else if let Some(item) = items_in_view.next() {
                render_item(item, area, buf, i, num_items, &scroll_direction);
            }
            pos_scroll_axis += size as u16;
        }
    }
}

fn render_item<T: ListableWidget>(
    item: T,
    area: Rect,
    buf: &mut Buffer,
    pos: usize,
    num_items: usize,
    scroll_direction: &ScrollAxis,
) {
    // Check if the first element is truncated and needs special handling
    let item_size = item.size(scroll_direction) as u16;
    if pos == 0 && num_items > 1 && area.height < item_size {
        // Create an intermediate buffer for rendering the truncated element
        let (width, height) = match scroll_direction {
            ScrollAxis::Vertical => (area.width, item_size),
            ScrollAxis::Horizontal => (item_size, area.height),
        };
        let mut hidden_buffer = Buffer::empty(Rect {
            x: area.left(),
            y: area.top(),
            width,
            height,
        });
        item.render(hidden_buffer.area, &mut hidden_buffer);

        // Copy the visible part from the intermediate buffer to the main buffer
        match scroll_direction {
            ScrollAxis::Vertical => {
                let offset = item_size.saturating_sub(area.height);
                for x in area.left()..area.right() {
                    for y in area.top()..area.bottom() {
                        *buf.get_mut(x, y) = hidden_buffer.get(x, y + offset).clone();
                    }
                }
            }
            ScrollAxis::Horizontal => {
                let offset = item_size.saturating_sub(area.width);
                for x in area.left()..area.right() {
                    for y in area.top()..area.bottom() {
                        *buf.get_mut(x, y) = hidden_buffer.get(x + offset, y).clone();
                    }
                }
            }
        };
    } else {
        item.render(area, buf);
    }
}

/// Represents the scroll axis of a list.
#[derive(Default, Clone)]
pub enum ScrollAxis {
    /// Indicates vertical scrolling. This is the default.
    #[default]
    Vertical,

    /// Indicates horizontal scrolling.
    Horizontal,
}
