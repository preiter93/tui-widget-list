use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::{error::Error, io};
use tui_widget_list::{ListBuilder, ListState, ListView};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut terminal = init_terminal()?;

    let mut app = App::new();
    app.run(&mut terminal).unwrap();

    reset_terminal()?;
    terminal.show_cursor()?;

    Ok(())
}

pub struct App {
    state: ListState,
}

impl App {
    pub fn new() -> App {
        let state = ListState::default();

        App { state }
    }
}

impl App {
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            self.draw(terminal)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Up | KeyCode::Char('k') => self.state.previous(),
                        KeyCode::Down | KeyCode::Char('j') => self.state.next(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        terminal.draw(|frame| {
            frame.render_widget(self, frame.size());
        })?;
        Ok(())
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let builder = ListBuilder::new(|context| {
            let text = format!("Item {0}", context.index);
            let mut item = Line::from(text);

            if context.index % 2 == 0 {
                item.style = Style::default().bg(Color::Rgb(28, 28, 32))
            } else {
                item.style = Style::default().bg(Color::Rgb(0, 0, 0))
            };

            if context.is_selected {
                item = prefix_text(item, ">>");
                item.style = Style::default()
                    .bg(Color::Rgb(255, 153, 0))
                    .fg(Color::Rgb(28, 28, 32));
            };

            return (item, 1);
        });
        let list = ListView::new(builder, 20);
        let state = &mut self.state;

        list.render(area, buf, state);
    }
}

fn prefix_text<'a>(line: Line<'a>, prefix: &'a str) -> Line<'a> {
    let mut spans = line.spans;
    spans.insert(0, Span::from(prefix));
    ratatui::text::Line::from(spans)
}

/// A simple list text item.
#[derive(Debug, Clone)]
pub struct ListItem<'a> {
    /// The line
    line: Line<'a>,

    /// The style
    style: Style,

    /// The current prefix. Changes when the item is selected.
    prefix: Option<&'a str>,
}

impl<'a> ListItem<'a> {
    pub fn new(text: String, style: Style, prefix: Option<&'a str>) -> Self {
        Self {
            line: Line::from(text),
            style,
            prefix,
        }
    }
}

impl Widget for ListItem<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = if let Some(prefix) = self.prefix {
            prefix_text(self.line, prefix)
        } else {
            self.line
        };
        Line::from(text).style(self.style).render(area, buf);
    }
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
