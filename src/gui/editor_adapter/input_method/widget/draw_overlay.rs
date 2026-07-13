//! Drawing, pointer interaction, and overlay forwarding methods.

macro_rules! gui_input_method_draw_overlay_methods {
    () => {
        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut iced::Renderer,
            theme: &Theme,
            style: &advanced_renderer::Style,
            layout: AdvancedLayout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }

        fn mouse_interaction(
            &self,
            tree: &Tree,
            layout: AdvancedLayout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &iced::Renderer,
        ) -> mouse::Interaction {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }

        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut Tree,
            layout: AdvancedLayout<'b>,
            renderer: &iced::Renderer,
            viewport: &Rectangle,
            translation: Vector,
        ) -> Option<advanced_overlay::Element<'b, Message, Theme, iced::Renderer>> {
            self.content.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        }
    };
}

pub(super) use gui_input_method_draw_overlay_methods;
