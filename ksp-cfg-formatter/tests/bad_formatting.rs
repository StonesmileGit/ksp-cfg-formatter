use ksp_cfg_formatter::parse_to_ast;
#[cfg(test)]
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

fn read_local_path(path: &str) -> String {
    let base_path = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(base_path).join(PathBuf::from(path));
    fs::read_to_string(path).expect("Failed to read path provided")
}

// macro_rules! gen_test {
//     ($func_name:ident, $file:literal) => {
//         #[test]
//         fn $func_name() {
//             let text = read_local_path($file);
//             match ksp_cfg_formatter::parse_to_ast(&text) {
//                 Ok(_) => (),
//                 Err(err) => panic!("{}", err.first().unwrap()),
//             }
//         }
//     };
// }

#[test]
fn bad_formatting() {
    for path in files_from_path(&"tests/bad_formatting".to_string()) {
        let input = read_local_path(&path);
        match parse_to_ast(&input) {
            Ok(_) => (),
            Err(err) => {
                if err
                    .iter()
                    .filter(|e| {
                        matches!(e.severity, ksp_cfg_formatter::parser::nom::Severity::Error)
                    })
                    .count()
                    > 0
                {
                    panic!("{}: {:?}", path, err)
                }
            }
        }
    }
}

fn files_from_path(path: &String) -> Vec<String> {
    let mut paths = Vec::new();
    for path in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        let name = path.path().to_owned();
        if let Some(extension) = name.extension() {
            if extension == "cfg" {
                if let Some(name) = name.to_str() {
                    paths.push(name.to_owned());
                };
            }
        }
    }
    paths
}
