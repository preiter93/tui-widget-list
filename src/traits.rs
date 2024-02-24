use ratatui::widgets::Widget;

use crate::ScrollAxis;

/// Represents an item in a list widget.
///
/// This trait should be implemented on widget list items to be used in a `List`.
/// Implementors should provide information about the item's size along the lists
/// main axis and optionally support highlighting.
pub trait ListableWidget: Widget {
    /// Returns the size of the item based on the scroll direction.
    ///
    /// This method should return the height for vertical lists and the width for horizontal lists.
    ///
    /// The `scroll_direction` parameter allows specifying different sizes depending on the list's scroll axis.
    /// In most cases, this parameter can be ignored.
    fn size(&self, scroll_direction: &ScrollAxis) -> usize;

    /// Highlight the selected widget. Optional.
    ///
    /// This method should return a new instance of the widget with modifications
    /// to render it highlighted.
    #[must_use]
    fn highlight(self) -> Self
    where
        Self: Sized,
    {
        self
    }
}
