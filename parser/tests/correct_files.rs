use ksp_cfg_formatter_lib::{ast_formatter::Formatter, Indentation, LineReturn};
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

macro_rules! gen_test {
    ($func_name:ident, $file:literal, $inline:literal) => {
        #[test]
        fn $func_name() {
            let text = read_local_path($file);
            let formatter = Formatter::new(Indentation::Tabs, $inline, LineReturn::Identify);
            let formatted_text = formatter.format_text(&text);
            assert_eq!(text, formatted_text);
        }
    };
}

gen_test!(simple, "tests/simple.cfg", true);

gen_test!(one_line_nodes, "tests/one_line_nodes.cfg", true);

gen_test!(sock, "tests/sock.cfg", false);

gen_test!(rn_cygnus, "tests/RO_RN_Cygnus.cfg", false);

gen_test!(
    weird_config_comments_in_empty_nodes,
    "tests/weird_configs/comments_in_empty_nodes.cfg",
    true
);

gen_test!(
    weird_config_weird_index_selection,
    "tests/weird_configs/weird_index_selection.cfg",
    true
);

gen_test!(
    weird_config_long_node_with_space,
    "tests/weird_configs/long_node_with_space.cfg",
    true
);

gen_test!(
    weird_config_multiple_equal_signs,
    "tests/weird_configs/multiple_equal_signs.cfg",
    true
);

gen_test!(has_needs_for, "tests/has_needs_for.cfg", true);
