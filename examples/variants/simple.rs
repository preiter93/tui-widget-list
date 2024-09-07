use ratatui::{layout::Alignment, style::Stylize, text::Line};
use tui_widget_list::{ListBuilder, ListView};

use crate::common::Colors;

pub(crate) struct SimpleListView;

impl SimpleListView {
    pub(crate) fn new<'a>() -> ListView<'a, Line<'a>> {
        let builder = ListBuilder::new(|context| {
            let mut line =
                Line::from(format!("Item {0}", context.index)).alignment(Alignment::Center);
            line = match context.is_selected {
                true => line.bg(Colors::ORANGE).fg(Colors::CHARCOAL),
                false if context.index % 2 == 0 => line.bg(Colors::CHARCOAL),
                false => line.bg(Colors::BLACK),
            };

            return (line, 1);
        });

        return ListView::new(builder, 50);
    }
}
