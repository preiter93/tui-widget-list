use std::collections::HashMap;

use crate::{ListBuildContext, ListBuilder, ListState, ScrollAxis};

pub(crate) fn layout_on_viewport<T>(
    state: &mut ListState,
    builder: &ListBuilder<T>,
    item_count: usize,
    main_axis_size: u16,
    cross_axis_size: u16,
    scroll_axis: ScrollAxis,
) -> HashMap<usize, ViewportElement<T>> {
    // The items heights on the viewport will be calculated on the fly.
    let mut viewport: HashMap<usize, ViewportElement<T>> = HashMap::new();

    // If none is selected, the first item should be show on top of the viewport.
    let selected = state.selected.unwrap_or(0);

    // If the selected value is smaller than the offset, we roll
    // the offset so that the selected value is at the top
    if selected < state.offset {
        state.offset = selected;
    }

    // Check if the selected item is in the current view
    let (mut y, mut found_last) = (0, false);
    let mut found = false;
    for index in state.offset..item_count {
        let mut truncate_by = 0;
        let available_size: u16 = main_axis_size.saturating_sub(y);

        // Build the widget
        let context = ListBuildContext {
            index,
            is_selected: state.selected.map_or(false, |j| index == j),
            scroll_axis,
            cross_axis_size,
        };
        let (widget, main_axis_size) = builder.call_closure(&context);

        // Out of bounds
        if !found && main_axis_size >= available_size {
            break;
        }

        // Selected value is within view/bounds, so we are good
        // but we keep iterating to collect the full viewport.
        if selected == index && main_axis_size <= available_size {
            found = true;
        }

        // Found the last element. We can stop iterating.
        if found && main_axis_size >= available_size {
            found_last = true;
            truncate_by = main_axis_size.saturating_sub(available_size);
        }
        let element = ViewportElement::new(widget, main_axis_size, truncate_by);
        viewport.insert(index, element);

        if found_last {
            break;
        }

        // We keep iterating until we collected all viewport elements
        y += main_axis_size;
    }

    if found {
        return viewport;
    }

    viewport.clear();

    // The selected item is out of bounds. We iterate backwards from the selected
    // item and determine the first widget that still fits on the screen.
    let (mut y, mut found_first) = (0, false);
    for index in (0..=selected).rev() {
        let available_size = main_axis_size - y;
        let mut truncate_by = 0;

        // Evaluate the widget
        let context = ListBuildContext {
            index,
            is_selected: state.selected.map_or(false, |j| index == j),
            scroll_axis,
            cross_axis_size,
        };
        let (widget, main_axis_size) = builder.call_closure(&context);

        // We found the first element
        if available_size <= main_axis_size {
            found_first = true;
            // main_axis_size = available_size;
            truncate_by = main_axis_size.saturating_sub(available_size);
            state.offset = index;
        }

        let element = ViewportElement::new(widget, main_axis_size, truncate_by);
        viewport.insert(index, element);

        if found_first {
            break;
        }

        y += main_axis_size;
    }

    viewport
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct ViewportElement<T> {
    pub(crate) widget: T,
    pub(crate) main_axis_size: u16,
    pub(crate) truncate_by: u16,
}

impl<T> ViewportElement<T> {
    #[must_use]
    pub(crate) fn new(widget: T, main_axis_size: u16, truncated_by: u16) -> Self {
        Self {
            widget,
            main_axis_size,
            truncate_by: truncated_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        prelude::*,
        widgets::{Block, Borders},
    };

    use super::*;

    #[derive(Debug, Default, PartialEq, Eq)]
    struct TestItem {}

    impl Widget for TestItem {
        fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized,
        {
            Block::default().borders(Borders::ALL).render(area, buf);
        }
    }

    #[test]
    fn happy_path() {
        // given
        let mut given_state = ListState {
            num_elements: 2,
            ..ListState::default()
        };
        let given_item_count = 2;
        let given_sizes = vec![2, 3];
        let given_total_size = 6;
        let builder = &ListBuilder::new(move |context| {
            return (TestItem {}, given_sizes[context.index]);
        });

        // when
        let viewport = layout_on_viewport(
            &mut given_state,
            builder,
            given_item_count,
            given_total_size,
            1,
            ScrollAxis::Vertical,
        );
        let offset = given_state.offset;

        // then
        let expected_offset = 0;

        let mut expected_viewport = HashMap::new();
        expected_viewport.insert(0, ViewportElement::new(TestItem {}, 2, 0));
        expected_viewport.insert(1, ViewportElement::new(TestItem {}, 3, 0));

        assert_eq!(offset, expected_offset);
        assert_eq!(viewport, expected_viewport);
    }

    #[test]
    fn scroll_down_out_of_bounds() {
        // given
        let mut given_state = ListState {
            num_elements: 3,
            selected: Some(2),
            ..ListState::default()
        };
        let given_sizes = vec![2, 3, 3];
        let given_item_count = given_sizes.len();
        let given_total_size = 6;
        let builder = &ListBuilder::new(move |context| {
            return (TestItem {}, given_sizes[context.index]);
        });

        // when
        let viewport = layout_on_viewport(
            &mut given_state,
            builder,
            given_item_count,
            given_total_size,
            1,
            ScrollAxis::Vertical,
        );
        let offset = given_state.offset;

        // then
        let expected_offset = 1;

        let mut expected_viewport = HashMap::new();
        expected_viewport.insert(1, ViewportElement::new(TestItem {}, 3, 0));
        expected_viewport.insert(2, ViewportElement::new(TestItem {}, 3, 0));

        assert_eq!(offset, expected_offset);
        assert_eq!(viewport, expected_viewport);
    }

    #[test]
    fn scroll_up() {
        // given
        let mut given_state = ListState {
            num_elements: 4,
            selected: Some(2),
            offset: 1,
            ..ListState::default()
        };
        let given_sizes = vec![2, 2, 2, 3];
        let given_total_size = 6;
        let given_item_count = given_sizes.len();
        let builder = &ListBuilder::new(move |context| {
            return (TestItem {}, given_sizes[context.index]);
        });

        // when
        let viewport = layout_on_viewport(
            &mut given_state,
            builder,
            given_item_count,
            given_total_size,
            1,
            ScrollAxis::Vertical,
        );
        let offset = given_state.offset;

        // then
        let expected_offset = 1;

        let mut expected_viewport = HashMap::new();
        expected_viewport.insert(1, ViewportElement::new(TestItem {}, 2, 0));
        expected_viewport.insert(2, ViewportElement::new(TestItem {}, 2, 0));
        expected_viewport.insert(3, ViewportElement::new(TestItem {}, 3, 1));

        assert_eq!(offset, expected_offset);
        assert_eq!(viewport, expected_viewport);
    }

    #[test]
    fn scroll_up_out_of_bounds() {
        // given
        let mut given_state = ListState {
            num_elements: 3,
            selected: Some(0),
            offset: 1,
            ..ListState::default()
        };
        let given_sizes = vec![2, 3, 3];
        let given_total_size = 6;
        let given_item_count = given_sizes.len();
        let builder = &ListBuilder::new(move |context| {
            return (TestItem {}, given_sizes[context.index]);
        });

        // when
        let viewport = layout_on_viewport(
            &mut given_state,
            builder,
            given_item_count,
            given_total_size,
            1,
            ScrollAxis::Vertical,
        );
        let offset = given_state.offset;

        // then
        let expected_offset = 0;

        let mut expected_viewport = HashMap::new();
        expected_viewport.insert(0, ViewportElement::new(TestItem {}, 2, 0));
        expected_viewport.insert(1, ViewportElement::new(TestItem {}, 3, 0));
        expected_viewport.insert(2, ViewportElement::new(TestItem {}, 3, 2));

        assert_eq!(offset, expected_offset);
        assert_eq!(viewport, expected_viewport);
    }

    #[test]
    fn truncate_top() {
        // given
        let mut given_state = ListState {
            num_elements: 2,
            selected: Some(1),
            ..ListState::default()
        };
        let given_sizes = vec![2, 3];
        let given_total_size = 4;
        let given_item_count = given_sizes.len();
        let builder = &ListBuilder::new(move |context| {
            return (TestItem {}, given_sizes[context.index]);
        });

        // when
        let viewport = layout_on_viewport(
            &mut given_state,
            builder,
            given_item_count,
            given_total_size,
            1,
            ScrollAxis::Vertical,
        );
        let offset = given_state.offset;

        // then
        let expected_offset = 0;

        let mut expected_viewport = HashMap::new();
        expected_viewport.insert(0, ViewportElement::new(TestItem {}, 2, 1));
        expected_viewport.insert(1, ViewportElement::new(TestItem {}, 3, 0));

        assert_eq!(offset, expected_offset);
        assert_eq!(viewport, expected_viewport);
    }

    #[test]
    fn truncate_bot() {
        // given
        let mut given_state = ListState {
            num_elements: 2,
            selected: Some(0),
            ..ListState::default()
        };
        let given_sizes = vec![2, 3];
        let given_total_size = 4;
        let given_item_count = given_sizes.len();
        let builder = &ListBuilder::new(move |context| {
            return (TestItem {}, given_sizes[context.index]);
        });

        // when
        let viewport = layout_on_viewport(
            &mut given_state,
            builder,
            given_item_count,
            given_total_size,
            1,
            ScrollAxis::Vertical,
        );
        let offset = given_state.offset;

        // then
        let expected_offset = 0;

        let mut expected_viewport = HashMap::new();
        expected_viewport.insert(0, ViewportElement::new(TestItem {}, 2, 0));
        expected_viewport.insert(1, ViewportElement::new(TestItem {}, 3, 1));

        assert_eq!(offset, expected_offset);
        assert_eq!(viewport, expected_viewport);
    }
}
