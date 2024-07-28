use std::collections::HashMap;

use crate::{ListState, PreRender, PreRenderContext, ScrollAxis};

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
pub(crate) fn layout_on_viewport<T: PreRender>(
    state: &mut ListState,
    widgets: &mut [T],
    total_main_axis_size: u16,
    cross_axis_size: u16,
    scroll_axis: ScrollAxis,
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

    let mut main_axis_size_cache: HashMap<usize, u16> = HashMap::new();

    // Check if the selected item is in the current view
    let (mut y, mut index) = (0, state.offset);
    let mut found = false;
    for widget in widgets.iter_mut().skip(state.offset) {
        // Get the main axis size of the widget.
        let is_selected = state.selected.map_or(false, |j| index == j);
        let context = PreRenderContext::new(is_selected, cross_axis_size, scroll_axis, index);

        let main_axis_size = widget.pre_render(&context);
        main_axis_size_cache.insert(index, main_axis_size);

        // Out of bounds
        if y + main_axis_size > total_main_axis_size {
            // Truncate the last widget
            let dy = total_main_axis_size - y;
            if dy > 0 {
                viewport_layouts.push(ViewportLayout {
                    main_axis_size: dy,
                    truncated_by: main_axis_size.saturating_sub(dy),
                });
            }
            break;
        }
        // Selected value is within view/bounds, so we are good
        // but we keep iterating to collect the view heights
        if selected == index {
            found = true;
        }
        y += main_axis_size;
        index += 1;

        viewport_layouts.push(ViewportLayout {
            main_axis_size,
            truncated_by: 0,
        });
    }
    if found {
        return viewport_layouts;
    }

    // The selected item is out of bounds. We iterate backwards from the selected
    // item and determine the first widget that still fits on the screen.
    viewport_layouts.clear();
    let (mut y, mut index) = (0, selected);
    let last = widgets.len().saturating_sub(1);
    for widget in widgets.iter_mut().rev().skip(last.saturating_sub(selected)) {
        // Get the main axis size of the widget. At this point we might have already
        // calculated it, so check the cache first.
        let main_axis_size = if let Some(main_axis_size) = main_axis_size_cache.remove(&index) {
            main_axis_size
        } else {
            let is_selected = state.selected.map_or(false, |j| index == j);
            let context = PreRenderContext::new(is_selected, cross_axis_size, scroll_axis, index);

            widget.pre_render(&context)
        };

        // Truncate the first widget
        if y + main_axis_size >= total_main_axis_size {
            let dy = total_main_axis_size - y;
            viewport_layouts.insert(
                0,
                ViewportLayout {
                    main_axis_size: dy,
                    truncated_by: main_axis_size.saturating_sub(dy),
                },
            );
            state.offset = index;
            break;
        }

        viewport_layouts.insert(
            0,
            ViewportLayout {
                main_axis_size,
                truncated_by: 0,
            },
        );

        y += main_axis_size;
        index -= 1;
    }

    viewport_layouts
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct ViewportLayout {
    pub(crate) main_axis_size: u16,
    pub(crate) truncated_by: u16,
}

#[cfg(test)]
mod tests {
    use ratatui::{
        prelude::*,
        widgets::{Block, Borders},
    };

    use super::*;

    struct TestItem {
        main_axis_size: u16,
    }

    impl Widget for TestItem {
        fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized,
        {
            Block::default().borders(Borders::ALL).render(area, buf);
        }
    }

    impl PreRender for TestItem {
        fn pre_render(&mut self, _context: &PreRenderContext) -> u16 {
            self.main_axis_size
        }
    }

    macro_rules! update_view_port_tests {
        ($($name:ident:
        [
           $given_offset:expr,
           $given_selected:expr,
           $given_sizes:expr,
           $given_max_size:expr
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
                    num_elements: $given_sizes.len(),
                    circular: true,
                };

                //when
                let mut widgets: Vec<TestItem> = $given_sizes
                    .into_iter()
                    .map(|main_axis_size| TestItem {
                        main_axis_size: main_axis_size as u16,
                    })
                    .collect();
                let scroll_axis = ScrollAxis::default();
                let layouts = layout_on_viewport(&mut given_state, &mut widgets, $given_max_size, 0, scroll_axis);
                let offset = given_state.offset;

                // then
                let main_axis_sizes: Vec<u16> = layouts.iter().map(|x| x.main_axis_size).collect();
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
