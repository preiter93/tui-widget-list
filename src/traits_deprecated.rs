use ratatui::widgets::Widget;

use crate::ListWidget;
use crate::ScrollAxis;

/// Represents an item in a list widget.
///
/// This trait should be implemented on widget list items to be used in a `List`.
/// Implementors should provide information about the item's size along the lists
/// main axis and optionally support highlighting.
#[deprecated(
    since = "0.9.0",
    note = "Implement the ListWidget trait instead. See tui-widget-lists Changelog for a migration guide."
)]
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

#[allow(deprecated, clippy::cast_possible_truncation)]
impl<T: ListableWidget> ListWidget for T {
    fn pre_render(mut self, context: &crate::PreRenderContext) -> (Self, u16) {
        let main_axis_size = self.size(&context.scroll_axis) as u16;

        if context.is_selected {
            self = self.highlight();
        }

        (self, main_axis_size)
    }
}
