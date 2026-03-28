# tui-widget-list

<div align="center">

## A versatile widget list for Ratatui

[![Crate Badge]](https://crates.io/crates/tui-widget-list) [![Continuous Integration](https://github.com/preiter93/tui-widget-list/actions/workflows/ci.yml/badge.svg)](https://github.com/preiter93/tui-widget-list/actions/workflows/ci.yml) [![Deps Status](https://deps.rs/repo/github/preiter93/tui-widget-list/status.svg)](https://deps.rs/repo/github/preiter93/tui-widget-list) [![License Badge]](./LICENSE)

</div>

This crate provides a stateful widget [`ListView`] implementation for `Ratatui`.
The associated [`ListState`], offers functionalities such as navigating to the next and previous items.
The list view support both horizontal and vertical scrolling.

### Configuration
The [`ListView`] can be customized with the following options:
- [`ListView::scroll_axis`]: Vertical or horizontal scrolling.
- [`ListView::scroll_direction`]: Forward or backward layout direction.
- [`ListView::scroll_padding`]: Padding preserved around the selected item while scrolling.
- [`ListView::infinite_scrolling`]: Wrap around when scrolling past the first or last element.
- [`ListView::style`]: Base style of the list.
- [`ListView::block`]: Optional outer block.
- [`ListView::scrollbar`]: Optional scrollbar.

### Example
```rust
use ratatui::prelude::*;
use tui_widget_list::{ListBuilder, ListState, ListView};

let builder = ListBuilder::new(|context| {
    let mut item = Line::from(format!("Item {}", context.index));
    if context.is_selected {
        item = item.style(Style::default().bg(Color::Rgb(255, 153, 0)));
    }
    (item, 1)
});

let mut state = ListState::default();
let list = ListView::new(builder, 20);
```

### Mouse handling

You can handle mouse clicks using `ListState` via `hit_test`:
```rust
use tui_widget_list::hit_test::Hit;

match event::read()? {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        ..
    }) => match state.hit_test(column, row) {
        Some(Hit::Item(index)) => state.select(Some(index)),
        Some(Hit::Area) | None => {}
    },
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        ..
    }) => {
        state.previous();
    }
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        ..
    }) => {
        state.next();
    }
    _ => {}
}
```

For more examples see [tui-widget-list](https://github.com/preiter93/tui-widget-list/tree/main/examples).

### Demo

![](examples/tapes/variants.gif?v=1)

### Documentation
[docs.rs](https://docs.rs/tui-widget-list/)

[Crate Badge]: https://img.shields.io/crates/v/tui-widget-list?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44
[License Badge]: https://img.shields.io/crates/l/tui-widget-list?style=flat-square&color=1370D3

License: MIT
