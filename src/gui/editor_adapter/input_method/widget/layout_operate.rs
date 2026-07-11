macro_rules! gui_input_method_layout_operate_methods {
    () => {
        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &iced::Renderer,
            limits: &advanced_layout::Limits,
        ) -> advanced_layout::Node {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        }

        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: AdvancedLayout<'_>,
            renderer: &iced::Renderer,
            operation: &mut dyn AdvancedOperation,
        ) {
            self.content
                .as_widget_mut()
                .operate(&mut tree.children[0], layout, renderer, operation);
        }
    };
}
