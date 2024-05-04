use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use ratatui::Terminal;
use std::error::Error;
use std::io::{stdout, Stdout};
use tui_widget_list::{List, ListState, ListWidget, RenderContext, ScrollAxis};

#[derive(Debug, Clone)]
pub struct TextContainer {
    title: String,
    content: Vec<String>,
    style: Style,
    height: u16,
    expand: bool,
}

impl TextContainer {
    pub fn new(title: &str, content: Vec<String>) -> Self {
        Self {
            title: title.to_string(),
            content,
            style: Style::default(),
            height: 2,
            expand: false,
        }
    }
}

impl ListWidget for TextContainer {
    fn pre_render(mut self, context: &RenderContext) -> (Self, u16) {
        if context.index % 2 == 0 {
            self.style = THEME.even;
        } else {
            self.style = THEME.odd;
        }

        if context.is_selected {
            self.style = THEME.selection;
            self.height = 3 + self.content.len() as u16;
            self.expand = true;
        }

        let height = self.height;
        (self, height)
    }
}

impl Widget for TextContainer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = vec![Line::styled(self.title, self.style)];
        if self.expand {
            lines.push(Line::from(String::new()));
            lines.extend(self.content.into_iter().map(|x| Line::from(x)));
            lines.push(Line::from(String::new()));
        }
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(self.style)
            .render(area, buf);
    }
}

struct ColoredContainer {
    color: Color,
    border_style: Style,
    border_type: BorderType,
}

impl ColoredContainer {
    fn new(color: Color) -> Self {
        Self {
            color,
            border_style: Style::default(),
            border_type: BorderType::Plain,
        }
    }
}

impl Widget for ColoredContainer {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Block::default()
            .borders(Borders::ALL)
            .border_style(self.border_style)
            .border_type(self.border_type)
            .bg(self.color)
            .render(area, buf);
    }
}
impl ListWidget for ColoredContainer {
    fn pre_render(mut self, context: &RenderContext) -> (Self, u16) {
        if context.is_selected {
            self.border_style = Style::default().fg(Color::Black);
            self.border_type = BorderType::Thick;
        }

        (self, 15)
    }
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut terminal = init_terminal()?;

    App::default().run(&mut terminal).unwrap();

    reset_terminal()?;
    terminal.show_cursor()?;

    Ok(())
}

/// Initializes the terminal.
fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    crossterm::execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout());

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    panic_hook();

    Ok(terminal)
}

/// Resets the terminal.
fn reset_terminal() -> Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

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

#[derive(Default)]
pub struct App {
    pub text_list_state: ListState,
    pub color_list_state: ListState,
}

impl App {
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            self.draw(terminal)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Up | KeyCode::Char('k') => self.text_list_state.previous(),
                        KeyCode::Down | KeyCode::Char('j') => self.text_list_state.next(),
                        KeyCode::Left | KeyCode::Char('h') => self.color_list_state.previous(),
                        KeyCode::Right | KeyCode::Char('l') => self.color_list_state.next(),
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

// pub fn ui(f: &mut Frame, app: &mut App) {
//     use Constraint::{Min, Percentage};
//     let area = f.size();
//     let [top, bottom] = Layout::vertical([Percentage(75), Min(0)]).areas(area);
//
//     f.render_stateful_widget(demo_text_list(), top, &mut app.text_list_state);
//     f.render_stateful_widget(demo_color_list(), bottom, &mut app.color_list_state);
// }
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        use Constraint::{Min, Percentage};
        let [top, bottom] = Layout::vertical([Percentage(75), Min(0)]).areas(area);
        demo_text_list().render(top, buf, &mut self.text_list_state);
        demo_color_list().render(bottom, buf, &mut self.color_list_state);
    }
}

pub struct Theme {
    pub root: Style,
    pub even: Style,
    pub odd: Style,
    pub selection: Style,
}

pub const THEME: Theme = Theme {
    root: Style::new().bg(SLATE.c900),
    even: Style::new().bg(SLATE.c900).fg(SLATE.c50),
    odd: Style::new().bg(SLATE.c800).fg(SLATE.c50),
    selection: Style::new().bg(Color::Rgb(30, 129, 176)).fg(SLATE.c50),
};

fn demo_text_list() -> List<'static, TextContainer> {
    let monday: Vec<String> = vec![
        String::from("1. Exercise for 30 minutes"),
        String::from("2. Work on the project for 2 hours"),
        String::from("3. Read a book for 1 hour"),
        String::from("4. Cook dinner"),
    ];
    let tuesday: Vec<String> = vec![
        String::from("1. Attend a team meeting at 10 AM"),
        String::from("2. Reply to emails"),
        String::from("3. Prepare lunch"),
    ];
    let wednesday: Vec<String> = vec![
        String::from("1. Update work tasks"),
        String::from("2. Conduct code review"),
        String::from("3. Attend a training"),
    ];
    let thursday: Vec<String> = vec![
        String::from("1. Brainstorm for an upcoming project"),
        String::from("2. Document ideas and refine tasks"),
    ];
    let friday: Vec<String> = vec![
        String::from("1. Have a one-on-one with a team lead"),
        String::from("2. Attent demo talk"),
        String::from("3. Go running for 1 hour"),
    ];
    let saturday: Vec<String> = vec![
        String::from("1. Work on a personal coding project for 2 hours"),
        String::from("2. Read a chapter from a book"),
        String::from("3. Go for a short walk"),
    ];
    let sunday: Vec<String> = vec![
        String::from("1. Plan and outline goals for the upcoming week"),
        String::from("2. Attend an online workshop"),
        String::from("3. Go to dinner with friends"),
        String::from("4. Watch a movie"),
    ];
    List::new(vec![
        TextContainer::new("Monday", monday),
        TextContainer::new("Tuesday", tuesday),
        TextContainer::new("Wednesday", wednesday),
        TextContainer::new("Thursday", thursday),
        TextContainer::new("Friday", friday),
        TextContainer::new("Saturday", saturday),
        TextContainer::new("Sunday", sunday),
    ])
    .style(THEME.root)
}

fn demo_color_list() -> List<'static, ColoredContainer> {
    List::new(vec![
        ColoredContainer::new(Color::Rgb(255, 0, 255)), // Neon Pink
        ColoredContainer::new(Color::Rgb(0, 255, 255)), // Neon Cyan
        ColoredContainer::new(Color::Rgb(255, 255, 0)), // Neon Yellow
        ColoredContainer::new(Color::Rgb(102, 255, 102)), // Neon Green
        ColoredContainer::new(Color::Rgb(255, 102, 102)), // Neon Red
        ColoredContainer::new(Color::Rgb(153, 51, 255)), // Neon Purple
        ColoredContainer::new(Color::Rgb(255, 102, 204)), // Neon Magenta
        ColoredContainer::new(Color::Rgb(255, 255, 102)), // Neon Lemon
        ColoredContainer::new(Color::Rgb(51, 255, 153)), // Neon Turquoise
        ColoredContainer::new(Color::Rgb(102, 255, 255)), // Neon Aqua
    ])
    .scroll_direction(ScrollAxis::Horizontal)
}
