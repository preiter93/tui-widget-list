use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame, Terminal,
};
use std::{error::Error, io};
use tui_widget_list::{ListableWidget, SelectableWidgetList};

#[derive(Debug, Clone)]
pub struct ListItem<'a> {
    /// The items text
    text: Text<'a>,

    /// The items style
    style: Style,

    /// The current prefix. Changes when the item is selected.
    prefix: Option<&'a str>,
}

impl<'a> ListItem<'a> {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            text: text.into(),
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

    fn paragraph(self) -> Paragraph<'a> {
        let text = if let Some(prefix) = self.prefix {
            prefix_text(self.text, prefix)
        } else {
            self.text
        };
        Paragraph::new(text).style(self.style)
    }

    // Renders the items differently depending on the selection state
    fn modify_fn(mut self, is_selected: Option<bool>) -> Self {
        if let Some(selected) = is_selected {
            if selected {
                self.prefix = Some(">>");
                self.style = Style::default().bg(Color::Cyan);
            } else {
                self.prefix = Some("  ");
            }
        }
        self
    }
}

impl<'a> Widget for ListItem<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.paragraph().render(area, buf);
    }
}

impl<'a> ListableWidget for ListItem<'a> {
    fn height(&self) -> u16 {
        1
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
    pub list: SelectableWidgetList<'a, ListItem<'a>>,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let items = vec![
            ListItem::new(Text::from("Item 1")),
            ListItem::new(Text::from("Item 2")),
            ListItem::new(Text::from("Item 3")),
        ];
        let list = SelectableWidgetList::new(items);
        App { list }
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => app.list.previous(),
                    KeyCode::Down => app.list.next(),
                    _ => {}
                }
            }
        }
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0)].as_ref())
        .split(f.size());

    let widget = app
        .list
        .content
        .clone()
        .style(Style::default().bg(Color::Black))
        .block(Block::default().borders(Borders::ALL))
        .modify_fn(ListItem::modify_fn);

    f.render_stateful_widget(widget, chunks[0], &mut app.list.state);
}

fn prefix_text<'a>(text: Text<'a>, prefix: &'a str) -> Text<'a> {
    let lines = text
        .lines
        .into_iter()
        .map(|mut line| {
            line.0.insert(0, Span::from(prefix));
            line
        })
        .collect();
    Text { lines }
}
