#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct ListState {
    /// The selected item. If `None`, no item is currently selected.
    pub selected: Option<usize>,

    /// The total number of elements in the list. This is necessary to correctly
    /// handle item selection.
    pub(crate) num_elements: usize,

    /// Indicates if the selection is circular. If circular, calling `next` on the last
    /// element returns the first, and calling `previous` on the first returns the last.
    ///
    /// True by default.
    pub(crate) circular: bool,

    /// The state for the viewport. Keeps track which item to show
    /// first and how much it is truncated.
    pub(crate) view_state: ViewState,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct ViewState {
    /// The index of the first item displayed on the screen.
    pub(crate) offset: usize,

    /// The truncation in rows/columns of the first item displayed on the screen.
    pub(crate) first_truncated: u16,
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            selected: None,
            num_elements: 0,
            circular: true,
            view_state: ViewState::default(),
        }
    }
}

impl ListState {
    /// Sets the selection to be circular or not circular.
    #[must_use]
    pub fn circular(mut self, circular: bool) -> Self {
        self.circular = circular;
        self
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
        if index.is_none() {
            self.view_state.offset = 0;
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
        let i = match self.selected {
            Some(i) => {
                if i >= self.num_elements - 1 {
                    if self.circular {
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
        let i = match self.selected {
            Some(i) => {
                if i == 0 {
                    if self.circular {
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
    }

    /// Updates the number of elements that are present in the list.
    pub(crate) fn set_num_elements(&mut self, num_elements: usize) {
        self.num_elements = num_elements;
    }
}
