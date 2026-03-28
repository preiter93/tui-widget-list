use crate::common::Colors;
use ratatui::{
    style::Stylize,
    text::{Line, Text},
    widgets::Padding,
};
use tui_widget_list::{ListBuilder, ListView, ScrollDirection};

use crate::common::item_container::ListItemContainer;

const MESSAGES: &[(&str, &str)] = &[
    ("Alice", "Good morning!"),
    ("Bob", "Hey, how are you?"),
    ("Alice", "Pretty good, thanks!"),
    ("Carl", "Hi everyone"),
    ("Bob", "Welcome!"),
    ("Alice", "How's it going?"),
];

pub(crate) struct BackwardListView;

impl BackwardListView {
    pub(crate) fn new<'a>() -> ListView<'a, ListItemContainer<'a, Text<'a>>> {
        let builder = ListBuilder::new(|context| {
            let (author, body) = MESSAGES[context.index % MESSAGES.len()];

            let author_color = if context.is_selected {
                Colors::WHITE
            } else {
                match author {
                    "Alice" => Colors::ORANGE,
                    "Bob" => Colors::TEAL,
                    _ => Colors::GRAY,
                }
            };

            let text = Text::from(vec![
                Line::from(author).style(ratatui::style::Style::default().fg(author_color)),
                Line::from(body),
            ]);

            let mut item = ListItemContainer::new(text, Padding::new(1, 1, 0, 0));
            item = if context.is_selected {
                item.bg(Colors::ORANGE).fg(Colors::CHARCOAL)
            } else if context.index % 2 == 0 {
                item.bg(Colors::CHARCOAL)
            } else {
                item.bg(Colors::BLACK)
            };

            (item, 2)
        });

        ListView::new(builder, MESSAGES.len()).scroll_direction(ScrollDirection::Backward)
    }
}
