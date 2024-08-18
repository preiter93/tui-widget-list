use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::{view::Truncation, ListBuildContext, ListBuilder, ListState, ScrollAxis};

/// Determines the new viewport layout based on the previous viewport state, i.e.
/// the offset of the first element and the truncation of the first element.
///
/// Iterates over the widgets in the list, evaluates their heights lazily, updates
/// the new view state (offset and truncation of the first element) and returns the
/// widgets that should be rendered in the current viewport.
///
/// # There
/// There are the following cases to consider:
///
/// - Selected item is on the viewport
/// - Selected item is above the previous viewport, either truncated or out of bounds
///      - If it is truncated, the viewport will be adjusted to bring the entire item into view.
///      - If it is out of bounds, the viewport will be scrolled upwards to make the selected item visible.
/// - Selected item is below the previous viewport, either truncated or out of bounds
///      - If it is truncated, the viewport will be adjusted to bring the entire item into view.
///      - If it is out of bounds, the viewport will be scrolled downwards to make the selected item visible.
#[allow(clippy::too_many_lines)]
pub(crate) fn layout_on_viewport<T>(
    state: &mut ListState,
    builder: &ListBuilder<T>,
    item_count: usize,
    total_main_axis_size: u16,
    cross_axis_size: u16,
    scroll_axis: ScrollAxis,
) -> HashMap<usize, ViewportElement<T>> {
    // The items heights on the viewport will be calculated on the fly.
    let mut viewport: HashMap<usize, ViewportElement<T>> = HashMap::new();

    // If none is selected, the first item should be show on top of the viewport.
    let selected = state.selected.unwrap_or(0);

    // If the selected value is smaller than the offset, we roll
    // the offset so that the selected value is at the top
    if selected < state.view_state.offset
        || (selected == state.view_state.offset && state.view_state.first_truncated > 0)
    {
        state.view_state.offset = selected;
        state.view_state.first_truncated = 0;
    }

    // Check if the selected item is in the current view
    let mut found_last = false;
    let mut found_selected = false;
    let mut available_size = total_main_axis_size;
    for index in state.view_state.offset..item_count {
        let is_first = index == state.view_state.offset;
        // Build the widget
        let context = ListBuildContext {
            index,
            is_selected: state.selected.map_or(false, |j| index == j),
            scroll_axis,
            cross_axis_size,
        };
        let (widget, total_main_axis_size) = builder.call_closure(&context);
        let main_axis_size = if is_first {
            total_main_axis_size.saturating_sub(state.view_state.first_truncated)
        } else {
            total_main_axis_size
        };

        // Out of bounds
        if !found_selected && main_axis_size >= available_size {
            break;
        }

        // Selected value is within view/bounds, so we are good
        // but we keep iterating to collect the full viewport.
        if selected == index {
            found_selected = true;
        }

        let truncation = match available_size.cmp(&main_axis_size) {
            // We found the last element and it fits into the viewport
            Ordering::Equal => {
                found_last = true;
                if is_first {
                    Truncation::Bot(state.view_state.first_truncated)
                } else {
                    Truncation::None
                }
            }
            // We found the last element but it needs to be truncated
            Ordering::Less => {
                found_last = true;
                let value = main_axis_size.saturating_sub(available_size);
                if is_first {
                    state.view_state.first_truncated = value;
                }
                Truncation::Bot(value)
            }
            Ordering::Greater => {
                // The first element was truncated in the last layout run,
                // we keep it truncated to handle scroll ups gracefully.
                if is_first && state.view_state.first_truncated != 0 {
                    Truncation::Top(state.view_state.first_truncated)
                } else {
                    Truncation::None
                }
            }
        };
        // if found_selected && is_first {
        //     state.truncated = truncation.value();
        // }

        viewport.insert(
            index,
            ViewportElement::new(widget, total_main_axis_size, truncation.clone()),
        );

        if found_last {
            break;
        }

        available_size -= main_axis_size;
    }

    if found_selected {
        return viewport;
    }

    viewport.clear();

    // The selected item is out of bounds. We iterate backwards from the selected
    // item and determine the first widget that still fits on the screen.
    let mut found_first = false;
    let mut available_size = total_main_axis_size;
    for index in (0..=selected).rev() {
        // Evaluate the widget
        let context = ListBuildContext {
            index,
            is_selected: state.selected.map_or(false, |j| index == j),
            scroll_axis,
            cross_axis_size,
        };
        let (widget, main_axis_size) = builder.call_closure(&context);

        let truncation = match available_size.cmp(&main_axis_size) {
            // We found the first element and it fits into the viewport
            Ordering::Equal => {
                found_first = true;
                state.view_state.offset = index;
                state.view_state.first_truncated = 0;
                Truncation::None
            }
            // We found the first element but it needs to be truncated
            Ordering::Less => {
                found_first = true;
                state.view_state.offset = index;
                state.view_state.first_truncated = main_axis_size.saturating_sub(available_size);
                // Truncate from the bottom if there is only one element on the viewport
                if index == selected {
                    Truncation::Bot(state.view_state.first_truncated)
                } else {
                    Truncation::Top(state.view_state.first_truncated)
                }
            }
            Ordering::Greater => Truncation::None,
        };

        let element = ViewportElement::new(widget, main_axis_size, truncation);
        viewport.insert(index, element);

        if found_first {
            break;
        }

        available_size -= main_axis_size;
    }

    viewport
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct ViewportElement<T> {
    pub(crate) widget: T,
    pub(crate) main_axis_size: u16,
    pub(crate) truncation: Truncation,
}

impl<T> ViewportElement<T> {
    #[must_use]
    pub(crate) fn new(widget: T, main_axis_size: u16, truncation: Truncation) -> Self {
        Self {
            widget,
            main_axis_size,
            truncation,
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        prelude::*,
        widgets::{Block, Borders},
    };

    use crate::state::ViewState;

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

    // From:
    //
    // -----
    // |   | 0
    // |   |
    // -----
    // |   | 1
    // |   |
    // -----
    //
    // To:
    //
    // -----
    // |   | 0 <-
    // |   |
    // -----
    // |   | 1
    // |   |
    // -----
    #[test]
    fn happy_path() {
        // given
        let mut state = ListState {
            num_elements: 2,
            ..ListState::default()
        };
        let given_item_count = 2;
        let given_sizes = vec![2, 2];
        let given_total_size = 6;

        let expected_view_state = ViewState {
            offset: 0,
            first_truncated: 0,
        };
        let expected_viewport = HashMap::from([
            (0, ViewportElement::new(TestItem {}, 2, Truncation::None)),
            (1, ViewportElement::new(TestItem {}, 2, Truncation::None)),
        ]);

        // when
        let viewport = layout_on_viewport(
            &mut state,
            &ListBuilder::new(move |context| {
                return (TestItem {}, given_sizes[context.index]);
            }),
            given_item_count,
            given_total_size,
            1,
            ScrollAxis::Vertical,
        );

        // then
        assert_eq!(viewport, expected_viewport);
        assert_eq!(state.view_state, expected_view_state);
    }

    // From:
    //
    // |   | 0
    // -----
    // |   | 1 <-
    // |   |
    // -----
    //
    // To:
    //
    // -----
    // |   | 0 <-
    // |   |
    // -----
    // |   | 1
    #[test]
    fn scroll_up() {
        // given
        let view_state = ViewState {
            offset: 0,
            first_truncated: 1,
        };
        let mut state = ListState {
            num_elements: 3,
            selected: Some(0),
            view_state,
            ..ListState::default()
        };
        let given_sizes = vec![2, 2];
        let given_total_size = 3;
        let given_item_count = given_sizes.len();

        let expected_view_state = ViewState {
            offset: 0,
            first_truncated: 0,
        };
        let expected_viewport = HashMap::from([
            (0, ViewportElement::new(TestItem {}, 2, Truncation::None)),
            (1, ViewportElement::new(TestItem {}, 2, Truncation::Bot(1))),
        ]);

        // when
        let viewport = layout_on_viewport(
            &mut state,
            &ListBuilder::new(move |context| {
                return (TestItem {}, given_sizes[context.index]);
            }),
            given_item_count,
            given_total_size,
            1,
            ScrollAxis::Vertical,
        );

        // then
        assert_eq!(viewport, expected_viewport);
        assert_eq!(state.view_state, expected_view_state);
    }

    // From:
    //
    // -----
    // |   | 0 <-
    // |   |
    // -----
    // |   | 1
    //
    // To:
    //
    // |   | 0
    // -----
    // |   | 1 <-
    // |   |
    // -----
    #[test]
    fn scroll_down() {
        // given
        let mut state = ListState {
            num_elements: 2,
            selected: Some(1),
            ..ListState::default()
        };
        let given_sizes = vec![2, 2];
        let given_item_count = given_sizes.len();
        let given_total_size = 3;

        let expected_view_state = ViewState {
            offset: 0,
            first_truncated: 1,
        };
        let expected_viewport = HashMap::from([
            (0, ViewportElement::new(TestItem {}, 2, Truncation::Top(1))),
            (1, ViewportElement::new(TestItem {}, 2, Truncation::None)),
        ]);

        // when
        let viewport = layout_on_viewport(
            &mut state,
            &ListBuilder::new(move |context| {
                return (TestItem {}, given_sizes[context.index]);
            }),
            given_item_count,
            given_total_size,
            1,
            ScrollAxis::Vertical,
        );

        // then
        assert_eq!(viewport, expected_viewport);
        assert_eq!(state.view_state, expected_view_state);
    }

    // From:
    //
    // -----
    // |   | 1 <-
    // |   |
    // -----
    // |   | 2
    //
    // To:
    //
    // -----
    // |   | 0 <-
    // |   |
    // -----
    // |   | 1
    #[test]
    fn scroll_up_out_of_viewport() {
        // given
        let view_state = ViewState {
            offset: 1,
            first_truncated: 0,
        };
        let mut state = ListState {
            num_elements: 3,
            selected: Some(0),
            view_state,
            ..ListState::default()
        };
        let given_sizes = vec![2, 2, 2];
        let given_total_size = 3;
        let given_item_count = given_sizes.len();

        let expected_view_state = ViewState {
            offset: 0,
            first_truncated: 0,
        };
        let expected_viewport = HashMap::from([
            (0, ViewportElement::new(TestItem {}, 2, Truncation::None)),
            (1, ViewportElement::new(TestItem {}, 2, Truncation::Bot(1))),
        ]);

        // when
        let viewport = layout_on_viewport(
            &mut state,
            &ListBuilder::new(move |context| {
                return (TestItem {}, given_sizes[context.index]);
            }),
            given_item_count,
            given_total_size,
            1,
            ScrollAxis::Vertical,
        );

        // then
        assert_eq!(viewport, expected_viewport);
        assert_eq!(state.view_state, expected_view_state);
    }

    // From:
    //
    // |   | 0
    // -----
    // |   | 1
    // |   |
    // -----
    // |   | 2 <-
    // |   |
    // -----
    //
    // To:
    //
    // |   | 0
    // -----
    // |   | 1 <-
    // |   |
    // -----
    // |   | 2
    // |   |
    // -----
    #[test]
    fn scroll_up_keep_first_element_truncated() {
        // given
        let view_state = ViewState {
            offset: 0,
            first_truncated: 1,
        };
        let mut state = ListState {
            num_elements: 3,
            selected: Some(1),
            view_state,
            ..ListState::default()
        };
        let given_sizes = vec![2, 2, 2];
        let given_total_size = 5;
        let given_item_count = given_sizes.len();

        let expected_view_state = ViewState {
            offset: 0,
            first_truncated: 1,
        };
        let expected_viewport = HashMap::from([
            (0, ViewportElement::new(TestItem {}, 2, Truncation::Top(1))),
            (1, ViewportElement::new(TestItem {}, 2, Truncation::None)),
            (2, ViewportElement::new(TestItem {}, 2, Truncation::None)),
        ]);

        // when
        let viewport = layout_on_viewport(
            &mut state,
            &ListBuilder::new(move |context| {
                return (TestItem {}, given_sizes[context.index]);
            }),
            given_item_count,
            given_total_size,
            1,
            ScrollAxis::Vertical,
        );

        // then
        assert_eq!(viewport, expected_viewport);
        assert_eq!(state.view_state, expected_view_state);
    }
}
