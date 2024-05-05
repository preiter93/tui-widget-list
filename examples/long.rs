//! This example showcases a list with many widgets.
//!
//! Note that we do not implement `Widget` and `PreRender` on the ListItem itself,
//! but on its reference. This avoids copying the values in each frame and makes
//! rendering more performant.
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::{error::Error, io};
use tui_widget_list::{List, ListState, PreRender, PreRenderContext};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Implement `Widget` on the mutable reference to ListItem
impl Widget for &mut ListItem<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.clone().get_line().render(area, buf);
    }
}

/// Implement `PreRender` on the mutable reference to ListItem
impl PreRender for &mut ListItem<'_> {
    fn pre_render(&mut self, context: &PreRenderContext) -> u16 {
        if context.index % 2 == 0 {
            self.style = Style::default().bg(Color::Rgb(28, 28, 32));
        } else {
            self.style = Style::default().bg(Color::Rgb(0, 0, 0));
        }

        if context.is_selected {
            self.prefix = Some(">>");
            self.style = Style::default()
                .bg(Color::Rgb(255, 153, 0))
                .fg(Color::Rgb(28, 28, 32));
        } else {
            self.prefix = None;
        };

        1
    }
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

fn main() -> Result<()> {
    let mut terminal = init_terminal()?;

    let app = App::new();
    run_app(&mut terminal, app).unwrap();

    reset_terminal()?;
    terminal.show_cursor()?;

    Ok(())
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    // Create the widgets only once
    let mut items: Vec<_> = (0..300000)
        .map(|index| ListItem::new(Line::from(format!("Item {index}"))))
        .collect();

    loop {
        // Then pass them by reference
        let list = List::from(items.iter_mut().collect::<Vec<_>>());
        terminal.draw(|f| ui(f, &mut app, list))?;

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

pub fn ui(f: &mut Frame, app: &mut App, list: List<&mut ListItem>) {
    f.render_stateful_widget(list, f.size(), &mut app.state);
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
    pub fn new<T>(line: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        Self {
            line: line.into(),
            style: Style::default(),
            prefix: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn prefix(mut self, prefix: Option<&'a str>) -> Self {
        self.prefix = prefix;
        self
    }

    fn get_line(self) -> Line<'a> {
        let text = if let Some(prefix) = self.prefix {
            prefix_text(self.line, prefix)
        } else {
            self.line
        };
        Line::from(text).style(self.style)
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
