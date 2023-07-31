use ksp_cfg_formatter_lib::{Formatter, Indentation, LineReturn};
#[cfg(test)]
use pretty_assertions::assert_eq;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn read_local_path(path: &str) -> String {
    let base_path = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(base_path).join(PathBuf::from(path));
    fs::read_to_string(path).expect("Failed to read path provided")
}

#[test]
fn simple() {
    let input = read_local_path("tests/incorrect_files/simple_input.cfg");
    let output = read_local_path("tests/incorrect_files/simple_output.cfg");
    let formatter = Formatter::new(Indentation::Tabs, true, LineReturn::Identify);
    let formatted_text = formatter.format_text(&input);
    assert_eq!(output, formatted_text);
}
