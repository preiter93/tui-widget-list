#![cfg(feature = "unstable-widget-ref")]
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    prelude::*,
    widgets::{Block, Borders, Widget},
};
use std::{error::Error, io};
use tui_widget_list::{ListBuilder, ListState, ListView};

#[derive(Debug, Clone)]
pub struct LineItem<'a> {
    line: Line<'a>,
}

impl LineItem<'_> {
    pub fn new(text: String) -> Self {
        let span = Span::styled(text, Style::default().fg(Color::Cyan));
        let line = Line::from(span).bg(Color::Black);
        Self { line }
    }

    pub fn set_style(&mut self, style: Style) {
        let mut paragraph = std::mem::replace(&mut self.line, Default::default());
        paragraph = paragraph.style(style);
        self.line = paragraph;
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

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut terminal = init_terminal()?;

    let app = App::new();
    run_app(&mut terminal, app).unwrap();

    reset_terminal()?;
    terminal.show_cursor()?;

    Ok(())
}

/// Initializes the terminal.
fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    panic_hook();

    Ok(terminal)
}

/// Resets the terminal.
fn reset_terminal() -> Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}

/// Shutdown gracefully
fn panic_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        reset_terminal().unwrap();
        original_hook(panic);
    }));
}

pub struct App {
    pub state: ListState,
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let sizes = vec![100, 100, 4, 6, 5, 4, 3, 3, 6, 5, 7, 3, 6, 9, 10, 4, 4, 6];
        let item_count = sizes.len();

        let block = Block::default().borders(Borders::ALL).title("Outer block");
        let builder = ListBuilder::new(move |context| {
            let size = sizes[context.index];
            let mut widget = LineItem::new(format!("Height: {:0}", size));

            if context.is_selected {
                widget.line.style = Style::default().bg(Color::White);
            };

            return (widget, size);
        });
        let list = ListView::new(builder, item_count)
            .bg(Color::Black)
            .block(block);
        list.render(area, buf, &mut self.state);
    }
}

impl App {
    pub fn new() -> App {
        let state = ListState::default();
        App { state }
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            frame.render_widget(&mut app, frame.area());
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => app.state.previous(),
                    KeyCode::Down => app.state.next(),
                    _ => {}
                }
            }
        }
    }
}
