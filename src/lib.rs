//! # Widget list implementation for TUI
//!
//! ## Configurations
//! The [`WidgetList`] can be modified
//! - **style**: The base style of the list.
//! - **block**: An optional outer block around the list.
//! - **circular**: Whether the selection is circular, i.e. if true the first element will be selected after the last. True by default.
//! - **truncate**: If truncate is true, the first and last element will be truncated to fill the full-screen. True by default.
//!
//! ## Demos
//! See `examples/paragraph_list` and `examples/simple_list` in [tui-widget-list](https://github.com/preiter93/tui-widget-list/tree/main/examples).
//!
//! ## Examples
//! ```
//! use ratatui::buffer::Buffer;
//! use ratatui::layout::Rect;
//! use ratatui::style::{Color, Style};
//! use ratatui::text::Text;
//! use ratatui::widgets::{Paragraph, Widget};
//! use tui_widget_list::{WidgetList, WidgetItem};
//!
//! #[derive(Debug, Clone)]
//! pub struct MyListItem<'a> {
//!     content: Paragraph<'a>,
//!     height: usize,
//! }
//!
//! impl MyListItem<'_> {
//!     pub fn new(text: &'static str, height: usize) -> Self {
//!         let content = Paragraph::new(Text::from(text));
//!         Self { content, height }
//!     }
//!
//!     pub fn style(mut self, style: Style) -> Self {
//!         self.content = self.content.style(style);
//!         self
//!     }
//! }
//!
//! impl<'a> WidgetItem for MyListItem<'a> {
//!     fn height(&self) -> usize {
//!         self.height
//!     }
//!
//!     fn highlighted(&self) -> Option<Self> {
//!         let mut highlighted = self.clone();
//!         Some(highlighted.style(Style::default().bg(Color::Cyan)))
//!     }
//!
//!     fn render(&self, area: Rect, buf: &mut Buffer) {
//!         self.clone().content.render(area, buf);
//!     }
//! }
//! // widget_list can be rendered like any other widget in TUI. Note that
//! // we pass it as mutable reference in order to not lose the state.
//! // f.render_widget(&mut widget_list, area);
//! ```
pub mod state;
pub mod traits;
pub mod widget;
pub use state::WidgetListState;
pub use traits::WidgetItem;
pub use widget::WidgetList;
