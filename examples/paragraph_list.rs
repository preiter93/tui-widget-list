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
use tui_widget_list::{ListableWidget, WidgetList, WidgetListState};

#[derive(Clone)]
pub struct StatefulWidgetList<T> {
    pub state: WidgetListState,
    pub items: Vec<T>,
}

impl<T> Default for StatefulWidgetList<T> {
    fn default() -> Self {
        Self {
            state: WidgetListState::default(),
            items: vec![],
        }
    }
}

impl<T: ListableWidget> StatefulWidgetList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            state: WidgetListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

#[derive(Debug, Clone)]
pub struct TestWidget<'a> {
    paragraph: Paragraph<'a>,
    height: u16,
}

impl TestWidget<'_> {
    pub fn new(text: &str, height: u16) -> Self {
        let paragraph = Paragraph::new(vec![Spans::from(Span::styled(
            text.to_string(),
            Style::default().fg(Color::Magenta),
        ))])
        .style(Style::default().bg(Color::Black))
        .block(Block::default().borders(Borders::ALL));
        Self { paragraph, height }
    }
}

impl<'a> ListableWidget for TestWidget<'a> {
    fn height(&self) -> u16 {
        self.height
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        self.paragraph.clone().render(area, buf);
    }

    fn render_selected(&self, area: Rect, buf: &mut Buffer) {
        self.paragraph
            .clone()
            .style(Style::default().bg(Color::White))
            .render(area, buf);
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
    pub list: StatefulWidgetList<TestWidget<'a>>,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let mut items = vec![];
        for i in 0..15 {
            let text = format!("Item: {}", i);
            let height = i + 3_u16;
            let item = TestWidget::new(&text, height);
            items.push(item);
        }
        let list = StatefulWidgetList::with_items(items);
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

    let items = &app.list.items;

    let widget = WidgetList::new(items.clone())
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().bg(Color::Black));

    f.render_stateful_widget(widget, chunks[0], &mut app.list.state);
}
