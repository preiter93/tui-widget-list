use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Styled},
    widgets::{Padding, Widget},
};

pub struct ListItemContainer<'a, W> {
    child: W,
    block: ratatui::widgets::Block<'a>,
    style: Style,
}

impl<'a, W> ListItemContainer<'a, W> {
    pub fn new(child: W, padding: Padding) -> Self {
        Self {
            child,
            block: ratatui::widgets::Block::default().padding(padding),
            style: Style::default(),
        }
    }
}

impl<T> Styled for ListItemContainer<'_, T> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

impl<W: Widget> Widget for ListItemContainer<'_, W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_area = self.block.inner(area);
        buf.set_style(area, self.style);
        self.block.render(area, buf);
        self.child.render(inner_area, buf);
    }
}
