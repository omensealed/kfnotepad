//! Widget tree lifecycle and size forwarding methods.

macro_rules! gui_input_method_tree_size_methods {
    () => {
        fn children(&self) -> Vec<Tree> {
            vec![Tree::new(&self.content)]
        }

        fn diff(&self, tree: &mut Tree) {
            tree.diff_children(std::slice::from_ref(&self.content));
        }

        fn size(&self) -> Size<Length> {
            self.content.as_widget().size()
        }

        fn size_hint(&self) -> Size<Length> {
            self.content.as_widget().size_hint()
        }
    };
}

pub(super) use gui_input_method_tree_size_methods;
