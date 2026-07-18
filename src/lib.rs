//!<div align="center">
//!
//! # tui-widget-list: A versatile widget list for Ratatui
//!
//! [![Crate Badge]](https://crates.io/crates/tui-widget-list) [![Continuous Integration](https://github.com/preiter93/tui-widget-list/actions/workflows/ci.yml/badge.svg)](https://github.com/preiter93/tui-widget-list/actions/workflows/ci.yml) [![Deps Status](https://deps.rs/repo/github/preiter93/tui-widget-list/status.svg)](https://deps.rs/repo/github/preiter93/tui-widget-list) [![License Badge]](./LICENSE)
//!
//! </div>
//!
//! This crate provides a stateful `ListView` widget for `Ratatui`.
//! The associated `ListState` handles navigation.
//! The list view supports both horizontal and vertical scrolling.
//!
//!![](https://raw.githubusercontent.com/preiter93/tui-widget-list/main/examples/tapes/variants.gif?v=1)
//!
//! ## Configuration
//! `ListView` can be customized with the following options:
//! - `ListView::scroll_axis`: Scroll vertically or horizontally.
//! - `ListView::scroll_direction`: Lay items out forward or in reverse.
//! - `ListView::scroll_padding`: Keep a number of items visible around the selected item while scrolling.
//! - `ListView::infinite_scrolling`: Wrap around when scrolling past the first or last item.
//! - `ListView::style`: Base style applied to the entire list.
//! - `ListView::block`: Optional surrounding block.
//! - `ListView::scrollbar`: Optional scrollbar overlay.
//!
//! ## Example
//!```
//! use ratatui::prelude::*;
//! use tui_widget_list::{ListBuilder, ListState, ListView};
//!
//! let builder = ListBuilder::new(|context| {
//!     let mut item = Line::from(format!("Item {}", context.index));
//!     if context.is_selected {
//!         item = item.style(Style::default().bg(Color::Rgb(255, 153, 0)));
//!     }
//!     (item, 1)
//! });
//!
//! let mut state = ListState::default();
//! let list = ListView::new(builder, 20);
//!```
//!
//! ## Mouse handling
//!
//! Handle mouse clicks and scroll events using `ListState::hit_test`:
//!```ignore
//! use tui_widget_list::hit_test::Hit;
//!
//! match event::read()? {
//!     Event::Mouse(MouseEvent {
//!         kind: MouseEventKind::Down(MouseButton::Left),
//!         column,
//!         row,
//!         ..
//!     }) => match state.hit_test(column, row) {
//!         Some(Hit::Item(index)) => state.select(Some(index)),
//!         Some(Hit::Area) | None => {}
//!     },
//!     Event::Mouse(MouseEvent {
//!         kind: MouseEventKind::ScrollUp,
//!         ..
//!     }) => {
//!         state.previous();
//!     }
//!     Event::Mouse(MouseEvent {
//!         kind: MouseEventKind::ScrollDown,
//!         ..
//!     }) => {
//!         state.next();
//!     }
//!     _ => {}
//! }
//!```
//!
//! ## Documentation
//! For more examples see the [examples directory](https://github.com/preiter93/tui-widget-list/tree/main/examples). Full API documentation is available on [docs.rs](https://docs.rs/tui-widget-list/).
//!
//! [Crate Badge]: https://img.shields.io/crates/v/tui-widget-list?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44
//! [License Badge]: https://img.shields.io/crates/l/tui-widget-list?style=flat-square&color=1370D3
pub mod hit_test;
pub(crate) mod state;
pub(crate) mod utils;
pub(crate) mod view;

pub use state::ListState;
pub use view::{ListBuildContext, ListBuilder, ListView, ScrollAxis, ScrollDirection};
