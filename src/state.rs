#[derive(Debug, Clone, Default)]
pub struct ListState {
    /// The selected item. If none, no item is selected.
    pub selected: Option<usize>,

    /// The index of the fist item on the screen
    pub(crate) offset: usize,

    /// The number of elements of the list. This is necessary to correctly
    /// wrap the selection of items.
    pub(crate) num_elements: usize,

    /// Whether the selection is circular. If circular, calling next on the
    /// last element returns the first element, and calling previous on
    /// the first element returns the last element.
    non_circular: bool,
}

impl ListState {
    /// Update the number of elements to be expected in the
    /// selection.
    pub fn set_num_elements(&mut self, num_elements: usize) {
        self.num_elements = num_elements;
    }

    /// If circular is True, the selection continues from the
    /// last item to the first when going down, and from the
    /// first item to the last when going up.
    /// It is true by default.
    #[must_use]
    pub fn circular(mut self, circular: bool) -> Self {
        self.non_circular = !circular;
        self
    }

    /// Return the currently selected items index
    #[must_use]
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Select an item by its index
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        if index.is_none() {
            self.offset = 0;
        }
    }

    /// Selects the next element of the list. If circular is true,
    /// calling next on the last element selects the first.
    pub fn next(&mut self) {
        if self.num_elements == 0 {
            return;
        }
        let i = match self.selected() {
            Some(i) => {
                if i >= self.num_elements - 1 {
                    if self.non_circular {
                        i
                    } else {
                        0
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
    pub fn previous(&mut self) {
        if self.num_elements == 0 {
            return;
        }
        let i = match self.selected() {
            Some(i) => {
                if i == 0 {
                    if self.non_circular {
                        i
                    } else {
                        self.num_elements - 1
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.select(Some(i));
    }

    /// Here we check and if necessary update the viewport. For this we start with the first item
    /// on the screen and iterate until we have reached the maximum height. If the selected value
    /// is within the bounds we do nothing. If the selected value is out of bounds, we adjust the
    /// offset accordingly.
    pub(crate) fn update_view_port(
        &mut self,
        heights: &[usize],
        max_height: usize,
        truncate: bool,
    ) -> Vec<usize> {
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
                if truncate {
                    // Truncate the last widget
                    view_heights.push(max_height - y);
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
                if truncate {
                    view_heights.insert(0, max_height - y);
                    self.offset = i;
                } else {
                    self.offset = i + 1;
                }
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
                    non_circular: false,
                };

                //when
                let heights = given_state.update_view_port(&$given_heights, $given_max_height, true);
                let offset = given_state.offset;

                // then
                assert_eq!(offset, $expected_offset);
                assert_eq!(heights, $expected_heights);
                assert_eq!(offset, $expected_offset);
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
    }
}
