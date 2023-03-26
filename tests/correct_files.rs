use ksp_cfg_formatter::token_formatter::{Formatter, Indentation, LineReturn};
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
    let text = read_local_path("tests/simple.cfg");
    let formatter = Formatter::new(Indentation::Tabs, true, LineReturn::Identify);
    let formatted_text = formatter.format_text(&text);
    assert_eq!(text, formatted_text);
}

#[test]
fn one_line_nodes() {
    let text = read_local_path("tests/one_line_nodes.cfg");
    let formatter = Formatter::new(Indentation::Tabs, true, LineReturn::Identify);
    let formatted_text = formatter.format_text(&text);
    assert_eq!(text, formatted_text);
}

#[test]
fn sock() {
    let text = read_local_path("tests/sock.cfg");
    let formatter = Formatter::new(Indentation::Tabs, true, LineReturn::Identify);
    let formatted_text = formatter.format_text(&text);
    assert_eq!(text, formatted_text);
}

#[test]
fn rn_cygnus() {
    let text = read_local_path("tests/RO_RN_Cygnus.cfg");
    let formatter = Formatter::new(Indentation::Tabs, true, LineReturn::Identify);
    let formatted_text = formatter.format_text(&text);
    assert_eq!(text, formatted_text);
}
