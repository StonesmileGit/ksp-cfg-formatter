use ksp_cfg_formatter::{Formatter, Indentation, LineReturn};
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
    let formatter = Formatter::new(Indentation::Tabs, Some(true), LineReturn::Identify);
    let formatted_text = formatter.format_text(&input).unwrap();
    assert_eq!(output, formatted_text);
}

#[test]
fn trailing_spaces() {
    let input = read_local_path("tests/incorrect_files/trailing_spaces_input.cfg");
    let output = read_local_path("tests/incorrect_files/trailing_spaces_output.cfg");
    let formatter = Formatter::new(Indentation::Spaces(4), Some(true), LineReturn::Identify);
    let formatted_text = formatter.format_text(&input).unwrap();
    assert_eq!(output, formatted_text);
}
