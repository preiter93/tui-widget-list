#[path = "common/lib.rs"]
mod common;
use common::{Colors, Result, Terminal};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Widget},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

const SIZES: [u16; 19] = [32, 3, 4, 64, 6, 5, 4, 3, 3, 6, 5, 7, 3, 6, 9, 10, 4, 4, 6];

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
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let item_count = SIZES.len();

        let block = Block::default().borders(Borders::ALL).title("Outer block");
        let builder = ListBuilder::new(move |context| {
            let size = SIZES[context.index];
            let mut widget = LineItem::new(format!("Size: {size}"));

            if context.is_selected {
                widget.line.style = widget.line.style.bg(Color::White);
            };

            return (widget, size);
        });
        let list = ListView::new(builder, item_count)
            .bg(Color::Black)
            .block(block);
        list.render(area, buf, state);
    }
}

#[derive(Debug, Clone)]
pub struct LineItem<'a> {
    line: Line<'a>,
}

impl LineItem<'_> {
    pub fn new(text: String) -> Self {
        let span = Span::styled(text, Style::default().fg(Colors::TEAL));
        let line = Line::from(span).bg(Colors::CHARCOAL);
        Self { line }
    }
}

impl Widget for LineItem<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = {
            let block = Block::default().borders(Borders::ALL);
            block.clone().render(area, buf);
            block.inner(area)
        };

        self.line.render(inner, buf);
    }
}
