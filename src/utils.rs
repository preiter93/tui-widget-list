use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;
use std::{cmp::Ordering, fs::OpenOptions};

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
    scroll_padding: u16,
) -> HashMap<usize, ViewportElement<T>> {
    // The items heights on the viewport will be calculated on the fly.
    let mut viewport: HashMap<usize, ViewportElement<T>> = HashMap::new();

    // If none is selected, the first item should be show on top of the viewport.
    let selected = state.selected.unwrap_or(0);

    // Calculate the effective scroll padding for each widget
    let effective_scroll_padding_by_index = calculate_effective_scroll_padding(
        state,
        builder,
        item_count,
        cross_axis_size,
        scroll_axis,
        scroll_padding,
    );

    // If the selected value is smaller than the offset, we roll
    // the offset so that the selected value is at the top. The complicated
    // part is that we also need to account for scroll padding.
    let scroll_padding_top = *effective_scroll_padding_by_index
        .get(&selected)
        .unwrap_or(&0);
    let mut first_element = selected;
    let mut first_element_truncated = 0;
    let mut available_size = scroll_padding_top;
    for index in (0..=selected).rev() {
        first_element = index;
        if available_size == 0 {
            break;
        }
        let context = ListBuildContext {
            index,
            is_selected: state.selected.map_or(false, |j| index == j),
            scroll_axis,
            cross_axis_size,
        };
        let (_, main_axis_size) = builder.call_closure(&context);
        available_size = available_size.saturating_sub(main_axis_size);
        if available_size > 0 {
            first_element_truncated = main_axis_size.saturating_sub(available_size);
        }
    }
    if first_element < state.view_state.offset
        || (first_element == state.view_state.offset && state.view_state.first_truncated > 0)
    {
        state.view_state.offset = first_element;
        state.view_state.first_truncated = first_element_truncated;
    }

    // Begin a forward pass, starting from `view_state.offset`.
    let found_selected = forward_pass(
        &mut viewport,
        state,
        builder,
        state.view_state.offset,
        item_count,
        total_main_axis_size,
        selected,
        cross_axis_size,
        scroll_axis,
        &effective_scroll_padding_by_index,
    );

    if found_selected {
        return viewport;
    }

    viewport.clear();

    // Perform a backward pass, starting from the `selected` item.
    // This step is only necessary if the forward pass did not
    // locate the selected item.
    backward_pass(
        &mut viewport,
        state,
        builder,
        item_count,
        total_main_axis_size,
        selected,
        cross_axis_size,
        scroll_axis,
        &effective_scroll_padding_by_index,
    );

    viewport
}

