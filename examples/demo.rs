use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use ratatui::Terminal;
use std::error::Error;
use std::io::{stdout, Stdout};
use tui_widget_list::{List, ListState, PreRender, PreRenderContext, ScrollAxis};

#[derive(Debug, Clone)]
pub struct TextContainer {
    title: String,
    content: Vec<String>,
    style: Style,
    selected_color: Color,
    expand: bool,
}

impl TextContainer {
    pub fn new(title: &str, content: Vec<String>, selected_color: Color) -> Self {
        Self {
            title: title.to_string(),
            content,
            style: Style::default(),
            selected_color,
            expand: false,
        }
    }
}

impl PreRender for TextContainer {
    fn pre_render(&mut self, context: &PreRenderContext) -> u16 {
        if context.index % 2 == 0 {
            self.style = Style::default().bg(Color::Rgb(28, 28, 32));
        } else {
            self.style = Style::default().bg(Color::Rgb(0, 0, 0));
        }

        let mut main_axis_size = 2;
        if context.is_selected {
            self.style = Style::default()
                .bg(self.selected_color)
                .fg(Color::Rgb(28, 28, 32));
            self.expand = true;
            main_axis_size = 3 + self.content.len() as u16;
        }

        main_axis_size
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
impl PreRender for ColoredContainer {
    fn pre_render(&mut self, context: &PreRenderContext) -> u16 {
        if context.is_selected {
            self.border_style = Style::default().fg(Color::Black);
            self.border_type = BorderType::Thick;
        }

        15
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

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        use Constraint::{Min, Percentage};
        let [top, bottom] = Layout::vertical([Percentage(75), Min(0)]).areas(area);
        let colors = demo_colors();
        let selected_color = match self.color_list_state.selected {
            Some(index) => colors[index],
            None => colors[1],
        };

        let text_list = demo_text_list(selected_color);
        text_list.render(top, buf, &mut self.text_list_state);

        let color_list = List::new(
            colors
                .into_iter()
                .map(|color| ColoredContainer::new(color))
                .collect(),
        )
        .scroll_direction(ScrollAxis::Horizontal);
        color_list.render(bottom, buf, &mut self.color_list_state);
    }
}

fn demo_text_list(selected_color: Color) -> List<'static, TextContainer> {
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
        String::from("1. Have a recap meeting"),
        String::from("2. Attent conference talk"),
        String::from("3. Go running for 1 hour"),
    ];
    let saturday: Vec<String> = vec![
        String::from("1. Work on coding project"),
        String::from("2. Read a chapter from a book"),
        String::from("3. Go for a short walk"),
    ];
    let sunday: Vec<String> = vec![
        String::from("1. Plan upcoming trip"),
        String::from("2. Read in the park"),
        String::from("3. Go to dinner with friends"),
    ];
    List::new(vec![
        TextContainer::new("Monday", monday, selected_color),
        TextContainer::new("Tuesday", tuesday, selected_color),
        TextContainer::new("Wednesday", wednesday, selected_color),
        TextContainer::new("Thursday", thursday, selected_color),
        TextContainer::new("Friday", friday, selected_color),
        TextContainer::new("Saturday", saturday, selected_color),
        TextContainer::new("Sunday", sunday, selected_color),
    ])
    .style(Style::default())
}

fn demo_colors() -> Vec<Color> {
    vec![
        Color::Rgb(255, 102, 102), // Neon Red
        Color::Rgb(255, 153, 0),   // Neon Orange
        Color::Rgb(255, 204, 0),   // Neon Yellow
        Color::Rgb(0, 204, 102),   // Neon Green
        Color::Rgb(0, 204, 255),   // Neon Blue
        Color::Rgb(102, 51, 255),  // Neon Purple
        Color::Rgb(255, 51, 204),  // Neon Magenta
        Color::Rgb(51, 255, 255),  // Neon Cyan
        Color::Rgb(255, 102, 255), // Neon Pink
        Color::Rgb(102, 255, 255), // Neon Aqua
    ]
}
