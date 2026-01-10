//!<div align="center">
//!
//! # A versatile widget list for Ratatui
//!
//! [![Crate Badge]](https://crates.io/crates/tui-widget-list) [![Continuous Integration](https://github.com/preiter93/tui-widget-list/actions/workflows/ci.yml/badge.svg)](https://github.com/preiter93/tui-widget-list/actions/workflows/ci.yml) [![Deps Status](https://deps.rs/repo/github/preiter93/tui-widget-list/status.svg)](https://deps.rs/repo/github/preiter93/tui-widget-list) [![License Badge]](./LICENSE)
//!
//! </div>
//!
//! This crate provides a stateful widget [`ListView`] implementation for `Ratatui`.
//! The associated [`ListState`], offers functionalities such as navigating to the next and previous items.
//! The list view support both horizontal and vertical scrolling.
//!
//! ## Configuration
//! The [`ListView`] can be customized with the following options:
//! - [`ListView::scroll_axis`]: Specifies whether the list is vertically or horizontally scrollable.
//! - [`ListView::scroll_padding`]: Specifies whether content should remain visible while scrolling, ensuring that a
//!   specified amount of padding is preserved above/below the selected item during scrolling.
//! - [`ListView::infinite_scrolling`]: Allows the list to wrap around when scrolling past the first or last element.
//! - [`ListView::style`]: Defines the base style of the list.
//! - [`ListView::block`]: Optional outer block surrounding the list.
//! - [`ListView::scrollbar`]: Optional scrollbar widget.
//!
//! ## Example
//!```
//! use ratatui::prelude::*;
//! use tui_widget_list::{ListBuilder, ListState, ListView};
//!
//! #[derive(Debug, Clone)]
//! pub struct ListItem {
//!     text: String,
//!     style: Style,
//! }
//!
//! impl ListItem {
//!     pub fn new<T: Into<String>>(text: T) -> Self {
//!         Self {
//!             text: text.into(),
//!             style: Style::default(),
//!         }
//!     }
//! }
//!
//! impl Widget for ListItem {
//!     fn render(self, area: Rect, buf: &mut Buffer) {
//!         Line::from(self.text).style(self.style).render(area, buf);
//!     }
//! }
//!
//! pub struct App {
//!     state: ListState,
//! }
//!```
//!
//! ## Mouse handling
//!
//! You can handle mouse clicks using `ListState` via `hit_test`:
//!```ignore
//! match event::read()? {
//!     Event::Mouse(MouseEvent {
//!         kind: MouseEventKind::Down(MouseButton::Left),
//!         column, row, ..
//!     }) => {
//!         if let Some(index) = hit_test(&state, column, row) {
//!             state.select(Some(index));
//!         }
//!     }
//!     Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollUp, .. }) => {
//!         state.previous();
//!     }
//!     Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollDown, .. }) => {
//!         state.next();
//!     }
//!     _ => {}
//! }
//!```
//!
//! For more examples see [tui-widget-list](https://github.com/preiter93/tui-widget-list/tree/main/examples).
//!
//! ## Documentation
//! [docs.rs](https://docs.rs/tui-widget-list/)
//!
//! ## Demo
//!
//! ### Infinite scrolling, scroll padding, horizontal scrolling
//!
//!![](examples/tapes/variants.gif?v=1)
//!
//! [Crate Badge]: https://img.shields.io/crates/v/tui-widget-list?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44
//! [License Badge]: https://img.shields.io/crates/l/tui-widget-list?style=flat-square&color=1370D3
pub(crate) mod hit_test;
pub(crate) mod legacy;
pub(crate) mod state;
pub(crate) mod utils;
pub(crate) mod view;

pub use hit_test::hit_test;
pub use state::ListState;
pub use view::{ListBuildContext, ListBuilder, ListView, ScrollAxis};

#[allow(deprecated)]
pub use legacy::{
    traits::{PreRender, PreRenderContext},
    widget::List,
};
