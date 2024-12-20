use crate::common::{item_container::ListItemContainer, Colors};
use ratatui::{layout::Alignment, style::Stylize, text::Line, widgets::Padding};
use tui_widget_list::{ListBuilder, ListView, ScrollAxis};

pub(crate) struct HorizontalListView;

impl HorizontalListView {
    pub(crate) fn new<'a>() -> ListView<'a, ListItemContainer<'a, Line<'a>>> {
        let builder = ListBuilder::new(|context| {
            let mut line = ListItemContainer::new(
                Line::from(format!("Item {0}", context.index)).alignment(Alignment::Center),
                Padding::vertical(1),
            );
            line = match context.is_selected {
                true => line.bg(Colors::ORANGE).fg(Colors::CHARCOAL),
                false if context.index % 2 == 0 => line.bg(Colors::CHARCOAL),
                false => line.bg(Colors::BLACK),
            };

            return (line, 20);
        });

        return ListView::new(builder, 10).scroll_axis(ScrollAxis::Horizontal);
    }
}
