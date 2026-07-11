pub(crate) struct GuiInputMethodArea<'a> {
    pub(crate) content: Element<'a, Message>,
    pub(crate) request: Option<GuiImeInputMethodRequest>,
}

impl<'a> GuiInputMethodArea<'a> {
    pub(crate) fn new(
        content: Element<'a, Message>,
        request: Option<GuiImeInputMethodRequest>,
    ) -> Self {
        Self { content, request }
    }
}
