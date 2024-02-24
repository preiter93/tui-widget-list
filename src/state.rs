#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct ListState {
    /// The selected item. If `None`, no item is currently selected.
    pub selected: Option<usize>,

    /// The index of the first item displayed on the screen.
    pub(crate) offset: usize,

    /// The total number of elements in the list. This is necessary to correctly
    /// handle item selection.
    pub(crate) num_elements: usize,

    /// Indicates if the selection forms a circular structure. If circular,
    /// calling `next` on the last element returns the first, and calling
    /// `previous` on the first element returns the last.
    ///
    /// True by default.
    circular: bool,
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            selected: None,
            offset: 0,
            num_elements: 0,
            circular: true,
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
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Selects an item by its index.
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        if index.is_none() {
            self.offset = 0;
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
        let i = match self.selected() {
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
        let i = match self.selected() {
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

    /// Here we check and if necessary update the viewport. For this we start with the first item
    /// on the screen and iterate until we have reached the maximum height. If the selected value
    /// is within the bounds we do nothing. If the selected value is out of bounds, we adjust the
    /// offset accordingly.
    pub(crate) fn update_view_port(&mut self, heights: &[usize], max_height: usize) -> Vec<usize> {
        // The items heights on the viewport will be calculated on the fly.
        let mut view_heights: Vec<usize> = Vec::new();

        // If none is selected, the first item should be show on top of the viewport.
        let selected = self.selected.unwrap_or(0);

        // If the selected value is smaller than the offset, we roll
        // the offset so that the selected value is at the top
        if selected < self.offset {
            self.offset = selected;
        }

        // Check if the selected item is in the current view
        let (mut y, mut i) = (0, self.offset);
        let mut found = false;
        for height in heights.iter().skip(self.offset) {
            // Out of bounds
            if y + height > max_height {
                // Truncate the last widget
                let dy = max_height - y;
                if dy > 0 {
                    view_heights.push(dy);
                }
                break;
            }
            // Selected value is within view/bounds, so we are good
            // but we keep iterating to collect the view heights
            if selected == i {
                found = true;
            }
            y += height;
            i += 1;
            view_heights.push(*height);
        }
        if found {
            return view_heights;
        }

        // The selected item is out of bounds. We iterate backwards from the selected
        // item and determine the first widget that still fits on the screen.
        view_heights.clear();
        let (mut y, mut i) = (0, selected);
        let last = heights.len().saturating_sub(1);
        for height in heights.iter().rev().skip(last.saturating_sub(selected)) {
            // out of bounds
            if y + height >= max_height {
                view_heights.insert(0, max_height - y);
                self.offset = i;
                break;
            }
            view_heights.insert(0, *height);
            y += height;
            i -= 1;
        }
        view_heights
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! update_view_port_tests {
        ($($name:ident:
        [
           $given_offset:expr,
           $given_selected:expr,
           $given_heights:expr,
           $given_max_height:expr
        ],
        [
           $expected_offset:expr,
           $expected_heights:expr
        ],)*) => {
        $(
            #[test]
            fn $name() {
                // given
                let mut given_state = ListState {
                    offset: $given_offset,
                    selected: $given_selected,
                    num_elements: $given_heights.len(),
                    circular: true,
                };

                //when
                let heights = given_state.update_view_port(&$given_heights, $given_max_height);
                let offset = given_state.offset;

                // then
                assert_eq!(offset, $expected_offset);
                assert_eq!(heights, $expected_heights);
            }
        )*
        }
    }

    update_view_port_tests! {
        happy_path: [0, Some(0), vec![2, 3], 6], [0, vec![2, 3]],
        empty_list: [0, None, Vec::<usize>::new(), 4], [0, vec![]],
        update_offset_down: [0, Some(2), vec![2, 3, 3], 6], [1, vec![3, 3]],
        update_offset_up: [1, Some(0), vec![2, 3, 3], 6], [0, vec![2, 3, 1]],
        truncate_bottom: [0, Some(0), vec![2, 3], 4], [0, vec![2, 2]],
        truncate_top: [0, Some(1), vec![2, 3], 4], [0, vec![1, 3]],
        num_elements: [0, None, vec![1, 1, 1, 1, 1], 3], [0, vec![1, 1, 1]],
    }
}
