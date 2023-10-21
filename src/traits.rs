use ratatui::prelude::{Buffer, Rect};

/// Should be implemented on widget list items to be used in `WidgetList`.
pub trait WidgetItem {
    /// Returns the height of the item.
    fn height(&self) -> usize;

    /// Highlight the selected widget. Optional.
    #[must_use]
    fn highlighted(&self) -> Self;

    // Render the item. Similar to ratatui's widget trait, but here the
    // render trait should be implemented on the reference of the class.
    fn render(&self, area: Rect, buf: &mut Buffer);
}
