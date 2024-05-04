use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    prelude::*,
    widgets::Widget,
};
use std::{error::Error, io};
use tui_widget_list::{List, ListState, ListWidget, PreRenderContext};

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

impl ListWidget for ListItem<'_> {
    fn pre_render(mut self, context: &PreRenderContext) -> (Self, u16) {
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
        };

        (self, 1)
    }
}

impl Widget for ListItem<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.get_line().render(area, buf);
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

pub struct App<'a> {
    list: List<'a, ListItem<'a>>,
    state: ListState,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let items = vec![
            ListItem::new(Line::from("Item 0")),
            ListItem::new(Line::from("Item 1")),
            ListItem::new(Line::from("Item 2")),
            ListItem::new(Line::from("Item 3")),
            ListItem::new(Line::from("Item 4")),
            ListItem::new(Line::from("Item 5")),
            ListItem::new(Line::from("Item 6")),
            ListItem::new(Line::from("Item 7")),
            ListItem::new(Line::from("Item 8")),
            ListItem::new(Line::from("Item 9")),
            ListItem::new(Line::from("Item 10")),
            ListItem::new(Line::from("Item 11")),
            ListItem::new(Line::from("Item 12")),
            ListItem::new(Line::from("Item 13")),
            ListItem::new(Line::from("Item 14")),
            ListItem::new(Line::from("Item 15")),
            ListItem::new(Line::from("Item 16")),
            ListItem::new(Line::from("Item 17")),
            ListItem::new(Line::from("Item 18")),
            ListItem::new(Line::from("Item 19")),
            ListItem::new(Line::from("Item 20")),
            ListItem::new(Line::from("Item 21")),
            ListItem::new(Line::from("Item 22")),
            ListItem::new(Line::from("Item 23")),
            ListItem::new(Line::from("Item 24")),
            ListItem::new(Line::from("Item 25")),
            ListItem::new(Line::from("Item 26")),
            ListItem::new(Line::from("Item 27")),
            ListItem::new(Line::from("Item 28")),
            ListItem::new(Line::from("Item 29")),
        ];
        let state = ListState::default();
        App {
            list: items.into(),
            state,
        }
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

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

pub fn ui(f: &mut Frame, app: &mut App) {
    let list = app.list.clone();
    f.render_stateful_widget(list, f.size(), &mut app.state);
}

fn prefix_text<'a>(line: Line<'a>, prefix: &'a str) -> Line<'a> {
    let mut spans = line.spans;
    spans.insert(0, Span::from(prefix));
    ratatui::text::Line::from(spans)
}
