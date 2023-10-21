use ratatui::{
    prelude::{Buffer, Rect},
    style::Style,
    widgets::{Block, Widget},
};

use crate::{WidgetItem, WidgetListState};

/// A [`WidgetList`] is a widget that can be used in Ratatui to
/// render an arbitrary list of widgets. It is generic over
/// T, where each T should implement the [`WidgetItem`] trait.
#[derive(Clone)]
pub struct WidgetList<'a, T: WidgetItem> {
    /// The lists items.
    pub items: Vec<T>,

    /// The lists state.
    pub state: WidgetListState,

    /// Style used as a base style for the widget.
    style: Style,

    /// Block surrounding the widget list.
    block: Option<Block<'a>>,

    /// Truncate widgets to fill full screen. Defaults to true.
    truncate: bool,

    /// Whether the selection is circular. If true, calling next on the
    /// last element returns the first element, and calling previous on
    /// the first element returns the last element.
    circular: bool,
}

impl<'a, T: WidgetItem> WidgetList<'a, T> {
    /// Instantiate a widget list with elements. The Elements must
    /// implement the [`WidgetItem`] trait.
    #[must_use]
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            style: Style::default(),
            block: None,
            truncate: true,
            circular: true,
            state: WidgetListState::default(),
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

    /// If circular is True, the selection continues from the
    /// last item to the first when going down, and from the
    /// first item to the last when going up.
    /// It is true by default.
    #[must_use]
    pub fn circular(mut self, circular: bool) -> Self {
        self.circular = circular;
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

    /// Selects the next element of the list. If circular is true,
    /// calling next on the last element selects the first.
    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    if self.circular {
                        0
                    } else {
                        i
                    }
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Selects the previous element of the list. If circular is true,
    /// calling previous on the first element selects the last.
    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.circular {
                        self.items.len() - 1
                    } else {
                        i
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl<'a, T: WidgetItem> From<Vec<T>> for WidgetList<'a, T> {
    /// Instantiates a [`WidgetList`] from a vector of elements implementing
    /// the [`WidgetList`] trait.
    fn from(items: Vec<T>) -> Self {
        Self::new(items)
    }
}

impl<'a, T: WidgetItem> Widget for &mut WidgetList<'a, T> {
    // Renders a mutable reference to a widget list
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Set the base style
        buf.set_style(area, self.style);
        let area = match self.block.as_ref() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.clone().render(area, buf);
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
        let max_height = area.height as usize;

        // The starting positions of the current item
        let x = area.left();
        let y0 = area.top();
        let mut y = y0;

        // Modify the widgets based on their selection state. Split out their heights for
        // efficiency as we have to iterate over the heights back and forth to determine
        // which widget is shown on the viewport.
        let mut highlighted_item: Option<T> = None;
        let raw_heights: Vec<_> = self
            .items
            .iter_mut()
            .enumerate()
            .map(|(i, item)| {
                if self.state.selected().is_some_and(|s| s == i) {
                    let item = item.highlighted();
                    let height = item.height();
                    highlighted_item = Some(item);
                    height
                } else {
                    item.height()
                }
            })
            .collect();

        // Determine which widgets to render and how much space they get assigned to.
        let view_heights = self
            .state
            .update_view_port(&raw_heights, max_height, self.truncate);

        // Iterate over the modified items
        let offset = self.state.offset;
        for (i, height) in view_heights.into_iter().enumerate() {
            let area = Rect::new(x, y, width, height as u16);
            let selected = self.state.selected().is_some_and(|s| s == i + offset);
            match (selected, highlighted_item.as_ref()) {
                (true, Some(item)) => item.render(area, buf),
                _ => self.items[i + offset].render(area, buf),
            }
            y += height as u16;
        }
    }
}
