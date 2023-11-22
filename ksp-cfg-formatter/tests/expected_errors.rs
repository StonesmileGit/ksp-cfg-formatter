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
fn missing_closing_bracket() {
    let text = read_local_path("tests/expected_errors/missing_closing_bracket.cfg");
    let res = ksp_cfg_formatter::parser::parse(&text).1;
    let exp_err = vec![ksp_cfg_formatter::parser::Error {
        severity: ksp_cfg_formatter::parser::Severity::Error,
        message: "Expected closing `]`".to_string(),
        range: ksp_cfg_formatter::parser::Range {
            start: ksp_cfg_formatter::parser::Position { line: 1, col: 11 },
            end: ksp_cfg_formatter::parser::Position { line: 1, col: 11 },
        },
        source: "".to_string(),
        context: Some(ksp_cfg_formatter::parser::Ranged::new(
            "Expected due to `[` found here".to_string(),
            ksp_cfg_formatter::parser::Range {
                start: ksp_cfg_formatter::parser::Position { line: 1, col: 5 },
                end: ksp_cfg_formatter::parser::Position { line: 1, col: 6 },
            },
        )),
    }];
    assert_eq!(res, exp_err);
}
