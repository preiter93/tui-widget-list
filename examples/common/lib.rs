#![allow(unused_imports, dead_code)]
pub mod item_container;
use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{BorderType, Borders, StatefulWidget},
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
        ratatui::crossterm::execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
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
        ratatui::crossterm::execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

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
