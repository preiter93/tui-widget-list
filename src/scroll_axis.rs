/// Represents the scroll axis of a list.
#[derive(Debug, Default, Clone, Copy)]
pub enum ScrollAxis {
    /// Indicates vertical scrolling. This is the default.
    #[default]
    Vertical,

    /// Indicates horizontal scrolling.
    Horizontal,
}
