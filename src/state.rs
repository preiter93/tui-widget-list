use std::collections::HashMap;

use ratatui_core::layout::Rect;
use ratatui_widgets::scrollbar::ScrollbarState;

use crate::{ListBuildContext, ListBuilder, ScrollAxis};

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

    /// When true, the next render will adjust the viewport to ensure the
    /// selected item is visible. Set by `select_next(true)` / `select_previous(true)`
    /// and cleared after the viewport layout pass.
    pub(crate) scroll_to_selected: bool,

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

    /// Pending scroll delta in rows/columns, resolved by `layout_on_viewport`.
    /// Positive values scroll forward (down/right), negative values scroll
    /// backward (up/left).
    pub(crate) pending_scroll: i16,

    /// Cached visible sizes from the last render: map from item index to its visible main-axis size.
    /// This avoids re-evaluating the builder for hit testing and other post-render queries.
    pub(crate) visible_main_axis_sizes: HashMap<usize, u16>,

    /// The inner area used during the last render (after applying the optional block).
    pub(crate) inner_area: Rect,

    /// The scroll axis used during the last render.
    pub(crate) scroll_axis: ScrollAxis,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            offset: 0,
            first_truncated: 0,
            pending_scroll: 0,
            visible_main_axis_sizes: HashMap::new(),
            inner_area: Rect::default(),
            scroll_axis: ScrollAxis::Vertical,
        }
    }
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            selected: None,
            num_elements: 0,
            infinite_scrolling: true,
            scroll_to_selected: false,
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

    /// Selects an item by its index. The viewport will be adjusted
    /// on the next render to ensure the selected item is visible.
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        self.scroll_to_selected = index.is_some();
        if index.is_none() {
            self.view_state.offset = 0;
            self.scrollbar_state = self.scrollbar_state.position(0);
        }
    }

    /// Selects the next element of the list and scrolls the viewport
    /// to keep it visible. If circular is true, calling next on the
    /// last element selects the first.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tui_widget_list::ListState;
    ///
    /// let mut list_state = ListState::default();
    /// list_state.next();
    /// ```
    #[deprecated(since = "0.15.0", note = "Use `select_next()` instead.")]
    pub fn next(&mut self) {
        self.select_next(true);
    }

    /// Selects the previous element of the list and scrolls the viewport
    /// to keep it visible. If circular is true, calling previous on the
    /// first element selects the last.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tui_widget_list::ListState;
    ///
    /// let mut list_state = ListState::default();
    /// list_state.previous();
    /// ```
    #[deprecated(since = "0.15.0", note = "Use `select_previous()` instead.")]
    pub fn previous(&mut self) {
        self.select_previous(true);
    }

    /// Selects the next element of the list. If circular is true,
    /// calling `select_next` on the last element selects the first.
    ///
    /// When `scroll_to` is true, the viewport will be adjusted on the
    /// next render to ensure the selected item is visible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tui_widget_list::ListState;
    ///
    /// let mut list_state = ListState::default();
    /// list_state.select_next(true);
    /// ```
    pub fn select_next(&mut self, scroll_to: bool) {
        if self.num_elements == 0 {
            return;
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
        self.scroll_to_selected = scroll_to;
    }

    /// Selects the previous element of the list. If circular is true,
    /// calling `select_previous` on the first element selects the last.
    ///
    /// When `scroll_to` is true, the viewport will be adjusted on the
    /// next render to ensure the selected item is visible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tui_widget_list::ListState;
    ///
    /// let mut list_state = ListState::default();
    /// list_state.select_previous(true);
    /// ```
    pub fn select_previous(&mut self, scroll_to: bool) {
        if self.num_elements == 0 {
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
            None => 0,
        };
        self.select(Some(i));
        self.scroll_to_selected = scroll_to;
    }

    /// Scrolls the viewport down by one row/column without changing the selection.
    pub fn scroll_down(&mut self) {
        self.scroll_by(1);
    }

    /// Scrolls the viewport up by one row/column without changing the selection.
    pub fn scroll_up(&mut self) {
        self.scroll_by(-1);
    }

    /// Scrolls the viewport by `delta` rows/columns without changing the selection.
    /// Positive values scroll forward (down/right), negative values scroll
    /// backward (up/left).
    pub fn scroll_by(&mut self, delta: i16) {
        self.view_state.pending_scroll = self.view_state.pending_scroll.saturating_add(delta);
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
}
