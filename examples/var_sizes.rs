#[path = "common/lib.rs"]
mod common;
use common::{Colors, Result, Terminal};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Scrollbar, Widget},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

const ITEMS: &[(&str, u16)] = &[
    ("Header", 3),
    ("Summary", 5),
    ("Details", 30),
    ("Note A", 4),
    ("Note B", 4),
    ("Body", 25),
    ("Sidebar", 6),
    ("Note C", 3),
    ("Note D", 3),
    ("Footer", 5),
];

fn main() -> Result<()> {
    let mut terminal = Terminal::init()?;
    App::default().run(&mut terminal)?;
    Terminal::reset()?;
    terminal.show_cursor()?;
    Ok(())
}

#[derive(Default)]
pub struct App;

impl App {
    pub fn run(&self, terminal: &mut Terminal) -> Result<()> {
        let mut state = ListState::default();
        loop {
            terminal.draw_app(self, &mut state)?;
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Up | KeyCode::Char('k') => state.previous(),
                        KeyCode::Down | KeyCode::Char('j') => state.next(),
                        _ => {}
                    }
                }
            }
        }
    }
}

impl StatefulWidget for &App {
    type State = ListState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let builder = ListBuilder::new(move |context| {
            let (title, size) = ITEMS[context.index];
            let label = format!(" {} (h={})", title, size);

            let style = if context.is_selected {
                Style::default().bg(Colors::ORANGE).fg(Colors::CHARCOAL)
            } else if context.index % 2 == 0 {
                Style::default().bg(Colors::CHARCOAL).fg(Colors::WHITE)
            } else {
                Style::default().bg(Colors::BLACK).fg(Colors::WHITE)
            };

            let block = Block::default().borders(Borders::ALL).style(style);
            let inner = block.inner(Rect::new(0, 0, context.cross_axis_size, size));
            let widget = SectionItem {
                label,
                block,
                inner_height: inner.height,
            };

            (widget, size)
        });

        let list = ListView::new(builder, ITEMS.len())
            .infinite_scrolling(false)
            .scrollbar(Scrollbar::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Variable Sizes "),
            );
        list.render(area, buf, state);
    }
}

struct SectionItem {
    label: String,
    block: Block<'static>,
    inner_height: u16,
}

impl Widget for SectionItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = self.block.inner(area);
        self.block.render(area, buf);
        if inner.height > 0 {
            Line::from(self.label).render(inner, buf);
        }
        for row in 1..inner.height.min(self.inner_height) {
            let y = inner.y + row;
            let filler = format!("  line {}", row);
            Line::from(filler)
                .style(Style::default().fg(Colors::GRAY))
                .render(Rect::new(inner.x, y, inner.width, 1), buf);
        }
    }
}
