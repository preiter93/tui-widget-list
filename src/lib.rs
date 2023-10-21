//! # Widget list implementation for TUI
//!
//! ## Configurations
//! The [`SelectableWidgetList`] can be modified
//! - **style**: The base style of the list.
//! - **block**: An optional outer block around the list.
//! - **circular**: Whether the selection is circular, i.e. if true the first element will be selected after the last. True by default.
//! - **truncate**: If truncate is true, the first and last element will be truncated to fill the full-screen. True by default.
//!
//! ## Demos
//! See `examples/paragraph_list` and `examples/simple_list` in [tui-widget-list](https://github.com/preiter93/tui-widget-list/tree/main/examples).
//!
//! ## Examples
//! Use a custom widget within a [`SelectableWidgetList`].
//! ```
//! use ratatui::buffer::Buffer;
//! use ratatui::layout::Rect;
//! use ratatui::style::{Color, Style};
//! use ratatui::text::Text;
//! use ratatui::widgets::{Paragraph, Widget};
//! use tui_widget_list::{WidgetListItem, SelectableWidgetList};
//!
//! #[derive(Debug, Clone)]
//! pub struct MyListItem<'a> {
//!     content: Paragraph<'a>,
//!     height: u16,
//! }
//!
//! impl MyListItem<'_> {
//!     pub fn new(text: &'static str, height: u16) -> Self {
//!         let content = Paragraph::new(Text::from(text));
//!         Self { content, height }
//!     }
//!
//!     pub fn style(mut self, style: Style) -> Self {
//!         self.content = self.content.style(style);
//!         self
//!     }
//!
//!     // Render the item differently depending on the selection state
//!     fn modify_fn(mut item: WidgetListItem<Self>, selected: Option<bool>) -> WidgetListItem<Self> {
//!         if let Some(selected) = selected {
//!             if selected {
//!                 let style = Style::default().bg(Color::White);
//!                 item.content = item.content.style(style);
//!                 // You can also change the height of the selected item
//!                 // item.height = 5_u16;
//!             }
//!         }
//!         item
//!     }
//! }
//!
//! /// Must implement the `Widget` trait.
//! impl<'a> Widget for MyListItem<'a> {
//!     fn render(self, area: Rect, buf: &mut Buffer) {
//!         self.content.render(area, buf);
//!     }
//! }
//!
//! /// Define a method to cast to a `WidgetListItem`
//! impl<'a> From<MyListItem<'a>> for WidgetListItem<MyListItem<'a>> {
//!     fn from(val: MyListItem<'a>) -> Self {
//!         let height = 1_u16; // Assume we have a one line paragraph
//!         Self::new(val, height).modify_fn(MyListItem::modify_fn)
//!     }
//! }
//!
//! let items = vec![
//!     MyListItem::new("hello", 3),
//!     MyListItem::new("world", 4),
//! ];
//! let widget_list = SelectableWidgetList::new(items);
//!
//! // widget_list can be rendered like any other widget in TUI. Note that
//! // we pass it as mutable reference in order to not lose the state.
//! // f.render_widget(&mut widget_list, area);
//! ```
pub mod selectable;
pub mod widget;
pub use selectable::SelectableWidgetList;
pub use widget::{WidgetList, WidgetListItem, WidgetListState};
