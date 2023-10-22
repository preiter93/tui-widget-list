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
use tui_widget_list::{WidgetItem, WidgetList};

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

    pub fn style(mut self, style: Style) -> Self {
        self.paragraph = self.paragraph.set_style(style);
        self
    }
}

impl<'a> WidgetItem for ParagraphItem<'a> {
    fn height(&self) -> usize {
        self.height as usize
    }

    fn highlighted(&self) -> Option<Self> {
        let style = Style::default().bg(Color::White);
        Some(self.clone().style(style))
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        self.clone().paragraph.render(area, buf);
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

impl<'a> WidgetItem for TabItem {
    fn height(&self) -> usize {
        3
    }

    fn highlighted(&self) -> Option<Self> {
        Some(Self {
            titles: self.titles.clone(),
            selected: true,
        })
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut tabs = Tabs::new(self.titles.clone())
            .block(Block::default().borders(Borders::ALL).title("Tabs"));
        if self.selected {
            tabs = tabs
                .highlight_style(Style::default().bold().on_black())
                .style(Style::default().on_dark_gray());
        }
        tabs.render(area, buf);
    }
}

// impl TabItem {
// pub fn new(text: &str, height: u16) -> Self {
//     let paragraph = Paragraph::new(vec![Line::from(Span::styled(
//         text.to_string(),
//         Style::default().fg(Color::Cyan),
//     ))])
//     .style(Style::default().bg(Color::Black))
//     .block(Block::default().borders(Borders::ALL).title("Inner block"));
//     Self { paragraph, height }
// }
//
// pub fn style(mut self, style: Style) -> Self {
//     self.paragraph = self.paragraph.set_style(style);
//     self
// }
// }

enum ListElements<'a> {
    TabItem(TabItem),
    ParagraphItem(ParagraphItem<'a>),
}

impl WidgetItem for ListElements<'_> {
    fn height(&self) -> usize {
        match &self {
            Self::TabItem(inner) => inner.height(),
            Self::ParagraphItem(inner) => inner.height(),
        }
    }

    fn highlighted(&self) -> Option<Self> {
        match &self {
            Self::TabItem(inner) => inner.highlighted().map(Self::TabItem),
            Self::ParagraphItem(inner) => inner.highlighted().map(Self::ParagraphItem),
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        match &self {
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
    list: WidgetList<'a, ListElements<'a>>,
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
        let list = WidgetList::new(items)
            .style(Style::default().bg(Color::Black))
            .block(Block::default().borders(Borders::ALL).title("Outer block"))
            .truncate(true);
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

    f.render_widget(&mut app.list, chunks[0]);
}
