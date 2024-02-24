
Released
--------
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
