use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    prelude::*,
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
};
use std::{error::Error, io};
use tui_widget_list::{List, ListState, ListWidget, RenderContext};

#[derive(Debug, Clone)]
pub struct ParagraphItem<'a> {
    paragraph: Paragraph<'a>,
    height: u16,
}

impl ParagraphItem<'_> {
    pub fn new(text: &str, height: u16) -> Self {
        let paragraph = Paragraph::new(vec![Line::from(Span::styled(
            text.to_string(),
            Style::default().fg(Color::Cyan),
        ))])
        .style(Style::default().bg(Color::Black))
        .block(Block::default().borders(Borders::ALL).title("Inner block"));
        Self { paragraph, height }
    }

    pub fn set_style(&mut self, style: Style) {
        let mut paragraph = std::mem::replace(&mut self.paragraph, Default::default());
        paragraph = paragraph.style(style);
        self.paragraph = paragraph;
    }
}

impl ListWidget for ParagraphItem<'_> {
    fn pre_render(mut self, context: &RenderContext) -> (Self, u16) {
        if context.is_selected {
            self.paragraph = self.paragraph.style(Style::default().bg(Color::White));
        }

        let height = self.height;

        (self, height)
    }
}

impl Widget for ParagraphItem<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.paragraph.render(area, buf);
    }
}

#[derive(Debug, Clone)]
pub struct TabItem {
    titles: Vec<String>,
    selected: bool,
}

impl TabItem {
    pub fn new(titles: Vec<String>) -> Self {
        Self {
            titles,
            selected: false,
        }
    }
}

impl ListWidget for TabItem {
    fn pre_render(mut self, context: &RenderContext) -> (Self, u16) {
        if context.is_selected {
            self.selected = true;
        }

        (self, 3)
    }
}

impl Widget for TabItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut tabs =
            Tabs::new(self.titles).block(Block::default().borders(Borders::ALL).title("Tabs"));
        if self.selected {
            tabs = tabs
                .highlight_style(Style::default().bold().on_black())
                .style(Style::default().on_dark_gray());
        }
        tabs.render(area, buf);
    }
}

#[derive(Clone)]
enum ListElements<'a> {
    TabItem(TabItem),
    ParagraphItem(ParagraphItem<'a>),
}

impl ListWidget for ListElements<'_> {
    fn pre_render(self, context: &RenderContext) -> (Self, u16) {
        match self {
            Self::TabItem(inner) => {
                let (inner, main_axis_size) = inner.pre_render(context);
                (ListElements::TabItem(inner), main_axis_size)
            }
            Self::ParagraphItem(inner) => {
                let (inner, main_axis_size) = inner.pre_render(context);
                (ListElements::ParagraphItem(inner), main_axis_size)
            }
        }
    }
}

impl Widget for ListElements<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Self::TabItem(inner) => inner.render(area, buf),
            Self::ParagraphItem(inner) => inner.render(area, buf),
        };
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
    list: List<'a, ListElements<'a>>,
    state: ListState,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let items = vec![
            ListElements::ParagraphItem(ParagraphItem::new("Height: 4", 4)),
            ListElements::TabItem(TabItem::new(vec![
                "Item A".to_string(),
                "Item B".to_string(),
            ])),
            ListElements::ParagraphItem(ParagraphItem::new("Height: 6", 6)),
        ];
        let list = List::new(items)
            .style(Style::default().bg(Color::Black))
            .block(Block::default().borders(Borders::ALL).title("Outer block"));
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
