use crate::common::{item_container::ListItemContainer, Colors};
use ratatui::{layout::Alignment, style::Stylize, text::Line, widgets::Padding};
use tui_widget_list::{ListBuilder, ListView};

pub(crate) const TRACKS: &[(&str, &str)] = &[
    ("Bob Dylan", "Blowin' in the Wind"),
    ("Johnny Cash", "Ring of Fire"),
    ("Queen", "Bohemian Rhapsody"),
    ("David Bowie", "Heroes"),
    ("Elton John", "I'm Still Standing"),
    ("Townes Van Zandt", "Pancho and Lefty"),
    ("Falco", "Rock Me Amadeus"),
    ("Johnny Cash", "Folsom Prison Blues"),
    ("David Bowie", "Starman"),
    ("Townes Van Zandt", "Waitin' Around to Die"),
    ("Bob Dylan", "Like a Rolling Stone"),
    ("Johnny Cash", "Hurt"),
    ("Townes Van Zandt", "If I Needed You"),
    ("Bob Dylan", "A Hard Rain's a-Gonna Fall"),
];

pub(crate) struct InfiniteListView;

impl InfiniteListView {
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

        ListView::new(builder, TRACKS.len()).infinite_scrolling(true)
    }
}
