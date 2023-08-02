use ksp_cfg_formatter::pest_validate;
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

#[test]
fn error_test() {
    for path in files_from_path(&"tests/error_files".to_string()) {
        let input = read_local_path(&path);
        let res = pest_validate(&input);
        dbg!(&res);
        assert!(res.is_err());
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
