//! # A versatile list implementation for Ratatui
//!
//! This crate offers a stateful widget list implementation [`List`] for `Ratatui` that allows to
//! work with any list of widgets that implement to the [`Listable`] trait. The associated selection state
//! is [`ListState`] which offers methods like next and previous.
//!
//! ## Examples
//! ```
//! use ratatui::buffer::Buffer;
//! use ratatui::layout::Rect;
//! use ratatui::style::{Color, Style};
//! use ratatui::text::Text;
//! use ratatui::widgets::{Paragraph, Widget};
//! use ratatui::Frame;
//! use tui_widget_list::{List, ListState, Listable};
//!
//! #[derive(Debug, Clone)]
//! pub struct MyListItem {
//!     text: String,
//!     style: Style,
//!     height: usize,
//! }
//!
//! impl MyListItem {
//!     pub fn new(text: &str, height: usize) -> Self {
//!         Self {
//!             text: text.to_string(),
//!             style: Style::default(),
//!             height,
//!         }
//!     }
//!
//!     pub fn style(mut self, style: Style) -> Self {
//!         self.style = style;
//!         self
//!     }
//! }
//!
//! impl Listable for MyListItem {
//!     fn height(&self) -> usize {
//!         self.height
//!     }
//!
//!     fn highlight(self) -> Self {
//!         self.style(Style::default().bg(Color::Cyan))
//!     }
//! }
//!
//! impl Widget for MyListItem {
//!     fn render(self, area: Rect, buf: &mut Buffer) {
//!         Paragraph::new(Text::from(self.text))
//!             .style(self.style)
//!             .render(area, buf);
//!     }
//! }
//!
//! pub fn render(f: &mut Frame) {
//!     let list = List::new(vec![
//!         MyListItem::new("hello", 1),
//!         MyListItem::new("world", 2),
//!     ]);
//!     let mut state = ListState::default();
//!     f.render_stateful_widget(list, f.size(), &mut state);
//! }
//! ```
//!
//! For more examples see [tui-widget-list](https://github.com/preiter93/tui-widget-list/tree/main/examples).
//!
//! ## Configuration
//! The appearance of [`List`] can be modified
//! - **style**: The base style of the list.
//! - **block**: An optional outer block around the list.
//! - **truncate**: If truncate is true, the first and last elements are truncated to fill the entire screen. True by default.
//!
//! The behaviour of [`ListState`] can be modified
//! - **circular**: Whether the selection is circular, i.e. if true, the first item is selected after the last. True by default.
//!
//!![](img/demo.gif)
pub mod state;
pub mod traits;
pub mod widget;
pub use state::ListState;
pub use traits::Listable;
pub use widget::List;
