#[path = "common/lib.rs"]
mod common;

use common::{Colors, Result, Terminal};
use ratatui::crossterm::event::{
    self, Event, KeyCode, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation},
};
use tui_widget_list::{hit_test, ListBuilder, ListState, ListView};

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

            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up | KeyCode::Char('k') => state.previous(),
                    KeyCode::Down | KeyCode::Char('j') => state.next(),
                    _ => {}
                },
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    ..
                }) => {
                    if let Some(index) = hit_test(&state, column, row) {
                        state.select(Some(index));
                    }
                }
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::ScrollUp,
                    ..
                }) => {
                    state.previous();
                }
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::ScrollDown,
                    ..
                }) => {
                    state.next();
                }
                _ => {}
            }
        }
    }
}

impl StatefulWidget for &App {
    type State = ListState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let builder = ListBuilder::new(|context| {
            let text = format!("Item {0}", context.index);
            let mut item = Line::from(text);

            if context.index % 2 == 0 {
                item.style = Style::default().bg(Colors::CHARCOAL)
            } else {
                item.style = Style::default().bg(Colors::BLACK)
            };

            if context.is_selected {
                let mut spans = item.spans;
                spans.insert(0, Span::from(">>"));
                item = Line::from(spans);
                item.style = Style::default().bg(Colors::ORANGE).fg(Colors::CHARCOAL);
            };

            let style = item.style;
            let lines = vec![item, Line::from("")];
            let paragraph = Paragraph::new(lines).style(style);
            (paragraph, 2)
        });

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("┐"))
            .end_symbol(Some("┘"));
        let list = ListView::new(builder, 50)
            .block(Block::default().borders(Borders::ALL))
            .scrollbar(scrollbar);

        list.render(area, buf, state);
    }
}
