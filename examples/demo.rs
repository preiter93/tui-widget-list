#[path = "common/lib.rs"]
mod common;
use common::{Colors, Result, Terminal};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::*,
    style::{Color, Style, Styled},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};
use tui_widget_list::{ListBuilder, ListState, ListView, ScrollAxis};

fn main() -> Result<()> {
    let mut terminal = Terminal::init()?;

    App::default().run(&mut terminal).unwrap();

    Terminal::reset()?;
    terminal.show_cursor()?;

    Ok(())
}

#[derive(Default)]
struct App {}

#[derive(Default)]
struct AppState {
    pub text_list_state: ListState,
    pub color_list_state: ListState,
}

impl App {
    pub fn run(&self, terminal: &mut Terminal) -> Result<()> {
        let mut state = AppState::default();
        loop {
            terminal.draw_app(self, &mut state)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Up | KeyCode::Char('k') => state.text_list_state.previous(),
                        KeyCode::Down | KeyCode::Char('j') => state.text_list_state.next(),
                        KeyCode::Left | KeyCode::Char('h') => state.color_list_state.previous(),
                        KeyCode::Right | KeyCode::Char('l') => state.color_list_state.next(),
                        _ => {}
                    }
                }
            }
        }
    }
}

impl StatefulWidget for &App {
    type State = AppState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        use Constraint::{Min, Percentage};
        let [top, bottom] = Layout::vertical([Percentage(75), Min(0)]).areas(area);

        // Text list
        let colors = demo_colors();
        let selected_color = match state.color_list_state.selected {
            Some(index) => colors[index],
            None => colors[1],
        };
        let text_list = TextContainer::demo(selected_color);
        text_list.render(top, buf, &mut state.text_list_state);

        // Color list
        let color_list = ColoredContainer::demo();
        color_list.render(bottom, buf, &mut state.color_list_state);
    }
}

#[derive(Debug, Clone)]
pub struct TextContainer {
    title: String,
    content: Vec<String>,
    style: Style,
    expand: bool,
}

impl Styled for TextContainer {
    type Item = Self;
    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

impl TextContainer {
    pub fn new(title: &str, content: Vec<String>) -> Self {
        Self {
            title: title.to_string(),
            content,
            style: Style::default(),
            expand: false,
        }
    }

    fn demo(selected_color: Color) -> ListView<'static, 'static, TextContainer> {
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
        let containers = vec![
            TextContainer::new("Monday", monday),
            TextContainer::new("Tuesday", tuesday),
            TextContainer::new("Wednesday", wednesday),
            TextContainer::new("Thursday", thursday),
            TextContainer::new("Friday", friday),
            TextContainer::new("Saturday", saturday),
            TextContainer::new("Sunday", sunday),
        ];

        let builder = ListBuilder::new(move |context| {
            let mut main_axis_size = 2;

            let mut container = containers[context.index].clone();

            if context.index % 2 == 0 {
                container.style = Style::default().bg(Colors::CHARCOAL);
            } else {
                container.style = Style::default().bg(Colors::BLACK);
            }

            if context.is_selected {
                container.style = Style::default().bg(selected_color).fg(Colors::CHARCOAL);
                container.expand = true;
                main_axis_size = 3 + container.content.len() as u16;
            }

            (container, main_axis_size)
        });

        ListView::new(builder, 7)
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

    fn demo() -> ListView<'static, 'static, ColoredContainer> {
        let colors = demo_colors();
        let builder = ListBuilder::new(move |context| {
            let color = demo_colors()[context.index];

            let mut widget = ColoredContainer::new(color);
            if context.is_selected {
                widget.border_style = Style::default().fg(Color::Black);
                widget.border_type = BorderType::Thick;
            };

            (widget, 15)
        });

        ListView::new(builder, colors.len()).scroll_axis(ScrollAxis::Horizontal)
    }
}

impl Widget for ColoredContainer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .borders(Borders::ALL)
            .border_style(self.border_style)
            .border_type(self.border_type)
            .bg(self.color)
            .render(area, buf);
    }
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
