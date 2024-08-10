use ratatui::{
    style::{Style, Styled},
    widgets::Block,
};

/// A struct representing a list view.
/// The widget displays a scrollable list of items.
pub struct ListView<'a, T> {
    /// The total number of items in the list
    pub item_count: usize,

    ///  A `ListBuilder<T>` responsible for constructing the items in the list.
    pub builder: ListBuilder<T>,

    /// Specifies the scroll axis. Either `Vertical` or `Horizontal`.
    pub scroll_axis: ScrollAxis,

    /// The base style of the list view.
    pub style: Style,

    /// The base block surrounding the widget list.
    pub block: Option<Block<'a>>,
}

impl<'a, T> ListView<'a, T> {
    /// Creates a new `ListView` with a builder an item count.
    #[must_use]
    pub fn new(builder: ListBuilder<T>, item_count: usize) -> Self {
        Self {
            builder,
            item_count,
            scroll_axis: ScrollAxis::Vertical,
            style: Style::default(),
            block: None,
        }
    }

    /// Sets the block style that surrounds the whole List.
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Checks whether the widget list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.item_count == 0
    }

    /// Returns the length of the widget list.
    #[must_use]
    pub fn len(&self) -> usize {
        self.item_count
    }

    /// Set the base style of the List.
    #[must_use]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Set the scroll axis of the list.
    #[must_use]
    pub fn scroll_axis(mut self, scroll_axis: ScrollAxis) -> Self {
        self.scroll_axis = scroll_axis;
        self
    }
}

impl<T> Styled for ListView<'_, T> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

impl<T: Copy + 'static> From<Vec<T>> for ListView<'_, T> {
    fn from(value: Vec<T>) -> Self {
        let item_count = value.len();
        let builder = ListBuilder::new(move |context| {
            return (value[context.index], 1);
        });

        ListView::new(builder, item_count)
    }
}

/// This structure holds information about the item's position, selection
/// status, scrolling behavior, and size along the cross axis.
pub struct ListBuildContext {
    /// The position of the item in the list.
    pub index: usize,

    /// A boolean flag indicating whether the item is currently selected.
    pub is_selected: bool,

    /// Defines the axis along which the list can be scrolled.
    pub scroll_axis: ScrollAxis,

    /// The size of the item along the cross axis.
    pub cross_axis_size: u16,
}

/// A type alias for the closure.
type ListBuilderClosure<T> = dyn Fn(&ListBuildContext) -> (T, u16);

/// The builder to for constructing list elements in a `ListView<T>`
pub struct ListBuilder<T> {
    closure: Box<ListBuilderClosure<T>>,
}

impl<T> ListBuilder<T> {
    /// Creates a new `ListBuilder` taking a closure as a parameter
    pub fn new<F>(closure: F) -> Self
    where
        F: Fn(&ListBuildContext) -> (T, u16) + 'static,
    {
        ListBuilder {
            closure: Box::new(closure),
        }
    }

    /// Method to call the stored closure.
    pub(crate) fn call_closure(&self, context: &ListBuildContext) -> (T, u16) {
        (self.closure)(context)
    }
}

/// Represents the scroll axis of a list.
#[derive(Debug, Default, Clone, Copy)]
pub enum ScrollAxis {
    /// Indicates vertical scrolling. This is the default.
    #[default]
    Vertical,

    /// Indicates horizontal scrolling.
    Horizontal,
}
