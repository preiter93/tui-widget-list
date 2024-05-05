use crate::ListState;

/// This method checks how to layout the items on the viewport and if necessary
/// updates the offset of the first item on the screen.
///
/// For this we start with the first item on the screen and iterate until we have
/// reached the maximum height. If the selected value is within the bounds we do
/// nothing. If the selected value is out of bounds, the offset is adjusted.
///
/// # Returns
/// - The sizes along the main axis of the elements on the viewport,
///   and how much they are being truncated to fit on the viewport.
pub(crate) fn update_view_port(
    state: &mut ListState,
    main_axis_sizes: &[u16],
    max_height: u16,
) -> Vec<ViewportLayout> {
    // The items heights on the viewport will be calculated on the fly.
    let mut viewport_layouts: Vec<ViewportLayout> = Vec::new();

    // If none is selected, the first item should be show on top of the viewport.
    let selected = state.selected.unwrap_or(0);

    // If the selected value is smaller than the offset, we roll
    // the offset so that the selected value is at the top
    if selected < state.offset {
        state.offset = selected;
    }

    // Check if the selected item is in the current view
    let (mut y, mut i) = (0, state.offset);
    let mut found = false;
    for main_axis_size in main_axis_sizes.iter().skip(state.offset) {
        // Out of bounds
        if y + main_axis_size > max_height {
            // Truncate the last widget
            let dy = max_height - y;
            if dy > 0 {
                let vl = ViewportLayout {
                    size: dy,
                    truncated_by: main_axis_size.saturating_sub(dy),
                };
                viewport_layouts.push(vl);
            }
            break;
        }
        // Selected value is within view/bounds, so we are good
        // but we keep iterating to collect the view heights
        if selected == i {
            found = true;
        }
        y += main_axis_size;
        i += 1;

        let vl = ViewportLayout {
            size: *main_axis_size,
            truncated_by: 0,
        };
        viewport_layouts.push(vl);
    }
    if found {
        return viewport_layouts;
    }

    // The selected item is out of bounds. We iterate backwards from the selected
    // item and determine the first widget that still fits on the screen.
    viewport_layouts.clear();
    let (mut y, mut i) = (0, selected);
    let last = main_axis_sizes.len().saturating_sub(1);
    for main_axis_size in main_axis_sizes
        .iter()
        .rev()
        .skip(last.saturating_sub(selected))
    {
        // Truncate the first widget
        if y + main_axis_size >= max_height {
            let dy = max_height - y;
            let vl = ViewportLayout {
                size: dy,
                truncated_by: main_axis_size.saturating_sub(dy),
            };
            viewport_layouts.insert(0, vl);
            state.offset = i;
            break;
        }

        let vl = ViewportLayout {
            size: *main_axis_size,
            truncated_by: 0,
        };
        viewport_layouts.insert(0, vl);

        y += main_axis_size;
        i -= 1;
    }
    viewport_layouts
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct ViewportLayout {
    pub(crate) size: u16,
    pub(crate) truncated_by: u16,
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
           $expected_sizes:expr
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
                let layouts = update_view_port(&mut given_state, &$given_heights, $given_max_height);
                let offset = given_state.offset;

                // then
                let main_axis_sizes: Vec<u16> = layouts.iter().map(|x| x.size).collect();
                assert_eq!(offset, $expected_offset);
                assert_eq!(main_axis_sizes, $expected_sizes);
            }
        )*
        }
    }

    update_view_port_tests! {
        happy_path: [0, Some(0), vec![2, 3], 6], [0, vec![2, 3]],
        empty_list: [0, None, Vec::<u16>::new(), 4], [0, vec![]],
        update_offset_down: [0, Some(2), vec![2, 3, 3], 6], [1, vec![3, 3]],
        update_offset_up: [1, Some(0), vec![2, 3, 3], 6], [0, vec![2, 3, 1]],
        truncate_bottom: [0, Some(0), vec![2, 3], 4], [0, vec![2, 2]],
        truncate_top: [0, Some(1), vec![2, 3], 4], [0, vec![1, 3]],
        num_elements: [0, None, vec![1, 1, 1, 1, 1], 3], [0, vec![1, 1, 1]],
    }
}
