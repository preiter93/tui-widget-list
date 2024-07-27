use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    prelude::*,
    widgets::{Block, Borders},
};
use std::{error::Error, io};
use tui_widget_list::{List, ListState, PreRender, PreRenderContext};

#[derive(Debug, Clone)]
pub struct LineItem<'a> {
    line: Line<'a>,
    height: u16,
}

impl LineItem<'_> {
    pub fn new(text: &str, height: u16) -> Self {
        let line = Line::from(Span::styled(
            text.to_string(),
            Style::default().fg(Color::Cyan),
        ))
        .style(Style::default().bg(Color::Black));
        Self { line, height }
    }

    pub fn set_style(&mut self, style: Style) {
        let mut paragraph = std::mem::replace(&mut self.line, Default::default());
        paragraph = paragraph.style(style);
        self.line = paragraph;
    }
}

impl PreRender for LineItem<'_> {
    fn pre_render(&mut self, context: &PreRenderContext) -> u16 {
        if context.is_selected {
            self.line.style = Style::default().bg(Color::White);
        }

        let main_axis_size = self.height;

        main_axis_size
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

pub struct App<'a> {
    pub list: List<'a, LineItem<'a>>,
    pub state: ListState,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let items = vec![
            LineItem::new("Height: 4", 4),
            LineItem::new("Height: 6", 6),
            LineItem::new("Height: 5", 5),
            LineItem::new("Height: 4", 4),
            LineItem::new("Height: 3", 3),
            LineItem::new("Height: 3", 3),
            LineItem::new("Height: 6", 6),
            LineItem::new("Height: 5", 5),
            LineItem::new("Height: 7", 7),
            LineItem::new("Height: 3", 3),
            LineItem::new("Height: 6", 6),
            LineItem::new("Height: 9", 9),
            LineItem::new("Height: 4", 4),
            LineItem::new("Height: 6", 6),
        ];
        let list = List::new(items)
            .bg(Color::Black)
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
