use crate::common::Colors;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Styled},
    text::Line,
    widgets::{Block, Borders, Widget},
};
use tui_widget_list::{ListBuilder, ListView, ScrollAxis};

const PALETTE: &[(&str, Color)] = &[
    ("Coral", Color::Rgb(255, 127, 80)),
    ("Teal", Color::Rgb(0, 128, 128)),
    ("Gold", Color::Rgb(255, 215, 0)),
    ("Orchid", Color::Rgb(218, 112, 214)),
    ("Steel", Color::Rgb(70, 130, 180)),
    ("Mint", Color::Rgb(152, 255, 152)),
    ("Salmon", Color::Rgb(250, 128, 114)),
    ("Slate", Color::Rgb(106, 90, 205)),
    ("Peach", Color::Rgb(255, 218, 185)),
    ("Olive", Color::Rgb(128, 128, 0)),
];

pub(crate) struct HorizontalListView;

impl HorizontalListView {
    pub(crate) fn new<'a>() -> ListView<'a, ColorSwatch> {
        let builder = ListBuilder::new(|context| {
            let (name, color) = PALETTE[context.index % PALETTE.len()];
            let mut swatch = ColorSwatch::new(name, color);
            if context.is_selected {
                swatch = swatch.set_style(Style::default().fg(Colors::ORANGE));
            }
            (swatch, 18)
        });

        ListView::new(builder, PALETTE.len()).scroll_axis(ScrollAxis::Horizontal)
    }
}

pub(crate) struct ColorSwatch {
    name: &'static str,
    color: Color,
    style: Style,
}

impl ColorSwatch {
    fn new(name: &'static str, color: Color) -> Self {
        Self {
            name,
            color,
            style: Style::default().fg(Colors::GRAY),
        }
    }
}

impl Styled for ColorSwatch {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

impl Widget for ColorSwatch {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).style(self.style);
        let inner = block.inner(area);
        block.render(area, buf);

        // Color swatch rectangle in the center
        let swatch_h = inner.height.saturating_sub(2).min(4);
        let swatch_w = inner.width.saturating_sub(2);
        let swatch_y = inner.y + (inner.height.saturating_sub(swatch_h + 2)) / 2;
        let swatch_x = inner.x + (inner.width.saturating_sub(swatch_w)) / 2;
        for y in swatch_y..swatch_y + swatch_h {
            for x in swatch_x..swatch_x + swatch_w {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(self.color);
                    cell.set_char(' ');
                }
            }
        }

        // Label below the swatch
        let label_y = swatch_y + swatch_h + 1;
        if label_y < inner.bottom() {
            let label_area = Rect::new(inner.x, label_y, inner.width, 1);
            Line::from(self.name)
                .centered()
                .style(self.style)
                .render(label_area, buf);
        }
    }
}
