//! # Widget list implementation for TUI
//!
//! # Demo
//! ```ignore
//! cargo run --example paragraph_list
//! ```
//!
//! # Examples
//! Items of [`WidgetList`] or of the convenience class [`SelectableWidgetList`]
//! must implement the [`ListableWidget`] trait. Then the render() method is available
//! on the widget list.
//!
//! ```
//! use ratatui::buffer::Buffer;
//! use ratatui::layout::Rect;
//! use ratatui::style::{Color, Style};
//! use ratatui::text::Text;
//! use ratatui::widgets::{Paragraph, Widget};
//! use tui_widget_list::{ListableWidget, SelectableWidgetList};
//!
//! #[derive(Debug, Clone)]
//! pub struct MyWidgetItem<'a> {
//!     item: Paragraph<'a>,
//!     height: u16,
//! }
//!
//! impl MyWidgetItem<'_> {
//!     pub fn new(text: &'static str, height: u16) -> Self {
//!         let item = Paragraph::new(Text::from(text));
//!         Self { item, height }
//!     }
//!
//!     // Render the item differently depending on the selection state
//!     fn modify_fn(mut self, is_selected: Option<bool>) -> Self {
//!         if let Some(selected) = is_selected {
//!             if selected {
//!                 let style = Style::default().bg(Color::White);
//!                 self.item = self.item.style(style);
//!             }
//!         }
//!         self
//!     }
//! }
//!
//! impl<'a> Widget for MyWidgetItem<'a> {
//!     fn render(self, area: Rect, buf: &mut Buffer) {
//!         self.item.render(area, buf);
//!     }
//! }
//!
//! impl<'a> ListableWidget for MyWidgetItem<'a> {
//!     fn height(&self) -> u16 {
//!         self.height
//!     }
//! }
//!
//!
//! let items = vec![
//!     MyWidgetItem::new("hello", 3),
//!     MyWidgetItem::new("world", 4),
//! ];
//! let widget_list = SelectableWidgetList::new(items).modify_fn(MyWidgetItem::modify_fn);
//!
//! ```
pub mod widget;
use ratatui::{style::Style, widgets::Block};
pub use widget::{ListableWidget, ModifyFn, WidgetList, WidgetListState};

/// [`SelectableWidgetList`] is a convenience method for [`WidgetList`].
/// It provides the methods next and previous to conveniently select
/// widgets of the list.
#[derive(Clone, Default)]
pub struct SelectableWidgetList<'a, T> {
    /// Holds the lists state, i.e. which element is selected.
    pub state: WidgetListState,

    /// A list of widgets.
    pub content: WidgetList<'a, T>,

    /// Whether the selection is circular. If true, calling next on the
    /// last element returns the first element, and calling previous on
    /// the first element returns the last element.
    circular: bool,
}

impl<'a, T: ListableWidget> SelectableWidgetList<'a, T> {
    /// Returns a [`SelectableWidgetList`]. The items elements
    /// must implement [`ListableWidget`].
    #[must_use]
    pub fn new(items: Vec<T>) -> Self {
        Self {
            state: WidgetListState::default(),
            content: WidgetList::new(items),
            circular: true,
        }
    }

    /// Use circular selection. When circular is True, the selection continues
    /// from the last item to the first, and vice versa.
    #[must_use]
    pub fn circular(mut self, circular: bool) -> Self {
        self.circular = circular;
        self
    }

    /// Set the block style which surrounds the whole List.
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.content = self.content.block(block);
        self
    }

    /// Set the base style of the List.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.content = self.content.style(style);
        self
    }

    /// Set a callback that can be used to modify the widget item
    /// based on the selection state.
    #[must_use]
    pub fn modify_fn(mut self, modify_fn: ModifyFn<T>) -> Self {
        self.content = self.content.modify_fn(modify_fn);
        self
    }

    /// Selects the next element in the list. If circular is true,
    /// calling next on the last element selects the first.
    pub fn next(&mut self) {
        if self.content.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.content.len() - 1 {
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

    /// Selects the previous element in the list. If circular is true,
    /// calling previous on the first element selects the last.
    pub fn previous(&mut self) {
        if self.content.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    if self.circular {
                        self.content.len() - 1
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
