use std::collections::HashMap;

use ratatui_core::layout::Rect;
use ratatui_widgets::scrollbar::ScrollbarState;

use crate::{ListBuildContext, ListBuilder, ScrollAxis, ScrollDirection};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct ListState {
    /// The selected item. If `None`, no item is currently selected.
    pub selected: Option<usize>,

    /// The total number of elements in the list. This is necessary to correctly
    /// handle item selection.
    pub(crate) num_elements: usize,

    /// Indicates if the selection is circular. If true, calling `next` on the last
    /// element returns the first, and calling `previous` on the first returns the last.
    ///
    /// True by default.
    pub(crate) infinite_scrolling: bool,

    /// Scroll offset within the currently selected item. When an item is larger
    /// than the viewport, this tracks how far we've scrolled into it.
    pub(crate) item_scroll: u16,

    /// The state for the viewport. Keeps track which item to show
    /// first and how much it is truncated.
    pub(crate) view_state: ViewState,

    /// The scrollbar state. This is only used if the view is
    /// initialzed with a scrollbar.
    pub(crate) scrollbar_state: ScrollbarState,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ViewState {
    /// The index of the first item displayed on the screen.
    pub(crate) offset: usize,

    /// The truncation in rows/columns of the first item displayed on the screen.
    pub(crate) first_truncated: u16,

    /// Cached visible sizes from the last render: map from item index to its visible main-axis size.
    /// This avoids re-evaluating the builder for hit testing and other post-render queries.
    pub(crate) visible_main_axis_sizes: HashMap<usize, u16>,

    /// The inner area used during the last render (after applying the optional block).
    pub(crate) inner_area: Rect,

    /// The scroll axis used during the last render.
    pub(crate) scroll_axis: ScrollAxis,

    /// The scroll direction used during the last render.
    pub(crate) scroll_direction: ScrollDirection,

    /// The viewport's main axis size from the last render.
    pub(crate) last_main_axis_size: u16,

    /// Full (untruncated) main-axis sizes of items from the last render.
    pub(crate) total_main_axis_sizes: HashMap<usize, u16>,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            offset: 0,
            first_truncated: 0,
            visible_main_axis_sizes: HashMap::new(),
            inner_area: Rect::default(),
            scroll_axis: ScrollAxis::Vertical,
            scroll_direction: ScrollDirection::Forward,
            last_main_axis_size: 0,
            total_main_axis_sizes: HashMap::new(),
        }
    }
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            selected: None,
            num_elements: 0,
            infinite_scrolling: true,
            item_scroll: 0,
            view_state: ViewState::default(),
            scrollbar_state: ScrollbarState::new(0).position(0),
        }
    }
}

impl ListState {
    pub(crate) fn set_infinite_scrolling(&mut self, infinite_scrolling: bool) {
        self.infinite_scrolling = infinite_scrolling;
    }

