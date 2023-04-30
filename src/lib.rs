//! # Widget list for TUI
pub mod widget;
pub use widget::{ListableWidget, WidgetList, WidgetListState};

/// [`SelectableWidgetList`] is a convenience method for [`WidgetList`].
/// It provides the methods next and previos to easily choose between
/// widgets in the list.
///
/// # Examples
/// ```
/// use ratatui::buffer::Buffer;
/// use ratatui::layout::Rect;
/// use ratatui::style::{Color, Style};
/// use ratatui::text::Text;
/// use ratatui::widgets::{Paragraph, Widget};
/// use tui_widget_list::ListableWidget;
///
/// #[derive(Debug, Clone)]
/// pub struct MyWidgetItem<'a> {
///     item: Paragraph<'a>,
///     height: u16,
/// }
///
/// impl MyWidgetItem<'_> {
///     pub fn new(text: &'static str, height: u16) -> Self {
///         let item = Paragraph::new(Text::from(text));
///         Self { item, height }
///     }
/// }
///
/// impl<'a> Widget for MyWidgetItem<'a> {
///     fn render(self, area: Rect, buf: &mut Buffer) {
///         self.item.render(area, buf);
///     }
/// }
///
/// impl<'a> ListableWidget for MyWidgetItem<'a> {
///     fn height(&self) -> u16 {
///         self.height
///     }
///
///     fn highlight(mut self) -> Self {
///         self.item = self.item.style(Style::default().bg(Color::White));
///         self
///     }
/// }
/// ```
#[derive(Clone, Default)]
pub struct SelectableWidgetList<T> {
    /// Holds the lists state, i.e. which element is selected.
    pub state: WidgetListState,

    /// The items of the list.
    pub items: Vec<T>,

    /// Whether the selection is circular. If true, calling next on the
    /// last element returns the first element, and calling previous on
    /// the first element returns the last element.
    pub circular: bool,
}

impl<T: ListableWidget> SelectableWidgetList<T> {
    /// Returns a [`SelectableWidgetList`]. The items elements
    /// must implement [`ListableWidget`].
    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            state: WidgetListState::default(),
            items,
            circular: true,
        }
    }

    /// Set circular. When circular is True, the selection continues
    /// from the first item in the list after reaching the last item.
    pub fn set_circular(mut self, circular: bool) -> Self {
        self.circular = circular;
        self
    }

    /// Selects the next element in the list. If circular is true,
    /// a call to next on the last element selects the first.
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

    /// Selects the previous element in the list. If circular is true,
    /// a call to previous on the first element selects the last.
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