/// Iterate forward through the list of widgets.
///
/// Returns true if the selected widget is inside the viewport.
#[allow(clippy::too_many_arguments)]
fn forward_pass<T>(
    viewport: &mut HashMap<usize, ViewportElement<T>>,
    state: &mut ListState,
    builder: &ListBuilder<T>,
    offset: usize,
    item_count: usize,
    total_main_axis_size: u16,
    selected: usize,
    cross_axis_size: u16,
    scroll_axis: ScrollAxis,
    scroll_padding_by_index: &HashMap<usize, u16>,
) -> bool {
    // Check if the selected item is in the current view
    let mut found_last = false;
    let mut found_selected = false;
    let mut available_size = total_main_axis_size;
    for index in offset..item_count {
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

        // The effective available size considering scroll padding.
        let scroll_padding_effective = scroll_padding_by_index.get(&index).unwrap_or(&0);
        let available_effective = available_size.saturating_sub(*scroll_padding_effective);

        // Out of bounds
        if !found_selected && main_axis_size >= available_effective {
            break;
        }

        // Selected value is within view/bounds, so we are good
        // but we keep iterating to collect the full viewport.
        if selected == index {
            found_selected = true;
        }

        let truncation = match available_size.cmp(&main_axis_size) {
            // We found the last element and it fits onto the viewport
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

        viewport.insert(
            index,
            ViewportElement::new(widget, total_main_axis_size, truncation.clone()),
        );

        if found_last {
            break;
        }

        available_size -= main_axis_size;
    }

    found_selected
}

// The selected item is out of bounds. We iterate backwards from the selected
// item and determine the first widget that still fits on the screen.
#[allow(clippy::too_many_arguments)]
fn backward_pass<T>(
    viewport: &mut HashMap<usize, ViewportElement<T>>,
    state: &mut ListState,
    builder: &ListBuilder<T>,
    item_count: usize,
    total_main_axis_size: u16,
    selected: usize,
    cross_axis_size: u16,
    scroll_axis: ScrollAxis,
    scroll_padding_by_index: &HashMap<usize, u16>,
) {
    let mut found_first = false;
    let mut available_size = total_main_axis_size;
    let scroll_padding_effective = *scroll_padding_by_index.get(&selected).unwrap_or(&0);
    for index in (0..=selected).rev() {
        // Evaluate the widget
        let context = ListBuildContext {
            index,
            is_selected: state.selected.map_or(false, |j| index == j),
            scroll_axis,
            cross_axis_size,
        };
        let (widget, main_axis_size) = builder.call_closure(&context);

        let available_effective = available_size.saturating_sub(scroll_padding_effective);

        let truncation = match available_effective.cmp(&main_axis_size) {
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
                state.view_state.first_truncated =
                    main_axis_size.saturating_sub(available_effective);
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

    // Append elements to the list to fill the viewport after the selected item.
    // Only necessary for lists with scroll padding.
    if scroll_padding_effective > 0 {
        available_size = scroll_padding_effective;
        for index in selected + 1..item_count {
            let context = ListBuildContext {
                index,
                is_selected: state.selected.map_or(false, |j| index == j),
                scroll_axis,
                cross_axis_size,
            };
            let (widget, main_axis_size) = builder.call_closure(&context);

            let truncation = match available_size.cmp(&main_axis_size) {
                Ordering::Greater | Ordering::Equal => Truncation::None,
                Ordering::Less => Truncation::Bot(main_axis_size.saturating_sub(available_size)),
            };
            viewport.insert(
                index,
                ViewportElement::new(widget, main_axis_size, truncation),
            );

            available_size = available_size.saturating_sub(main_axis_size);
            // Out of bounds
            if available_size == 0 {
                break;
            }
        }
    }
}

/// Calculate the effective scroll padding.
/// Padding is applied until the scroll padding limit is reached,
/// after which elements at the beginning or end of the list do
/// not receive padding.
///
/// Returns:
/// A `HashMap` where the keys are the indices of the list items and the values are
/// the corresponding padding applied. If the item is not on the list, `scroll_padding`
/// is unaltered.
fn calculate_effective_scroll_padding<T>(
    state: &mut ListState,
    builder: &ListBuilder<T>,
    item_count: usize,
    cross_axis_size: u16,
    scroll_axis: ScrollAxis,
    scroll_padding: u16,
) -> HashMap<usize, u16> {
    let mut padding_by_element = HashMap::new();
    let mut total_main_axis_size = 0;

    for index in 0..item_count {
        if total_main_axis_size >= scroll_padding {
            padding_by_element.insert(index, scroll_padding);
            continue;
        }
        padding_by_element.insert(index, total_main_axis_size);

        let context = ListBuildContext {
            index,
            is_selected: state.selected.map_or(false, |j| index == j),
            scroll_axis,
            cross_axis_size,
        };

        let (_, item_main_axis_size) = builder.call_closure(&context);
        total_main_axis_size += item_main_axis_size;
    }

    total_main_axis_size = 0;
    for index in (0..item_count).rev() {
        // Stop applying padding once the scroll padding limit is reached
        if total_main_axis_size >= scroll_padding {
            break;
        }
        padding_by_element.insert(index, total_main_axis_size);

        let context = ListBuildContext {
            index,
            is_selected: state.selected.map_or(false, |j| index == j),
            scroll_axis,
            cross_axis_size,
        };

        let (_, item_main_axis_size) = builder.call_closure(&context);
        total_main_axis_size += item_main_axis_size;
    }

    padding_by_element
}

#[allow(dead_code)]
pub fn log_to_file<T: Debug>(data: T) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
        .unwrap();

    if let Err(e) = writeln!(file, "{data:?}") {
        eprintln!("Couldn't write to file: {e}");
    }
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
            0,
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
            0,
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
            0,
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
    // |   |
    // -----
    //
    // To:
    //
    // |   |
    // -----
    // |   | 1 <-
    // |   |
    // -----
    // |   |
    #[test]
    fn scroll_padding_bottom() {
        // given
        let mut state = ListState {
            num_elements: 3,
            selected: Some(1),
            ..ListState::default()
        };
        let given_sizes = vec![2, 2, 2];
        let given_item_count = given_sizes.len();
        let given_total_size = 4;

        let expected_view_state = ViewState {
            offset: 0,
            first_truncated: 1,
        };
        let expected_viewport = HashMap::from([
            (0, ViewportElement::new(TestItem {}, 2, Truncation::Top(1))),
            (1, ViewportElement::new(TestItem {}, 2, Truncation::None)),
            (2, ViewportElement::new(TestItem {}, 2, Truncation::Bot(1))),
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
            1,
        );

        // then
        assert_eq!(viewport, expected_viewport);
        assert_eq!(state.view_state, expected_view_state);
    }

    // From:
    //
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
    // |   |
    // -----
    // |   | 1 <-
    // |   |
    // -----
    // |   |
    #[test]
    fn scroll_padding_top() {
        // given
        let view_state = ViewState {
            offset: 2,
            first_truncated: 0,
        };
        let mut state = ListState {
            num_elements: 3,
            selected: Some(1),
            view_state,
            ..ListState::default()
        };
        let given_sizes = vec![2, 2, 2];
        let given_item_count = given_sizes.len();
        let given_total_size = 4;

        let expected_view_state = ViewState {
            offset: 0,
            first_truncated: 1,
        };
        let expected_viewport = HashMap::from([
            (0, ViewportElement::new(TestItem {}, 2, Truncation::Top(1))),
            (1, ViewportElement::new(TestItem {}, 2, Truncation::None)),
            (2, ViewportElement::new(TestItem {}, 2, Truncation::Bot(1))),
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
            1,
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
            0,
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
            0,
        );

        // then
        assert_eq!(viewport, expected_viewport);
        assert_eq!(state.view_state, expected_view_state);
    }

    #[test]
    fn test_calculate_effective_scroll_padding() {
        let mut state = ListState::default();
        let given_sizes = vec![2, 2, 2, 2, 2];
        let item_count = 5;
        let scroll_padding = 3;

        let builder = ListBuilder::new(move |context| {
            return (TestItem {}, given_sizes[context.index]);
        });

        let scroll_padding = calculate_effective_scroll_padding(
            &mut state,
            &builder,
            item_count,
            1,
            ScrollAxis::Vertical,
            scroll_padding,
        );

        assert_eq!(*scroll_padding.get(&0).unwrap(), 0);
        assert_eq!(*scroll_padding.get(&1).unwrap(), 2);
        assert_eq!(*scroll_padding.get(&2).unwrap(), 3);
        assert_eq!(*scroll_padding.get(&3).unwrap(), 2);
        assert_eq!(*scroll_padding.get(&4).unwrap(), 0);
    }
}
