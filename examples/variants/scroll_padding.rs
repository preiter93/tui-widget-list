use crate::common::{item_container::ListItemContainer, Colors};
use crate::infinite::TRACKS;
use ratatui::widgets::Scrollbar;
use ratatui::{layout::Alignment, style::Stylize, text::Line, widgets::Padding};
use tui_widget_list::{ListBuilder, ListView};

pub(crate) struct ScrollPaddingListView;

impl ScrollPaddingListView {
    pub(crate) fn new<'a>() -> ListView<'a, ListItemContainer<'a, Line<'a>>> {
        let builder = ListBuilder::new(|context| {
            let (artist, title) = TRACKS[context.index];
            let label = format!(" {:>2}. {} — {}", context.index + 1, artist, title);

            let mut item = ListItemContainer::new(
                Line::from(label).alignment(Alignment::Left),
                Padding::vertical(1),
            );
            item = match context.is_selected {
                true => item.bg(Colors::ORANGE).fg(Colors::CHARCOAL),
                false if context.index % 2 == 0 => item.bg(Colors::CHARCOAL),
                false => item.bg(Colors::BLACK),
            };

            (item, 3)
        });

        ListView::new(builder, TRACKS.len())
            .infinite_scrolling(false)
            .scroll_padding(5)
            .scrollbar(Scrollbar::default())
    }
}
