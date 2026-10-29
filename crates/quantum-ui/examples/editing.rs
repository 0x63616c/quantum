//! Exercise the pure editor without opening a native window.
use quantum_ui::editing::Editor;

fn main() {
    let mut editor = Editor::default();
    editor.set("Quantum 🧊");
    assert_eq!(editor.text(), "Quantum 🧊");
    assert_eq!(editor.head(), editor.text().len());
}
