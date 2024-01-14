Unreleased
--------

main
====
- Improved Documentation
- Make set_num_elements of the ListState private:

Released
--------

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
