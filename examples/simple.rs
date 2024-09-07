#[path = "common/lib.rs"]
mod common;
use common::{Colors, Result, Terminal};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use tui_widget_list::{ListBuilder, ListState, ListView};

fn main() -> Result<()> {
    let mut terminal = Terminal::init()?;

    App::default().run(&mut terminal).unwrap();

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
        let builder = ListBuilder::new(|context| {
            let text = format!("Item {0}", context.index);
            let mut item = Line::from(text);

            if context.index % 2 == 0 {
                item.style = Style::default().bg(Colors::CHARCOAL)
            } else {
                item.style = Style::default().bg(Colors::BLACK)
            };

            if context.is_selected {
                item = prefix_text(item, ">>");
                item.style = Style::default().bg(Colors::ORANGE).fg(Colors::CHARCOAL);
            };

            return (item, 1);
        });
        let list = ListView::new(builder, 20);

        list.render(area, buf, state);
    }
}

fn prefix_text<'a>(line: Line<'a>, prefix: &'a str) -> Line<'a> {
    let mut spans = line.spans;
    spans.insert(0, Span::from(prefix));
    ratatui::text::Line::from(spans)
}
