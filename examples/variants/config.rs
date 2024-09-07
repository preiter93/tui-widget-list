use ratatui::{
    layout::Alignment,
    style::Stylize,
    text::Line,
    widgets::{Padding, Widget},
};
use tui_widget_list::{ListBuilder, ListView};

use crate::common::{item_container::ListItemContainer, Colors};

#[derive(PartialEq, Eq, Default, Clone)]
pub enum Variant {
    #[default]
    Simple,
    Padded,
    Horizontal,
}

impl Variant {
    pub const COUNT: usize = 3;
    pub fn from_index(index: usize) -> Self {
        match index {
            1 => Variant::Padded,
            2 => Variant::Horizontal,
            _ => Variant::Simple,
        }
    }
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::Simple => write!(f, "Simple"),
            Variant::Padded => write!(f, "Padded"),
            Variant::Horizontal => write!(f, "Horizontal"),
        }
    }
}

pub struct VariantsListView;
impl VariantsListView {
    pub fn new<'a>() -> ListView<'a, ListItemContainer<'a, Line<'a>>> {
        let builder = ListBuilder::new(move |context| {
            let config = Variant::from_index(context.index);
            let line = Line::from(format!("{config}")).alignment(Alignment::Center);
            let mut item = ListItemContainer::new(line, Padding::vertical(1));

            if context.is_selected {
                item = item.bg(Colors::ORANGE).fg(Colors::CHARCOAL);
            };

            return (item, 3);
        });

        return ListView::new(builder, Variant::COUNT);
    }
}

#[derive(Default)]
pub struct Controls;
impl Widget for Controls {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Line::from("k: Up | j: Down | Tab: Left/Right").render(area, buf);
    }
}
