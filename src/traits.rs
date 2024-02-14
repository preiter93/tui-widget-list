use ratatui::widgets::Widget;

/// Represents an item in a list widget.
///
/// This trait should be implemented on widget list items to be used in a `List`.
/// Implementors should provide information about the item's size along the lists
/// main axis and optionally support highlighting.
pub trait ListableWidget: Widget {
    /// Returns the main axis size of the item.
    ///
    /// For vertical lists, this represents the height, and for horizontal lists,
    /// this represents the width.
    fn main_axis_size(&self) -> usize;

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

/// Should be implemented on widget list items to be used in `List`.
#[deprecated(
    since = "0.7.2",
    note = "Use ListableWidget trait instead, with method main_axis_size replacing height."
)]
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

#[allow(deprecated)]
impl<T: Listable> ListableWidget for T {
    fn main_axis_size(&self) -> usize {
        self.height()
    }

    fn highlight(self) -> Self
    where
        Self: Sized,
    {
        self.highlight()
    }
}
