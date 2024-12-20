diff --git a/examples/demo.rs b/examples/demo.rs
index 1e664e5..7d9b9bd 100644
--- a/examples/demo.rs
+++ b/examples/demo.rs
@@ -107,7 +107,7 @@ impl TextContainer {
         }
     }
 
-    fn demo(selected_color: Color) -> ListView<'static, TextContainer> {
+    fn demo(selected_color: Color) -> ListView<'static, 'static, TextContainer> {
         let monday: Vec<String> = vec![
             String::from("1. Exercise for 30 minutes"),
             String::from("2. Work on the project for 2 hours"),
@@ -207,7 +207,7 @@ impl ColoredContainer {
         }
     }
 
-    fn demo() -> ListView<'static, ColoredContainer> {
+    fn demo() -> ListView<'static, 'static, ColoredContainer> {
         let colors = demo_colors();
         let builder = ListBuilder::new(move |context| {
             let color = demo_colors()[context.index];
diff --git a/examples/variants/classic.rs b/examples/variants/classic.rs
index f5d5a12..acb35d9 100644
--- a/examples/variants/classic.rs
+++ b/examples/variants/classic.rs
@@ -7,7 +7,7 @@ pub(crate) struct PaddedListView;
 impl PaddedListView {
     pub(crate) fn new<'a>(
         infinite_scrolling: bool,
-    ) -> ListView<'a, ListItemContainer<'a, Line<'a>>> {
+    ) -> ListView<'a, 'a, ListItemContainer<'a, Line<'a>>> {
         let builder = ListBuilder::new(|context| {
             let mut line = ListItemContainer::new(
                 Line::from(format!("Item {0}", context.index)).alignment(Alignment::Center),
diff --git a/examples/variants/config.rs b/examples/variants/config.rs
index f5d2659..277bb3b 100644
--- a/examples/variants/config.rs
+++ b/examples/variants/config.rs
@@ -42,7 +42,7 @@ impl std::fmt::Display for Variant {
 
 pub struct VariantsListView;
 impl VariantsListView {
-    pub fn new<'a>() -> ListView<'a, ListItemContainer<'a, Line<'a>>> {
+    pub fn new<'a>() -> ListView<'a, 'a, ListItemContainer<'a, Line<'a>>> {
         let builder = ListBuilder::new(move |context| {
             let config = Variant::from_index(context.index);
             let line = Line::from(format!("{config}")).alignment(Alignment::Center);
diff --git a/examples/variants/horizontal.rs b/examples/variants/horizontal.rs
index 62a8a64..4f015e1 100644
--- a/examples/variants/horizontal.rs
+++ b/examples/variants/horizontal.rs
@@ -5,7 +5,7 @@ use tui_widget_list::{ListBuilder, ListView, ScrollAxis};
 pub(crate) struct HorizontalListView;
 
 impl HorizontalListView {
-    pub(crate) fn new<'a>() -> ListView<'a, ListItemContainer<'a, Line<'a>>> {
+    pub(crate) fn new<'a>() -> ListView<'a, 'a, ListItemContainer<'a, Line<'a>>> {
         let builder = ListBuilder::new(|context| {
             let mut line = ListItemContainer::new(
                 Line::from(format!("Item {0}", context.index)).alignment(Alignment::Center),
diff --git a/examples/variants/scroll_padding.rs b/examples/variants/scroll_padding.rs
index e5a4806..a262718 100644
--- a/examples/variants/scroll_padding.rs
+++ b/examples/variants/scroll_padding.rs
@@ -5,7 +5,7 @@ use tui_widget_list::{ListBuilder, ListView};
 pub(crate) struct ScrollPaddingListView;
 
 impl ScrollPaddingListView {
-    pub(crate) fn new<'a>() -> ListView<'a, ListItemContainer<'a, Line<'a>>> {
+    pub(crate) fn new<'a, 'r>() -> ListView<'a, 'r, ListItemContainer<'a, Line<'a>>> {
         let builder = ListBuilder::new(|context| {
             let mut line = ListItemContainer::new(
                 Line::from(format!("Item {0}", context.index)).alignment(Alignment::Center),
diff --git a/src/utils.rs b/src/utils.rs
index 8ecae70..99f9c3e 100644
--- a/src/utils.rs
+++ b/src/utils.rs
@@ -370,18 +370,18 @@ fn calculate_effective_scroll_padding<T>(
     padding_by_element
 }
 
-struct WidgetCacher<'a, T> {
+struct WidgetCacher<'a, 'render, T> {
     cache: HashMap<usize, (T, u16)>,
-    builder: &'a ListBuilder<T>,
+    builder: &'a ListBuilder<'render, T>,
     scroll_axis: ScrollAxis,
     cross_axis_size: u16,
     selected: Option<usize>,
 }
 
-impl<'a, T> WidgetCacher<'a, T> {
+impl<'a, 'render, T> WidgetCacher<'a, 'render, T> {
     // Create a new WidgetCacher
     fn new(
-        builder: &'a ListBuilder<T>,
+        builder: &'a ListBuilder<'render, T>,
         scroll_axis: ScrollAxis,
         cross_axis_size: u16,
         selected: Option<usize>,
diff --git a/src/view.rs b/src/view.rs
index d93735b..65edea4 100644
--- a/src/view.rs
+++ b/src/view.rs
@@ -1,3 +1,5 @@
+use std::marker::PhantomData;
+
 use ratatui::{
     buffer::Buffer,
     layout::{Position, Rect},
@@ -10,12 +12,12 @@ use crate::{utils::layout_on_viewport, ListState};
 /// A struct representing a list view.
 /// The widget displays a scrollable list of items.
 #[allow(clippy::module_name_repetitions)]
-pub struct ListView<'a, T> {
+pub struct ListView<'block, 'render, T> {
     /// The total number of items in the list
     pub item_count: usize,
 
     ///  A `ListBuilder<T>` responsible for constructing the items in the list.
-    pub builder: ListBuilder<T>,
+    pub builder: ListBuilder<'render, T>,
 
     /// Specifies the scroll axis. Either `Vertical` or `Horizontal`.
     pub scroll_axis: ScrollAxis,
@@ -24,7 +26,7 @@ pub struct ListView<'a, T> {
     pub style: Style,
 
     /// The base block surrounding the widget list.
-    pub block: Option<Block<'a>>,
+    pub block: Option<Block<'block>>,
 
     /// The scroll padding.
     pub(crate) scroll_padding: u16,
@@ -34,10 +36,10 @@ pub struct ListView<'a, T> {
     pub(crate) infinite_scrolling: bool,
 }
 
-impl<'a, T> ListView<'a, T> {
+impl<'block, 'render, T> ListView<'block, 'render, T> {
     /// Creates a new `ListView` with a builder an item count.
     #[must_use]
-    pub fn new(builder: ListBuilder<T>, item_count: usize) -> Self {
+    pub fn new(builder: ListBuilder<'render, T>, item_count: usize) -> Self {
         Self {
             builder,
             item_count,
@@ -63,7 +65,7 @@ impl<'a, T> ListView<'a, T> {
 
     /// Sets the block style that surrounds the whole List.
     #[must_use]
-    pub fn block(mut self, block: Block<'a>) -> Self {
+    pub fn block(mut self, block: Block<'block>) -> Self {
         self.block = Some(block);
         self
     }
@@ -97,7 +99,7 @@ impl<'a, T> ListView<'a, T> {
     }
 }
 
-impl<T> Styled for ListView<'_, T> {
+impl<T> Styled for ListView<'_, '_, T> {
     type Item = Self;
 
     fn style(&self) -> Style {
@@ -110,7 +112,7 @@ impl<T> Styled for ListView<'_, T> {
     }
 }
 
-impl<T: Copy + 'static> From<Vec<T>> for ListView<'_, T> {
+impl<'render, T: Copy + 'render> From<Vec<T>> for ListView<'_, 'render, T> {
     fn from(value: Vec<T>) -> Self {
         let item_count = value.len();
         let builder = ListBuilder::new(move |context| (value[context.index], 1));
@@ -136,21 +138,23 @@ pub struct ListBuildContext {
 }
 
 /// A type alias for the closure.
-type ListBuilderClosure<T> = dyn Fn(&ListBuildContext) -> (T, u16);
+type ListBuilderClosure<'render, T> = dyn Fn(&ListBuildContext) -> (T, u16) + 'render;
 
 /// The builder to for constructing list elements in a `ListView<T>`
-pub struct ListBuilder<T> {
-    closure: Box<ListBuilderClosure<T>>,
+pub struct ListBuilder<'render, T> {
+    closure: Box<ListBuilderClosure<'render, T>>,
+    _phantom: PhantomData<&'render T>,
 }
 
-impl<T> ListBuilder<T> {
+impl<'render, T> ListBuilder<'render, T> {
     /// Creates a new `ListBuilder` taking a closure as a parameter
     pub fn new<F>(closure: F) -> Self
     where
-        F: Fn(&ListBuildContext) -> (T, u16) + 'static,
+        F: Fn(&ListBuildContext) -> (T, u16) + 'render,
     {
         ListBuilder {
             closure: Box::new(closure),
+            _phantom: PhantomData::default(),
         }
     }
 
@@ -171,7 +175,7 @@ pub enum ScrollAxis {
     Horizontal,
 }
 
-impl<T: Widget> StatefulWidget for ListView<'_, T> {
+impl<T: Widget> StatefulWidget for ListView<'_, '_, T> {
     type State = ListState;
 
     fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
@@ -357,7 +361,14 @@ mod test {
         }
     }
 
-    fn test_data(total_height: u16) -> (Rect, Buffer, ListView<'static, TestItem>, ListState) {
+    fn test_data<'render>(
+        total_height: u16,
+    ) -> (
+        Rect,
+        Buffer,
+        ListView<'static, 'render, TestItem>,
+        ListState,
+    ) {
         let area = Rect::new(0, 0, 5, total_height);
         let list = ListView::new(ListBuilder::new(|_| (TestItem {}, 3)), 3);
         (area, Buffer::empty(area), list, ListState::default())
