# tui-widget-list

## A versatile list implementation for Ratatui

This crate offers a stateful widget list implementation [`List`] for `Ratatui` that allows to
work with any list of widgets that implement to the [`Listable`] trait. The associated selection state
is [`ListState`] which offers methods like next and previous.

### Examples
```rust
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Widget};
use tui_widget_list::{List, Listable};

#[derive(Debug, Clone)]
pub struct MyListItem<'a> {
    content: Paragraph<'a>,
    height: usize,
}

impl MyListItem<'_> {
    pub fn new(text: &'static str, height: usize) -> Self {
        let content = Paragraph::new(Text::from(text));
        Self { content, height }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.content = self.content.style(style);
        self
    }
}

impl Listable for MyListItem<'_> {
    fn height(&self) -> usize {
        self.height
    }

    fn highlight(self) -> Option<Self> {
        Some(self.style(Style::default().bg(Color::Cyan)))
    }
 }

impl Widget for MyListItem<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.content.render(area, buf);
    }
}
// widget_list can be rendered like any other widget in TUI. Note that
// we pass it as mutable reference in order to not lose the state.
// f.render_stateful_widget(list, area, &mut state);
```

For more examples see `examples/simple`, `examples/paragraph` or `examples/mixed` in
[tui-widget-list](https://github.com/preiter93/tui-widget-list/tree/main/examples).

### Configuration
The appearance of [`List`] can be modified
- **style**: The base style of the list.
- **block**: An optional outer block around the list.
- **truncate**: If truncate is true, the first and last elements are truncated to fill the entire screen. True by default.

The behaviour of [`ListState`] can be modified
- **circular**: Whether the selection is circular, i.e. if true, the first item is selected after the last. True by default.

License: MIT
