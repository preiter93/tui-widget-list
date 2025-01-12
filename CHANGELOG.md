Released
--------

0.13.3 - 01 Jan 2025
===================
- Add support for scrollbars

0.13.2 - 28 Dec 2024
===================
- Return [ListState::scroll_offset_index]
 
0.13.1 - 22 Dec 2024
===================
- More flexible listbuilder lifetimes by @nmandrus1

0.13.0 - 22 Okt 2024
===================
- Bump ratatui to v0.29.0

0.12.2 - 15 Sep 2024
===================
- ListView::scroll_padding added. 
Allows to keep a specified number of cells above/below the selected widget visibile while scrolling.

- ListState::circular got removed. Use ListView::infinite_scrolling instead.

0.12.1 - 25 Aug 2024
===================
- Change scroll up behaviour - keep first element truncated if possible

0.12.0 - 17 Aug 2024
===================
- Bump ratatui to v0.28

0.11.0 - 10 Aug 2024
===================
- Introduces `ListView`, `ListBuilder` and `ListBuildContext` as replacement for `List`, `PreRender` and `PreRenderContext`.
- `List`, `PreRender` and `PreRenderContext` are marked as deprecated.
```rust
// Builder provides the ListBuildContext.
// Return the widget and the widget's size along its main-axis.
let builder = ListBuilder::new(|context| {
    let mut item = Line::from(format!("Item {0}", context.index));

    if context.index % 2 == 0 {
    item.style = Style::default().bg(Color::Rgb(28, 28, 32))
    } else {
    item.style = Style::default().bg(Color::Rgb(0, 0, 0))
    };

    if context.is_selected {
    item.style = Style::default()
    .bg(Color::Rgb(255, 153, 0))
    .fg(Color::Rgb(28, 28, 32));
    };

    return (item, 1);
    });

// Construct the list. ListView takes in the builder and an item count.
let list = ListView::new(builder, 20);

// Render the list
list.render(area, &mut buf, &mut state);
``` 

0.10.1 - 28 July 2024
===================
- Implement Styled for List (contributor: airblast-dev)
 
0.10.0 - 27 June 2024
===================
- Bump ratatui to v0.27

0.9.0 - 11 May 2024
===================
- Introduced `PreRender` trait as a replacement for `ListableWidget`.
- This change is non-breaking 
- It provides a more concise and clearer interface.
Migration Guide
- Update trait implementations to use the new `pre_render` signature:

```rust
fn pre_render(&mut self, context: &PreRenderContext) -> u16 {
  let main_axis_size = // The widgets size in the main axis

    if context.is_selected {
      self = // You can modify the selected item here
    }

  main_axis_size
}
```
- Deprecated ListState::selected(). Use the struct field selected instead.
- Updated examples
- Add example for long lists

0.8.3 - 1 May 2024 
===================
- Fix: Missing base style on truncated items

0.8.2 - 29 February 2024
===================
- Truncate last element correctly
- Add tests

0.8.1 - 24 February 2024
===================
- Add support for horizontal scroll
- Remove `truncate` option

**Breaking Change**
The ListableWidgets trait method main_axis_size() was renamed to
size with an additional scroll_direction parameter. This parameter
can be ignored if the ListItem is only used in vertical or horizontal
scroll mode, and not in both.

0.7.2 - 14 February 2024
===================
- Deprecate Listable trait in favor of ListableWidget
  - Migration: height() becomes main_axis_size()
  - Listable got deprectated, but old apps still compile
  - ListableWidget is more descriptive and using
    main_axis_size allows for reusing the trait 
    in horizontal lists, which will come in the future

0.7.1 - 10 February 2024
===================
- Bugfix: Some cases paniced when the truncated element had a size 0

0.7.0 - 3 February 2024
===================
Bump ratatui to version 0.26

0.6.1 - 12 January 2024
===================
**Bugfix**
- Correct truncation of the top element, see issues/6

0.6.0 - 23 December 2023
===================
- Bump ratatui to version 0.25
- Move crossterm from dependency to dev-dependency

0.5.0 - 28 October 2023
===================
**Breaking Changes**
- Change of Listable trait Interface returning Self instead of Option
  
**Bugfix**
- Selected widgets height is correctly calculated

The api should be more stable from now on.

0.4.0 - 25 October 2023
===================
**Breaking Changes**
- Renamed WidgetList to List
- Renamed WidgetListState to ListState
- Renamed trait WidgetItem to Listable
- Interface change of Listable

Api improvement to make the crate more reusable
and idiomatic in the ratatui ecosystem.
