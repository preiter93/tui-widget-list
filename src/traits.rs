use ratatui::widgets::Widget;

/// Should be implemented on widget list items to be used in `List`.
pub trait Listable: Widget {
    /// Returns the height of the item.
    fn height(&self) -> usize;

    /// Highlight the selected widget. Optional. If None, no highlighting
    /// is applied.
    #[must_use]
    fn highlight(self) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}
