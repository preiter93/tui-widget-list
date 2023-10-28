use ratatui::widgets::Widget;

/// Should be implemented on widget list items to be used in `List`.
pub trait Listable: Widget {
    /// Returns the height of the item.
    fn height(&self) -> usize;

    /// Highlight the selected widget. Optional.
    #[must_use]
    fn highlight(self) -> Self
    where
        Self: Sized,
    {
        self
    }
}
