use crate::common::{item_container::ListItemContainer, Colors};
use ratatui::{
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Padding, Scrollbar},
};
use tui_widget_list::{ListBuilder, ListView};

const FILES: &[(&str, &str)] = &[
    (".cargo/", "—"),
    (".github/", "—"),
    ("assets/", "—"),
    ("benches/", "—"),
    ("ci/", "—"),
    ("crates/", "—"),
    ("docs/", "—"),
    ("examples/", "—"),
    ("fixtures/", "—"),
    ("migrations/", "—"),
    ("scripts/", "—"),
    ("src/", "—"),
    ("target/", "—"),
    ("tests/", "—"),
    ("tmp/", "—"),
    (".clippy.toml", "0.1 KB"),
    (".editorconfig", "0.2 KB"),
    (".env.example", "0.3 KB"),
    (".gitignore", "0.1 KB"),
    (".rustfmt.toml", "0.1 KB"),
    ("CHANGELOG.md", "4.8 KB"),
    ("CONTRIBUTING.md", "2.1 KB"),
    ("Cargo.lock", "12 KB"),
    ("Cargo.toml", "0.4 KB"),
    ("LICENSE", "1.1 KB"),
    ("Makefile", "0.3 KB"),
    ("README.md", "3.2 KB"),
    ("SECURITY.md", "1.0 KB"),
    ("build.rs", "0.8 KB"),
    ("cliff.toml", "0.5 KB"),
    ("config.yaml", "1.5 KB"),
    ("deny.toml", "0.9 KB"),
    ("flake.nix", "1.2 KB"),
    ("justfile", "0.6 KB"),
    ("release.toml", "0.3 KB"),
    ("rust-toolchain", "0.1 KB"),
    ("taplo.toml", "0.2 KB"),
];

pub(crate) struct ClassicListView;

impl ClassicListView {
    pub(crate) fn new<'a>() -> ListView<'a, ListItemContainer<'a, Line<'a>>> {
        let builder = ListBuilder::new(|context| {
            let (name, size) = FILES[context.index];
            let is_dir = name.ends_with('/');
            let width = context.cross_axis_size.saturating_sub(4) as usize;
            let name_part = format!(" {name:<0$}", width.saturating_sub(8));
            let size_part = format!("{size:>8}");

            let name_color = if context.is_selected {
                Colors::CHARCOAL
            } else if is_dir {
                Color::Rgb(100, 149, 237)
            } else {
                Colors::WHITE
            };

            let line = Line::from(vec![
                Span::styled(name_part, ratatui::style::Style::default().fg(name_color)),
                Span::from(size_part),
            ]);

            let mut item = ListItemContainer::new(line, Padding::vertical(0));
            item = match context.is_selected {
                true => item.bg(Colors::ORANGE).fg(Colors::CHARCOAL),
                false if context.index % 2 == 0 => item.bg(Colors::CHARCOAL),
                false => item.bg(Colors::BLACK),
            };

            (item, 1)
        });

        ListView::new(builder, FILES.len())
            .infinite_scrolling(false)
            .scrollbar(Scrollbar::default())
    }
}
