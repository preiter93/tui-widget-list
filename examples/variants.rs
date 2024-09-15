#[path = "variants/classic.rs"]
mod classic;
#[path = "common/lib.rs"]
mod common;
#[path = "variants/config.rs"]
mod config;
#[path = "variants/fps.rs"]
mod fps;
#[path = "variants/horizontal.rs"]
mod horizontal;
#[path = "variants/scroll_padding.rs"]
mod scroll_padding;
use classic::PaddedListView;
use common::{Block, Colors, Result, Terminal};
use config::{Controls, Variant, VariantsListView};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use fps::FPSCounter;
use horizontal::HorizontalListView;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{StatefulWidget, Widget},
};
use scroll_padding::ScrollPaddingListView;
use tui_widget_list::ListState;

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
    variant_state: ListState,
    list_state: ListState,
    fps_counter: FPSCounter,
}

impl AppState {
    fn new() -> Self {
        let mut scroll_config_state = ListState::default();
        scroll_config_state.select(Some(0));
        Self {
            variant_state: scroll_config_state,
            list_state: ListState::default().circular(false),
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

impl App {
    pub fn run(&self, terminal: &mut Terminal) -> Result<()> {
        let mut state = AppState::new();
        loop {
            terminal.draw_app(self, &mut state)?;
            if Self::handle_events(&mut state)? {
                return Ok(());
            }
            state.fps_counter.update();
        }
    }

    /// Handles app events.
    /// Returns true if the app should quit.
    fn handle_events(state: &mut AppState) -> Result<bool> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let list_state = match state.selected_tab {
                    Tab::Selection => &mut state.variant_state,
                    Tab::List => &mut state.list_state,
                };
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Up | KeyCode::Char('k') => list_state.previous(),
                    KeyCode::Down | KeyCode::Char('j') => list_state.next(),
                    KeyCode::Char('f') => state.fps_counter.toggle(),
                    KeyCode::Tab
                    | KeyCode::Left
                    | KeyCode::Char('h')
                    | KeyCode::Right
                    | KeyCode::Char('l') => {
                        state.list_state = state.list_state.clone().circular(false);
                        state.list_state.select(None);
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
        use Constraint::{Length, Min, Percentage};

        let [top, main] = Layout::vertical([Length(1), Min(0)]).areas(area);
        let [left, right] = Layout::horizontal([Percentage(25), Min(0)]).areas(main);

        // Key mappings
        let [top_left, top_right] = Layout::horizontal([Min(0), Length(10)]).areas(top);
        Controls::default().render(top_left, buf);
        state.fps_counter.render(top_right, buf);

        // Scroll config selection
        let block = match state.selected_tab {
            Tab::Selection => Block::selected(),
            _ => Block::disabled(),
        };
        VariantsListView::new()
            .block(block)
            .render(left, buf, &mut state.variant_state);

        // List demo
        let block = match state.selected_tab {
            Tab::List => Block::selected(),
            _ => Block::disabled(),
        };
        let fg = match state.selected_tab {
            Tab::List => Colors::WHITE,
            _ => Colors::GRAY,
        };
        match Variant::from_index(state.variant_state.selected.unwrap_or(0)) {
            Variant::Classic => {
                PaddedListView::new()
                    .block(block)
                    .fg(fg)
                    .render(right, buf, &mut state.list_state)
            }
            Variant::InfiniteScrolling => {
                state.list_state = state.list_state.clone().circular(true);
                PaddedListView::new()
                    .block(block)
                    .fg(fg)
                    .render(right, buf, &mut state.list_state)
            }
            Variant::ScrollPadding => ScrollPaddingListView::new().block(block).fg(fg).render(
                right,
                buf,
                &mut state.list_state,
            ),
            Variant::Horizontal => HorizontalListView::new().block(block).fg(fg).render(
                right,
                buf,
                &mut state.list_state,
            ),
        };
    }
}
