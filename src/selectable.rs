use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{WidgetList, WidgetListItem, WidgetListState};

/// [`SelectableWidgetList`] is a convenience method for [`WidgetList`].
/// It provides the next and previous method to select items and it
/// implements the [`Widget`] trait.
#[derive(Clone, Default)]
pub struct SelectableWidgetList<'a, T> {
    /// Holds the lists state, i.e. which element is selected.
    pub state: WidgetListState,

    /// The list of widgets.
    pub items: Vec<T>,

    /// Style used as a base style for the widget.
    style: Style,

    /// Block surrounding the widget list.
    block: Option<Block<'a>>,

    /// Whether the selection is circular. If true, calling next on the
    /// last element returns the first element, and calling previous on
    /// the first element returns the last element.
    circular: bool,

    /// If truncate is true, the widget list will render over the full-screen.
    /// In this case the first/last widget might be truncated.
    truncate: bool,
}

impl<'a, T> SelectableWidgetList<'a, T>
where
    T: Widget + Into<WidgetListItem<T>> + Clone,
{
    /// `items` must implement [`Widget`] and should be castable into [`WidgetListItem`].
    #[must_use]
    pub fn new(items: Vec<T>) -> Self {
        Self {
            state: WidgetListState::default(),
            items,
            style: Style::default(),
            block: None,
            circular: true,
            truncate: true,
        }
    }

    /// The base style of the list. Not the style of the list elements.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// The base block around the list. Must not be set.
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set circular selection. When circular is True, the selection continues
    /// from the last item to the first, and vice versa.
    #[must_use]
    pub fn circular(mut self, circular: bool) -> Self {
        self.circular = circular;
        self
    }

    /// If truncate is true, the widget list will render over the full-screen.
    /// In this case the first/last widget might be truncated.
    #[must_use]
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
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

    /// Returns a reference to the selected item.
    #[must_use]
    pub fn get_selected(&self) -> Option<&T> {
        self.state
            .selected()
            .and_then(|index| self.items.get(index))
    }

    /// Returns a mutable reference to the selected item.
    #[must_use]
    pub fn get_selected_mut(&mut self) -> Option<&mut T> {
        self.state
            .selected()
            .and_then(|index| self.items.get_mut(index))
    }
}

impl<T> Widget for &mut SelectableWidgetList<'_, T>
where
    T: Widget + Into<WidgetListItem<T>> + Clone,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items = self.items.iter().map(|x| x.clone().into()).collect();
        let mut widget = WidgetList::new(items)
            .style(self.style)
            .truncate(self.truncate);
        if let Some(block) = self.block.clone() {
            widget = widget.block(block);
        }

        widget.render(area, buf, &mut self.state);
    }
}
