use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};
use std::{error::Error, io};
use tui_widget_list::{widget::List, ListState, Listable};

/// A simple list text item.
#[derive(Debug, Clone)]
pub struct ListItem<'a> {
    /// The text
    text: Text<'a>,

    /// The style
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

    fn get_paragraph(self) -> Paragraph<'a> {
        let text = if let Some(prefix) = self.prefix {
            prefix_text(self.text, prefix)
        } else {
            self.text
        };
        Paragraph::new(text).style(self.style)
    }
}

impl Listable for ListItem<'_> {
    fn height(&self) -> usize {
        self.text.height()
    }

    fn highlight(self) -> Option<Self> {
        Some(
            self.prefix(Some(">>"))
                .style(Style::default().bg(Color::Cyan)),
        )
    }
}

impl Widget for ListItem<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.get_paragraph().render(area, buf);
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
            ListItem::new(Text::from("Item 1")),
            ListItem::new(Text::from("Item 2")),
            ListItem::new(Text::from("Item 3")),
            ListItem::new(Text::from("Item 1")),
            ListItem::new(Text::from("Item 1")),
            ListItem::new(Text::from("Item 1")),
            ListItem::new(Text::from("Item 2")),
            ListItem::new(Text::from("Item 3")),
            ListItem::new(Text::from("Item 2")),
            ListItem::new(Text::from("Item 3")),
            ListItem::new(Text::from("Item 2")),
            ListItem::new(Text::from("Item 3")),
        ];
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .title(Span::styled("Selection", Style::default()));
        let mut list: List<ListItem> = items.into();
        list = list.block(block);
        let state = ListState::default();
        App { list, state }
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

fn prefix_text<'a>(text: Text<'a>, prefix: &'a str) -> Text<'a> {
    let lines = text
        .lines
        .into_iter()
        .map(|line| {
            let mut spans = line.spans;
            spans.insert(0, Span::from(prefix));
            ratatui::text::Line::from(spans)
        })
        .collect();
    Text { lines }
}
