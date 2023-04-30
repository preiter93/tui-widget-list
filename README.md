## Widget list implementation for TUI/Ratatui


[Ratatui](https://github.com/tui-rs-revival/ratatui) is a UI Framework to build terminal user interfaces. Ratatui itself provides
some base widgets such as a list of Texts.

This library provides an extension to render a list of arbitrary widgets.


### Documentation

The documentation can be found on [docs.rs.](https://docs.rs/tui-widget-list)

### Demo
```rust
cargo run --example paragraph_list
```

### Usage
Items of [`WidgetList`] or of the convenience class [`SelectableWidgetList`]
must implement the [`ListableWidget`] trait. Then the render() method is available
on the widget list.

For an implementation example see [`SelectableWidgetList`].

![](img/screenshot.png)
![](img/screen.png)
![](img/screen.png)