    /// Returns the index of the currently selected item, if any.
    #[must_use]
    #[deprecated(since = "0.9.0", note = "Use ListState's selected field instead.")]
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Selects an item by its index.
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        self.item_scroll = 0;
        if index.is_none() {
            self.view_state.offset = 0;
            self.scrollbar_state = self.scrollbar_state.position(0);
        }
    }

    /// Selects the next element of the list. If circular is true,
    /// calling next on the last element selects the first.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tui_widget_list::ListState;
    ///
    /// let mut list_state = ListState::default();
    /// list_state.next();
    /// ```
    pub fn next(&mut self) {
        if self.num_elements == 0 {
            return;
        }
        // If the current item overflows the viewport, scroll within it first
        if let Some(selected) = self.selected {
            let overflow = self.item_overflow(selected);
            if overflow > 0 && self.item_scroll < overflow {
                self.item_scroll += 1;
                return;
            }
        }
        let i = match self.selected {
            Some(i) => {
                if i >= self.num_elements - 1 {
                    if self.infinite_scrolling {
                        0
                    } else {
                        i
                    }
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.select(Some(i));
    }

    /// Selects the previous element of the list. If circular is true,
    /// calling previous on the first element selects the last.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tui_widget_list::ListState;
    ///
    /// let mut list_state = ListState::default();
    /// list_state.previous();
    /// ```
    pub fn previous(&mut self) {
        if self.num_elements == 0 {
            return;
        }
        // If the current item overflows the viewport, scroll back within it first
        if self.item_scroll > 0 {
            self.item_scroll -= 1;
            return;
        }
        let i = match self.selected {
            Some(i) => {
                if i == 0 {
                    if self.infinite_scrolling {
                        self.num_elements - 1
                    } else {
                        i
                    }
                } else {
                    i - 1
                }
            }
            None => self.num_elements - 1,
        };
        // If the previous item overflows the viewport, start at its bottom
        let overflow = self.item_overflow(i);
        if overflow > 0 {
            self.selected = Some(i);
            self.item_scroll = overflow;
            return;
        }
        self.select(Some(i));
    }

    /// Returns the index of the first item currently displayed on the screen.
    #[must_use]
    pub fn scroll_offset_index(&self) -> usize {
        self.view_state.offset
    }

    /// Returns the number of rows/columns of the first visible item that are scrolled off the top/left.
    ///
    /// When the first visible item is partially scrolled out of view, this returns how many
    /// rows (for vertical lists) or columns (for horizontal lists) are hidden above/left of
    /// the viewport. Returns 0 if the first visible item is fully visible.
    ///
    /// # Example
    ///
    /// If message #5 is the first visible item but its first 2 rows are scrolled off the top,
    /// this returns 2. Combined with `scroll_offset_index()`, you can calculate the exact
    /// scroll position in pixels/rows.
    #[must_use]
    pub fn scroll_truncation(&self) -> u16 {
        self.view_state.first_truncated
    }

    /// Updates the number of elements that are present in the list.
    pub(crate) fn set_num_elements(&mut self, num_elements: usize) {
        self.num_elements = num_elements;
    }

    /// Updates the current scrollbar content length and position.
    pub(crate) fn update_scrollbar_state<T>(
        &mut self,
        builder: &ListBuilder<T>,
        item_count: usize,
        main_axis_size: u16,
        cross_axis_size: u16,
        scroll_axis: ScrollAxis,
    ) {
        let mut max_scrollbar_position = 0;
        let mut cumulative_size = 0;

        for index in (0..item_count).rev() {
            let context = ListBuildContext {
                index,
                is_selected: self.selected == Some(index),
                scroll_axis,
                cross_axis_size,
            };
            let (_, widget_size) = builder.call_closure(&context);
            cumulative_size += widget_size;

            if cumulative_size > main_axis_size {
                max_scrollbar_position = index + 1;
                break;
            }
        }

        self.scrollbar_state = self.scrollbar_state.content_length(max_scrollbar_position);
        self.scrollbar_state = self.scrollbar_state.position(self.view_state.offset);
    }

    /// Replace the cached visible sizes with a new map computed during render.
    /// The values should be the actually visible size (after truncation) along the main axis.
    pub(crate) fn set_visible_main_axis_sizes(&mut self, sizes: HashMap<usize, u16>) {
        self.view_state.visible_main_axis_sizes = sizes;
    }

    /// Get a reference to the cached visible sizes map from the last render.
    #[must_use]
    pub(crate) fn visible_main_axis_sizes(&self) -> &HashMap<usize, u16> {
        &self.view_state.visible_main_axis_sizes
    }

    /// Set the inner area used during the last render.
    pub(crate) fn set_inner_area(&mut self, inner_area: Rect) {
        self.view_state.inner_area = inner_area;
    }

    /// Get the inner area used during the last render.
    #[must_use]
    pub(crate) fn inner_area(&self) -> Rect {
        self.view_state.inner_area
    }

    /// Set the scroll axis used during the last render.
    pub(crate) fn set_scroll_axis(&mut self, scroll_axis: ScrollAxis) {
        self.view_state.scroll_axis = scroll_axis;
    }

    /// Get the scroll axis used during the last render.
    #[must_use]
    pub(crate) fn last_scroll_axis(&self) -> ScrollAxis {
        self.view_state.scroll_axis
    }

    /// Set the scroll direction used during the last render.
    pub(crate) fn set_scroll_direction(&mut self, scroll_direction: ScrollDirection) {
        self.view_state.scroll_direction = scroll_direction;
    }

    /// Get the scroll direction used during the last render.
    #[must_use]
    pub(crate) fn last_scroll_direction(&self) -> ScrollDirection {
        self.view_state.scroll_direction
    }

    /// Returns how many rows/cols of the item extend beyond the viewport,
    /// or 0 if the item fits or its size is not cached.
    fn item_overflow(&self, index: usize) -> u16 {
        self.view_state
            .total_main_axis_sizes
            .get(&index)
            .map(|&total| total.saturating_sub(self.view_state.last_main_axis_size))
            .unwrap_or(0)
    }

    /// Set the viewport's main axis size from the last render.
    pub(crate) fn set_last_main_axis_size(&mut self, size: u16) {
        self.view_state.last_main_axis_size = size;
    }

    /// Set the full (untruncated) main-axis sizes of items from the last render.
    pub(crate) fn set_total_main_axis_sizes(&mut self, sizes: HashMap<usize, u16>) {
        self.view_state.total_main_axis_sizes = sizes;
    }

    /// Get the scroll offset within the currently selected item.
    #[must_use]
    pub(crate) fn item_scroll(&self) -> u16 {
        self.item_scroll
    }
}
