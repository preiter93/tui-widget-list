use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame, Terminal,
};
use std::{error::Error, io};
use tui_widget_list::{ListableWidget, SelectableWidgetList};

#[derive(Debug, Clone)]
pub struct ParagraphItem<'a> {
    paragraph: Paragraph<'a>,
    height: u16,
}

impl ParagraphItem<'_> {
    pub fn new(text: &str, height: u16) -> Self {
        let paragraph = Paragraph::new(vec![Spans::from(Span::styled(
            text.to_string(),
            Style::default().fg(Color::Magenta),
        ))])
        .style(Style::default().bg(Color::Black))
        .block(Block::default().borders(Borders::ALL));
        Self { paragraph, height }
    }

    // Render the item differently depending on the selection state
    fn modify_fn(mut self, is_selected: Option<bool>) -> Self {
        if let Some(selected) = is_selected {
            if selected {
                let style = Style::default().bg(Color::White);
                self.paragraph = self.paragraph.style(style);
            }
        }
        self
    }
}

impl<'a> Widget for ParagraphItem<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.paragraph.render(area, buf);
    }
}

impl<'a> ListableWidget for ParagraphItem<'a> {
    fn height(&self) -> u16 {
        self.height
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
    pub list: SelectableWidgetList<'a, ParagraphItem<'a>>,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let items = vec![
            ParagraphItem::new("Height: 4", 4),
            ParagraphItem::new("Height: 4", 4),
            ParagraphItem::new("Height: 5", 5),
            ParagraphItem::new("Height: 4", 4),
            ParagraphItem::new("Height: 3", 3),
            ParagraphItem::new("Height: 3", 3),
            ParagraphItem::new("Height: 6", 6),
            ParagraphItem::new("Height: 5", 5),
            ParagraphItem::new("Height: 7", 7),
            ParagraphItem::new("Height: 3", 3),
            ParagraphItem::new("Height: 6", 6),
            ParagraphItem::new("Height: 9", 9),
            ParagraphItem::new("Height: 4", 4),
            ParagraphItem::new("Height: 6", 6),
        ];
        let list = SelectableWidgetList::new(items).modify_fn(ParagraphItem::modify_fn);
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
        .modify_fn(ParagraphItem::modify_fn);

    f.render_stateful_widget(widget, chunks[0], &mut app.list.state);
}
