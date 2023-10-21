use ratatui::widgets::Widget;

/// A Widget that can be used in a `WidgetList`.
pub trait WidgetItem: Widget {
    /// The height of the item
    fn height(&self) -> u16;

    /// Highlight the selected widget
    fn highlight(self) -> Self
    where
        Self: Sized;
}
