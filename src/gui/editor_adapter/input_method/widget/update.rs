macro_rules! gui_input_method_update_methods {
    () => {
        fn update(
            &mut self,
            tree: &mut Tree,
            event: &Event,
            layout: AdvancedLayout<'_>,
            cursor: mouse::Cursor,
            renderer: &iced::Renderer,
            clipboard: &mut dyn AdvancedClipboard,
            shell: &mut AdvancedShell<'_, Message>,
            viewport: &Rectangle,
        ) {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            if let (Event::Window(window::Event::RedrawRequested(_)), Some(request)) =
                (event, &self.request)
            {
                shell.request_input_method(&input_method::InputMethod::Enabled {
                    cursor: request.cursor_rect(layout.bounds()),
                    purpose: input_method::Purpose::Normal,
                    preedit: request.preedit.as_ref().map(input_method::Preedit::as_ref),
                });
            }
        }
    };
}
