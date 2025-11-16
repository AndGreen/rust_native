use mf_core::View;

pub trait Backend: Send {
    fn mount(&mut self, view: &View);
    fn update(&mut self, view: &View);
}

pub fn debug_tree(view: &View) -> String {
    fn recurse(view: &View, depth: usize, out: &mut String) {
        let indent = "  ".repeat(depth);
        out.push_str(&format!("{}{}\n", indent, view.element().describe()));
        for child in view.children() {
            recurse(child, depth + 1, out);
        }
    }
    let mut buffer = String::new();
    recurse(view, 0, &mut buffer);
    buffer
}
