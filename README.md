## Widget list implementation for TUI/Ratatui


[Ratatui](https://github.com/tui-rs-revival/ratatui) is a UI Framework to build terminal user interfaces. Ratatui itself provides
some base widgets such as a list of Texts.

This library provides an extension to render a list of arbitrary widgets.


### Documentation

The documentation can be found on [docs.rs.](https://docs.rs/tui-widget-list)

### Demo
Simple List, similar to TUIs List widget
```
cargo run --example simple_list
```

List with differently sized Paragraphs
```
cargo run --example paragraph_list
```

### Usage
Use a custom widget with [`SelectableWidgetList`]. The render method is
available on `wiget_list`.

```rust
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Widget};
use tui_widget_list::{WidgetListItem, SelectableWidgetList};

#[derive(Debug, Clone)]
pub struct MyListItem<'a> {
    content: Paragraph<'a>,
    height: u16,
}

impl MyListItem<'_> {
    pub fn new(text: &'static str, height: u16) -> Self {
        let content = Paragraph::new(Text::from(text));
        Self { content, height }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.content = self.content.style(style);
        self
    }

    // Render the item differently depending on the selection state
    fn modify_fn(mut item: WidgetListItem<Self>, selected: Option<bool>) -> WidgetListItem<Self> {
        if let Some(selected) = selected {
            if selected {
                let style = Style::default().bg(Color::White);
                item.content = item.content.style(style);
            }
        }
        item
    }
}

impl<'a> Widget for MyListItem<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.content.render(area, buf);
    }
}

impl<'a> From<MyListItem<'a>> for WidgetListItem<MyListItem<'a>> {
    fn from(val: MyListItem<'a>) -> Self {
        let height = 1_u16; // Assume we have a one line paragraph
        Self::new(val, height).modify_fn(MyListItem::modify_fn)
    }
}

let items = vec![
    MyListItem::new("hello", 3),
    MyListItem::new("world", 4),
];
let widget_list = SelectableWidgetList::new(items);

// widget_list can be rendered like any other widget in TUI. Note that
// we pass it as mutable reference in order to not lose the state.
// f.render_widget(&mut widget_list, area);
```

![](img/screenshot.png)
