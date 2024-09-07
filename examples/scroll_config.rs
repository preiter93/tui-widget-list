#[path = "common/lib.rs"]
mod common;
use common::{Block, Colors, PaddedLine, Result, Terminal};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    text::Line,
    widgets::{Padding, StatefulWidget},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

fn main() -> Result<()> {
    let mut terminal = Terminal::init()?;
    App::default().run(&mut terminal).unwrap();

    Terminal::reset()?;
    terminal.show_cursor()?;

    Ok(())
}

#[derive(Default, Clone)]
pub struct App;

#[derive(Default)]
pub struct AppState {
    selected_tab: Tab,
    scroll_config_state: ListState,
    list_state: ListState,
}

impl AppState {
    fn new() -> Self {
        let mut scroll_config_state = ListState::default();
        scroll_config_state.select(Some(0));
        Self {
            scroll_config_state,
            ..AppState::default()
        }
    }
}

#[derive(PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Selection,
    List,
}

impl Tab {
    fn next(&mut self) {
        match self {
            Self::Selection => *self = Tab::List,
            Self::List => *self = Tab::Selection,
        }
    }
}

#[derive(PartialEq, Eq, Default, Clone)]
enum ScrollConfig {
    #[default]
    Default,
    // Fixed,
}
impl ScrollConfig {
    pub const COUNT: usize = 1;
    pub fn from_index(index: usize) -> Self {
        match index {
            // 1 => ScrollConfig::Fixed,
            _ => ScrollConfig::Default,
        }
    }
}

impl std::fmt::Display for ScrollConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrollConfig::Default => write!(f, "Default"),
            // ScrollConfig::Fixed => write!(f, "Fixed"),
        }
    }
}

impl App {
    pub fn run(&self, terminal: &mut Terminal) -> Result<()> {
        let mut state = AppState::new();
        loop {
            terminal.draw_app(self, &mut state)?;
            if Self::handle_events(&mut state)? {
                return Ok(());
            }
        }
    }

    /// Handles app events.
    /// Returns true if the app should quit.
    fn handle_events(state: &mut AppState) -> Result<bool> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let list_state = match state.selected_tab {
                    Tab::Selection => &mut state.scroll_config_state,
                    Tab::List => &mut state.list_state,
                };
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Up | KeyCode::Char('k') => list_state.previous(),
                    KeyCode::Down | KeyCode::Char('j') => list_state.next(),
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') => {
                        state.selected_tab.next()
                    }
                    _ => {}
                }
            }
            return Ok(false);
        }
        return Ok(false);
    }
}

impl StatefulWidget for &App {
    type State = AppState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        use Constraint::{Min, Percentage};
        let [left, right] = Layout::horizontal([Percentage(25), Min(0)]).areas(area);

        // Scroll config selection
        let block = match state.selected_tab {
            Tab::Selection => Block::selected(),
            _ => Block::disabled(),
        };
        ConfigListView::new()
            .block(block)
            .render(left, buf, &mut state.scroll_config_state);

        // List demo
        let block = match state.selected_tab {
            Tab::List => Block::selected(),
            _ => Block::disabled(),
        };
        DemoListView::new()
            .block(block)
            .render(right, buf, &mut state.list_state);
    }
}
struct ConfigListView;
impl ConfigListView {
    fn new<'a>() -> ListView<'a, PaddedLine<'a>> {
        let builder = ListBuilder::new(move |context| {
            let config = ScrollConfig::from_index(context.index);
            let line = Line::from(format!("{config}")).alignment(Alignment::Center);
            let mut item = PaddedLine::new(line, Padding::vertical(1));

            if context.is_selected {
                item.style = item.style.bg(Colors::ORANGE).fg(Colors::CHARCOAL);
            };

            return (item, 3);
        });

        return ListView::new(builder, ScrollConfig::COUNT);
    }
}

struct DemoListView;
impl DemoListView {
    fn new<'a>() -> ListView<'a, PaddedLine<'a>> {
        let builder = ListBuilder::new(|context| {
            let line = Line::from(format!("Item {0}", context.index)).alignment(Alignment::Center);
            let mut item = PaddedLine::new(line, Padding::vertical(1));

            if context.index % 2 == 0 {
                item.style = item.style.bg(Colors::CHARCOAL);
            } else {
                item.style = item.style.bg(Colors::BLACK);
            };

            if context.is_selected {
                item.style = item.style.bg(Colors::ORANGE).fg(Colors::CHARCOAL);
            };

            return (item, 3);
        });

        return ListView::new(builder, 100);
    }
}
