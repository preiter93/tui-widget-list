use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::Line,
    widgets::{BorderType, Borders, Padding, StatefulWidget, Widget},
};
use std::{
    error::Error,
    io::{stdout, Stdout},
    ops::{Deref, DerefMut},
};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct Colors;

impl Colors {
    pub const BLACK: Color = Color::Rgb(0, 0, 0);
    pub const WHITE: Color = Color::Rgb(255, 255, 255);
    pub const CHARCOAL: Color = Color::Rgb(28, 28, 32);
    pub const ORANGE: Color = Color::Rgb(255, 153, 0);
    pub const GRAY: Color = Color::Rgb(96, 96, 96);
    pub const TEAL: Color = Color::Rgb(0, 128, 128);
}

pub struct PaddedLine<'a> {
    pub line: Line<'a>,
    pub block: ratatui::widgets::Block<'a>,
    pub style: Style,
}

impl<'a> PaddedLine<'a> {
    pub fn new(line: Line<'a>, padding: Padding) -> Self {
        let block = ratatui::widgets::Block::default().padding(padding);
        let style = Style::default().fg(Color::White);
        Self { line, block, style }
    }
}

impl Widget for PaddedLine<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_area = self.block.inner(area);
        buf.set_style(area, self.style);
        self.block.render(area, buf);
        self.line.render(inner_area, buf);
    }
}

pub struct Block;
impl Block {
    pub fn disabled() -> ratatui::widgets::Block<'static> {
        return ratatui::widgets::Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Colors::GRAY));
    }

    pub fn selected() -> ratatui::widgets::Block<'static> {
        return ratatui::widgets::Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(Style::default().fg(Colors::WHITE));
    }
}

pub struct Terminal(ratatui::Terminal<CrosstermBackend<Stdout>>);

impl Deref for Terminal {
    type Target = ratatui::Terminal<CrosstermBackend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Terminal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Terminal {
    pub fn init() -> Result<Self> {
        crossterm::execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(stdout());

        let mut terminal = ratatui::Terminal::new(backend)?;
        terminal.hide_cursor()?;

        fn panic_hook() {
            let original_hook = std::panic::take_hook();

            std::panic::set_hook(Box::new(move |panic| {
                Terminal::reset().unwrap();
                original_hook(panic);
            }));
        }

        panic_hook();

        Ok(Self(terminal))
    }

    pub fn reset() -> Result<()> {
        disable_raw_mode()?;
        crossterm::execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

        Ok(())
    }

    pub fn draw_app<W>(&mut self, widget: W, state: &mut W::State) -> Result<()>
    where
        W: StatefulWidget,
    {
        self.0.draw(|frame| {
            frame.render_stateful_widget(widget, frame.area(), state);
        })?;
        Ok(())
    }
}
