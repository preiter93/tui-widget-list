use ratatui::widgets::Widget;

use crate::ScrollAxis;

/// Represents an item in a list widget.
///
/// This trait should be implemented on widget list items to be used in a `List`.
pub trait ListableWidget: Widget {
    /// Callback invoked when rendering the widget.
    ///
    /// This method is called during rendering to allow the widget to mutate itself based
    /// on additional render info.
    ///
    /// Return the main axis size of the widget.
    fn on_render(&mut self, render_info: &RenderInfo) -> u16;
}

/// Information provided during rendering.
pub struct RenderInfo {
    /// Cross axis size of the widget.
    pub cross_axis_size: u16,

    /// Indicates whether the widget should be rendered as highlighted.
    pub highlighted: bool,

    /// The scroll axis:
    ///
    /// - `vertical` (default)
    /// - `horizontal`
    pub scroll_axis: ScrollAxis,
}
